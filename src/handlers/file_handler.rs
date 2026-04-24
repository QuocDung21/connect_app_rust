use axum::extract::Path;
use axum::{Json, response::IntoResponse};
use tokio::fs;

use crate::models::file_info::FileInfo;
use crate::utils::cleanup::clean_uploads;
use crate::utils::size::format_size;

pub async fn upload_handler(mut multipart: axum::extract::Multipart) -> impl IntoResponse {
    use axum::response::Html;
    use tokio::io::{AsyncWriteExt, BufWriter};

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unnamed_file").to_string();
        let safe_name = file_name.replace("/", "").replace("..", "");
        let path = format!("uploads/{}", safe_name);

        let file = match fs::File::create(&path).await {
            Ok(f) => f,
            Err(_) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Lỗi tạo file",
                )
                    .into_response();
            }
        };
        let mut buffered_file = BufWriter::with_capacity(64 * 1024, file);

        // Tránh dùng TryStreamExt vào lúc này để đỡ gặp rắc rối compile, dùng trực tiếp vòng lặp stream của Axum
        let mut stream = field;
        let mut received: usize = 0;

        while let Some(chunk_result) = stream.chunk().await.unwrap_or(None) {
            received += chunk_result.len();
            if let Err(_) = buffered_file.write_all(&chunk_result).await {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Lỗi ghi dữ liệu",
                )
                    .into_response();
            }
        }
        let _ = buffered_file.flush().await;
        println!(
            "✅ Saved Large File: {} - Size: {}",
            safe_name,
            format_size(received)
        );
    }
    Html("<h2>Upload thành công!</h2><a href='/'>Quay lại Dashboard</a>").into_response()
}

pub async fn delete_handler(Path(file_name): Path<String>) -> impl IntoResponse {
    let safe_name = file_name.replace("/", "").replace("..", "");
    let path = format!("uploads/{}", safe_name);
    match fs::remove_file(&path).await {
        Ok(_) => {
            println!("🗑️ Deleted: {}", safe_name);
            (axum::http::StatusCode::OK, "Xóa file thành công").into_response()
        }
        Err(e) => (
            axum::http::StatusCode::NOT_FOUND,
            format!("Không tìm thấy: {}", e),
        )
            .into_response(),
    }
}

pub async fn list_files_handler() -> impl IntoResponse {
    let mut files = Vec::new();
    let mut entries = fs::read_dir("uploads").await.unwrap();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();

        let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);

        files.push(FileInfo {
            url: format!("/files/{}", name),
            name,
            size,
        });
    }
    Json(files)
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
