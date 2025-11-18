use crate::auth::AuthUser;
use crate::config::Config;
use crate::models::{CreateUploadLog, SpotifyDownloadRequest, UploadResponse};
use crate::paths::get_user_directories;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

pub async fn download_spotify(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<SpotifyDownloadRequest>,
) -> Result<Json<UploadResponse>, Response> {
    // Generate session ID for progress tracking
    let session_id = uuid::Uuid::new_v4().to_string();
    // Check if Spotify downloads are enabled
    if !state.config.spotify.enabled {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Spotify downloads are disabled"
            })),
        )
            .into_response());
    }

    // Get user from database to access library_path
    let db_user = state
        .db
        .get_user_by_id(&user.user_id)
        .await
        .map_err(|e| internal_error(&format!("Failed to get user: {}", e)))?;

    // Get user-specific directories
    let (music_dir, temp_dir) = get_user_directories(&state.config, &db_user.library_path)
        .await
        .map_err(|e| {
            internal_error(&format!("Failed to get user directories: {}", e))
        })?;

    tracing::info!(
        "User {} downloading Spotify to music_dir: {}, temp_dir: {}",
        user.username,
        music_dir.display(),
        temp_dir.display()
    );

    // SECURITY: Strict URL validation to prevent command injection
    // Only allow HTTPS Spotify URLs with specific patterns
    let url = req.url.trim();
    let is_valid = (url.starts_with("https://open.spotify.com/track/")
        || url.starts_with("https://open.spotify.com/album/")
        || url.starts_with("https://open.spotify.com/playlist/")
        || url.starts_with("https://open.spotify.com/artist/"))
        && !url.contains(';')
        && !url.contains('|')
        && !url.contains('`')
        && !url.contains('$')
        && !url.contains("&&")
        && !url.contains("||");

    if !is_valid || url.len() > 300 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid Spotify URL format. Must be a standard HTTPS Spotify URL (track, album, playlist, or artist)."
            })),
        )
            .into_response());
    }

    // Send initial progress
    crate::progress::send_progress(&state.progress_store, &session_id, "Starting Spotify download...".to_string()).await;

    // Create upload log
    let log_id = state
        .db
        .create_upload_log(CreateUploadLog {
            user_id: user.user_id.clone(),
            upload_type: "spotify".to_string(),
            source: req.url.clone(),
        })
        .await
        .map_err(|e| internal_error(&format!("Failed to create upload log: {}", e)))?;

    // Update status to processing
    state
        .db
        .update_upload_log_status(log_id, "processing", None, None)
        .await
        .map_err(|e| internal_error(&format!("Failed to update log: {}", e)))?;

    // Download with spotdl
    crate::progress::send_progress(&state.progress_store, &session_id, "Downloading from Spotify...".to_string()).await;
    let result = download_with_spotdl(&state.config, &temp_dir, &req.url).await;

    match result {
        Ok(file_count) => {
            // Process with Ferric (check database for ferric_enabled setting)
            crate::progress::send_progress(&state.progress_store, &session_id, format!("Downloaded {} file(s), now processing...", file_count)).await;
            match process_temp_dir(&state, &temp_dir, &music_dir).await {
                Ok(_) => {
                    state
                        .db
                        .update_upload_log_status(log_id, "completed", Some(file_count), None)
                        .await
                        .map_err(|e| internal_error(&format!("Failed to update log: {}", e)))?;

                    crate::progress::send_progress(&state.progress_store, &session_id, "✓ Complete!".to_string()).await;
                    // Cleanup session after a short delay
                    let store = state.progress_store.clone();
                    let sid = session_id.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        crate::progress::unregister_session(&store, &sid).await;
                    });

                    Ok(Json(UploadResponse {
                        success: true,
                        message: format!(
                            "Successfully downloaded and processed {} file(s)",
                            file_count
                        ),
                        log_id: Some(log_id),
                        session_id: Some(session_id),
                    }))
                }
                Err(e) => {
                    let error_msg = format!("Processing failed: {}", e);
                    state
                        .db
                        .update_upload_log_status(
                            log_id,
                            "failed",
                            Some(file_count),
                            Some(error_msg.clone()),
                        )
                        .await
                        .ok();

                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": error_msg })),
                    )
                        .into_response())
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Download failed: {}", e);
            state
                .db
                .update_upload_log_status(log_id, "failed", Some(0), Some(error_msg.clone()))
                .await
                .ok();

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": error_msg })),
            )
                .into_response())
        }
    }
}

