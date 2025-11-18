use crate::auth::AuthUser;
use crate::models::{CreateUploadLog, UploadResponse};
use crate::paths::get_user_directories;
use axum::{
    extract::{Extension, Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::path::{Path, PathBuf};
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
        "User {} uploading to music_dir: {}, temp_dir: {}",
        user.username,
        music_dir.display(),
        temp_dir.display()
    );

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
        let temp_path = temp_dir.join(&sanitized_name);
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

    // Process files with Ferric (check database for ferric_enabled setting)
    let result = process_with_ferric(&state, &temp_dir, &music_dir, &uploaded_files).await;

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
                session_id: None,  // TODO: Add progress tracking to upload
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

async fn process_with_ferric(
    state: &Arc<crate::AppState>,
    temp_dir: &PathBuf,
    music_dir: &PathBuf,
    files: &[std::path::PathBuf],
) -> anyhow::Result<()> {
    // Check database for ferric_enabled setting (overrides config file)
    let ferric_enabled = state
        .db
        .get_ferric_enabled(&state.config)
        .await
        .unwrap_or(state.config.paths.ferric_enabled);

    if ferric_enabled {
        // Call Ferric to process the files
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
        for file in files {
            if let Some(filename) = file.file_name() {
                let dest = music_dir.join(filename);
                // Use copy+remove instead of rename to handle cross-filesystem moves
                fs::copy(file, &dest).await?;
                fs::remove_file(file).await?;
            }
        }
    }

    // Clean up remaining temp files
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
