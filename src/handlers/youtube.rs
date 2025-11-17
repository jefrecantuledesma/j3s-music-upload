use crate::auth::AuthUser;
use crate::config::Config;
use crate::models::{CreateUploadLog, UploadResponse, YoutubeDownloadRequest};
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

pub async fn download_youtube(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<YoutubeDownloadRequest>,
) -> Result<Json<UploadResponse>, Response> {
    // Check if YouTube downloads are enabled
    if !state.config.youtube.enabled {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "YouTube downloads are disabled"
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
        "User {} downloading YouTube to music_dir: {}, temp_dir: {}",
        user.username,
        music_dir.display(),
        temp_dir.display()
    );

    // SECURITY: Strict URL validation to prevent command injection
    // Only allow HTTPS YouTube URLs with specific patterns
    let url = req.url.trim();
    let is_valid = (url.starts_with("https://www.youtube.com/watch?v=")
        || url.starts_with("https://youtube.com/watch?v=")
        || url.starts_with("https://youtu.be/")
        || url.starts_with("https://m.youtube.com/watch?v="))
        && !url.contains(';')
        && !url.contains('|')
        && !url.contains('`')
        && !url.contains('$')
        && !url.contains("&&")
        && !url.contains("||");

    if !is_valid || url.len() > 200 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid YouTube URL format. Must be a standard HTTPS YouTube watch URL."
            })),
        )
            .into_response());
    }

    // Create upload log
    let log_id = state
        .db
        .create_upload_log(CreateUploadLog {
            user_id: user.user_id.clone(),
            upload_type: "youtube".to_string(),
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

    // Download with yt-dlp
    let result = download_with_ytdlp(&state.config, &temp_dir, &req.url).await;

    match result {
        Ok(file_count) => {
            // Process with Ferric
            match process_temp_dir(&state.config, &temp_dir, &music_dir).await {
                Ok(_) => {
                    state
                        .db
                        .update_upload_log_status(log_id, "completed", Some(file_count), None)
                        .await
                        .map_err(|e| internal_error(&format!("Failed to update log: {}", e)))?;

                    Ok(Json(UploadResponse {
                        success: true,
                        message: format!(
                            "Successfully downloaded and processed {} file(s)",
                            file_count
                        ),
                        log_id: Some(log_id),
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

async fn download_with_ytdlp(config: &Config, temp_dir: &PathBuf, url: &str) -> anyhow::Result<i32> {
    let args = build_ytdlp_args(config, temp_dir, url);

    let output = tokio::process::Command::new(&config.youtube.ytdlp_path)
        .args(&args)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp failed: {}", stderr);
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

fn build_ytdlp_args(config: &Config, temp_dir: &PathBuf, url: &str) -> Vec<String> {
    let mut args = vec![
        "--no-warnings".to_string(),
        "--extract-audio".to_string(),
        "--audio-format".to_string(),
        config.youtube.audio_format.clone(),
        "--output".to_string(),
        format!("{}/%(title)s.%(ext)s", temp_dir.display()),
        "--no-playlist".to_string(),
        "--ignore-no-formats-error".to_string(),
    ];

    let format_selector = config.youtube.format_selector.trim();
    if !format_selector.is_empty() {
        args.push("--format".to_string());
        args.push(format_selector.to_string());
    }

    if let Some(client) = config.youtube.player_client.as_deref() {
        let trimmed = client.trim();
        if !trimmed.is_empty() {
            // Force yt-dlp to use a stable player client (web avoids "Precondition check failed").
            args.push("--extractor-args".to_string());
            args.push(format!("youtube:player_client={}", trimmed));
        }
    }

    if !config.youtube.extra_args.is_empty() {
        args.extend(config.youtube.extra_args.iter().cloned());
    }

    args.push(url.to_string());
    args
}

async fn process_temp_dir(config: &Config, temp_dir: &PathBuf, music_dir: &PathBuf) -> anyhow::Result<()> {
    // Call Ferric to process the files in temp dir
    let output = tokio::process::Command::new(&config.paths.ferric_path)
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
    fn build_args_include_default_player_client() {
        let config = Config::default();
        let temp_dir = PathBuf::from("/tmp/test");
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let args = build_ytdlp_args(&config, &temp_dir, url);

        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--extractor-args" && pair[1] == "youtube:player_client=web"),
            "expected --extractor-args youtube:player_client=web in {:?}",
            args
        );
        assert_eq!(args.last().unwrap(), url);
    }

    #[test]
    fn build_args_append_extra_args() {
        let mut config = Config::default();
        config.youtube.extra_args = vec!["--throttled-rate=100K".to_string()];
        let temp_dir = PathBuf::from("/tmp/test");
        let url = "https://youtu.be/example";

        let args = build_ytdlp_args(&config, &temp_dir, url);

        assert!(args.contains(&"--throttled-rate=100K".to_string()));
        assert_eq!(args.last().unwrap(), url);
    }
}