async fn download_with_spotdl(
    config: &Config,
    temp_dir: &PathBuf,
    url: &str,
) -> anyhow::Result<i32> {
    let args = build_spotdl_args(config, temp_dir, url);

    let output = tokio::process::Command::new(&config.spotify.spotdl_path)
        .args(&args)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!("spotdl failed: {}\n{}", stderr, stdout);
    }

    // Count downloaded files
    let mut count = 0;
    let mut entries = fs::read_dir(temp_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            count += 1;
        }
    }

    Ok(count)
}

fn build_spotdl_args(config: &Config, temp_dir: &PathBuf, url: &str) -> Vec<String> {
    // SpotDL expects a file pattern, not just a directory
    // Pattern: {output_dir}/{artist} - {title}.{output-ext}
    let output_pattern = format!("{}/{{artist}} - {{title}}.{{output-ext}}", temp_dir.display());

    vec![
        "download".to_string(),
        url.to_string(),
        "--output".to_string(),
        output_pattern,
        "--format".to_string(),
        config.spotify.audio_format.clone(),
    ]
}

async fn process_temp_dir(
    state: &Arc<crate::AppState>,
    temp_dir: &PathBuf,
    music_dir: &PathBuf,
) -> anyhow::Result<()> {
    // Check database for ferric_enabled setting (overrides config file)
    let ferric_enabled = state
        .db
        .get_ferric_enabled(&state.config)
        .await
        .unwrap_or(state.config.paths.ferric_enabled);

    if ferric_enabled {
        // Call Ferric to process the files in temp dir
        tracing::info!("Ferric enabled: processing files");
        let output = tokio::process::Command::new(&state.config.paths.ferric_path)
            .arg("--input-dir")
            .arg(temp_dir)
            .arg("--output-dir")
            .arg(music_dir)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Ferric processing failed: {}", stderr);
        }
    } else {
        // Ferric disabled: just move files directly to music_dir
        tracing::info!("Ferric disabled: moving files directly to music directory");
        let mut entries = fs::read_dir(temp_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                let dest = music_dir.join(entry.file_name());
                // Use copy+remove instead of rename to handle cross-filesystem moves
                fs::copy(entry.path(), &dest).await?;
                fs::remove_file(entry.path()).await?;
            }
        }
    }

    // Clean up temp directory
    let mut entries = fs::read_dir(temp_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            fs::remove_file(entry.path()).await.ok();
        }
    }

    Ok(())
}

fn internal_error(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": message
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_args_correct_format() {
        let config = Config::default();
        let temp_dir = PathBuf::from("/tmp/test");
        let url = "https://open.spotify.com/track/example";
        let args = build_spotdl_args(&config, &temp_dir, url);

        assert_eq!(args[0], "download");
        assert_eq!(args[1], url);
        assert_eq!(args[2], "--output");
        assert_eq!(args[3], "/tmp/test/{artist} - {title}.{output-ext}");
        assert_eq!(args[4], "--format");
        assert_eq!(args[5], "opus");
    }

    #[test]
    fn build_args_handles_different_urls() {
        let config = Config::default();
        let temp_dir = PathBuf::from("/srv/music/tmp");
        let urls = vec![
            "https://open.spotify.com/track/123",
            "https://open.spotify.com/album/456",
            "https://open.spotify.com/playlist/789",
        ];

        for url in urls {
            let args = build_spotdl_args(&config, &temp_dir, url);
            assert_eq!(args.len(), 6); // Now includes --format opus
            assert_eq!(args[0], "download");
            assert_eq!(args[1], url);
            assert_eq!(args[4], "--format");
            assert_eq!(args[5], "opus");
        }
    }
}
