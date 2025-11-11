use crate::auth::AuthUser;
use crate::config::Config;
use crate::models::{CreateUploadLog, UploadResponse, YoutubeDownloadRequest};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
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

    // Validate URL
    if !req.url.contains("youtube.com") && !req.url.contains("youtu.be") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid YouTube URL"
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
    let result = download_with_ytdlp(&state.config, &req.url).await;

    match result {
        Ok(file_count) => {
            // Process with Ferric
            match process_temp_dir(&state.config).await {
                Ok(_) => {
                    state
                        .db
                        .update_upload_log_status(log_id, "completed", Some(file_count), None)
                        .await
                        .map_err(|e| internal_error(&format!("Failed to update log: {}", e)))?;

                    Ok(Json(UploadResponse {
                        success: true,
                        message: format!("Successfully downloaded and processed {} file(s)", file_count),
                        log_id: Some(log_id),
                    }))
                }
                Err(e) => {
                    let error_msg = format!("Processing failed: {}", e);
                    state
                        .db
                        .update_upload_log_status(log_id, "failed", Some(file_count), Some(error_msg.clone()))
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

async fn download_with_ytdlp(config: &Config, url: &str) -> anyhow::Result<i32> {
    let output = tokio::process::Command::new(&config.youtube.ytdlp_path)
        .args([
            "--extract-audio",
            "--audio-format", &config.youtube.audio_format,
            "--output", &format!("{}/%(title)s.%(ext)s", config.paths.temp_dir.display()),
            "--no-playlist",
            url,
        ])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp failed: {}", stderr);
    }

    // Count downloaded files
    let mut count = 0;
    let mut entries = fs::read_dir(&config.paths.temp_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_file() {
            count += 1;
        }
    }

    Ok(count)
}

async fn process_temp_dir(config: &Config) -> anyhow::Result<()> {
    // Call Ferric to process the files in temp dir
    let output = tokio::process::Command::new(&config.paths.ferric_path)
        .arg("--input-dir")
        .arg(&config.paths.temp_dir)
        .arg("--output-dir")
        .arg(&config.paths.music_dir)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Ferric processing failed: {}", stderr);
    }

    // Clean up temp directory
    let mut entries = fs::read_dir(&config.paths.temp_dir).await?;
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
