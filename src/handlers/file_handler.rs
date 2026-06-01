use crate::AppState;
use crate::models::file_info::FileInfo;
use crate::utils::cleanup::clean_uploads;
use crate::utils::size::format_size;
use axum::extract::{Multipart, Path, State};
use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
pub struct AuthRequest {
    pin: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    success: bool,
}

pub async fn auth_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthRequest>,
) -> Json<AuthResponse> {
    let success = payload.pin == state.pin;
    Json(AuthResponse { success })
}

pub async fn list_files_handler() -> impl IntoResponse {
    let mut files = Vec::new();

    if let Ok(mut entries) = tokio::fs::read_dir("uploads").await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and directories
            if name.starts_with('.') {
                continue;
            }

            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    let size = metadata.len();
                    files.push(FileInfo {
                        name: name.clone(),
                        url: format!("/files/{}", name),
                        size,
                    });
                }
            }
        }
    }

    Json(files)
}

pub async fn delete_handler(Path(file_name): Path<String>) -> impl IntoResponse {
    let safe_name = file_name.replace("/", "").replace("..", "");
    let path = format!("uploads/{}", safe_name);
    match tokio::fs::remove_file(&path).await {
        Ok(_) => {
            println!("🗑️ Deleted: {}", safe_name);
            (axum::http::StatusCode::OK, "Xóa file thành công").into_response()
        }
        Err(_) => (
            axum::http::StatusCode::NOT_FOUND,
            "File not found",
        )
            .into_response(),
    }
}

pub async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    // For now, simple upload without auth
    // TODO: Add auth back when needed
    let mut result = "Upload completed".to_string();

    if let Ok(Some(mut field)) = multipart.next_field().await {
        if let Some(file_name) = field.file_name() {
            let safe_name = file_name.replace("/", "").replace("..", "");
            let path = format!("uploads/{}", safe_name);

            if let Ok(mut file) = tokio::fs::File::create(&path).await {
                let mut received: usize = 0;
                while let Ok(Some(chunk)) = field.chunk().await {
                    received += chunk.len();
                    if let Err(_) = file.write_all(&chunk).await {
                        result = "Upload failed".to_string();
                        break;
                    }
                }
                if result == "Upload completed" {
                    println!(
                        "✅ Saved file: {} - Size: {}",
                        safe_name,
                        format_size(received)
                    );
                }
            } else {
                result = "Cannot create file".to_string();
            }
        }
    }

    (axum::http::StatusCode::OK, result).into_response()
}

pub async fn clean_uploads_handler() -> impl IntoResponse {
    match clean_uploads().await {
        Ok(_) => (axum::http::StatusCode::OK, "Dọn dẹp uploads thành công").into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi dọn dẹp: {}", e),
        )
            .into_response(),
    }
}
