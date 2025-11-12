use crate::auth::AuthUser;
use crate::config::Config;
use crate::models::{CreateUploadLog, UploadResponse};
use axum::{
    extract::{Extension, Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub async fn upload_files(
    State(state): State<Arc<crate::AppState>>,
    Extension(user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, Response> {
    let mut uploaded_files = Vec::new();
    let mut file_count = 0;

    // Create upload log
    let log_id = state
        .db
        .create_upload_log(CreateUploadLog {
            user_id: user.user_id.clone(),
            upload_type: "file".to_string(),
            source: "multipart upload".to_string(),
        })
        .await
        .map_err(|e| internal_error(&format!("Failed to create upload log: {}", e)))?;

    // Update status to processing
    state
        .db
        .update_upload_log_status(log_id, "processing", None, None)
        .await
        .map_err(|e| internal_error(&format!("Failed to update log: {}", e)))?;

    // Process each file in the multipart upload
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| internal_error(&format!("Failed to read field: {}", e)))?
    {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => continue,
        };

        // SECURITY: Sanitize filename to prevent path traversal attacks
        // Remove any path components and only keep the filename
        let sanitized_name = Path::new(&file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| internal_error("Invalid filename"))?
            .to_string();

        // Additional check: reject files with suspicious characters
        if sanitized_name.contains("..")
            || sanitized_name.contains('/')
            || sanitized_name.contains('\\')
        {
            let error_msg = "Invalid filename: path traversal attempt detected";
            state
                .db
                .update_upload_log_status(
                    log_id,
                    "failed",
                    Some(file_count),
                    Some(error_msg.to_string()),
                )
                .await
                .ok();
            return Err(
                (StatusCode::BAD_REQUEST, Json(json!({ "error": error_msg }))).into_response(),
            );
        }

        // Check file extension
        let extension = Path::new(&sanitized_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !state
            .config
            .upload
            .allowed_extensions
            .contains(&extension.to_string())
        {
            let error_msg = format!("File type .{} not allowed", extension);
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
            return Err(
                (StatusCode::BAD_REQUEST, Json(json!({ "error": error_msg }))).into_response(),
            );
        }

        // Read file data
        let data = field
            .bytes()
            .await
            .map_err(|e| internal_error(&format!("Failed to read file: {}", e)))?;

        // Check file size
        if data.len() > state.config.max_file_size_bytes() {
            let error_msg = format!(
                "File too large: {} MB (max: {} MB)",
                data.len() / 1024 / 1024,
                state.config.upload.max_file_size_mb
            );
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
            return Err(
                (StatusCode::BAD_REQUEST, Json(json!({ "error": error_msg }))).into_response(),
            );
        }

        // Save to temp directory (using sanitized filename)
        let temp_path = state.config.paths.temp_dir.join(&sanitized_name);
        let mut file = File::create(&temp_path)
            .await
            .map_err(|e| internal_error(&format!("Failed to create file: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| internal_error(&format!("Failed to write file: {}", e)))?;

        uploaded_files.push(temp_path);
        file_count += 1;
    }

    if uploaded_files.is_empty() {
        state
            .db
            .update_upload_log_status(
                log_id,
                "failed",
                Some(0),
                Some("No files uploaded".to_string()),
            )
            .await
            .ok();
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "No files uploaded" })),
        )
            .into_response());
    }

    // Process files with Ferric
    let result = process_with_ferric(&state.config, &uploaded_files).await;

    match result {
        Ok(_) => {
            state
                .db
                .update_upload_log_status(log_id, "completed", Some(file_count), None)
                .await
                .map_err(|e| internal_error(&format!("Failed to update log: {}", e)))?;

            Ok(Json(UploadResponse {
                success: true,
                message: format!("Successfully uploaded and processed {} file(s)", file_count),
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

async fn process_with_ferric(config: &Config, files: &[std::path::PathBuf]) -> anyhow::Result<()> {
    // Call Ferric to process the files
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

    // Clean up temp files
    for file in files {
        if file.exists() {
            fs::remove_file(file).await.ok();
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
