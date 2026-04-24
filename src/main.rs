use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use osc8::Hyperlink;
use std::env;
use std::net::SocketAddr;
use tokio::fs;
use tower_http::services::ServeDir;

// utils
mod utils;

// models
mod models;

// handlers
mod handlers;
use handlers::{file_handler, page_handler, wifi_api_handler};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    let upload_dir = "uploads";
    let _ = fs::create_dir_all(upload_dir).await;

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let my_ip = local_ip_address::local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());

    let api_routes = Router::new()
        .route("/wifi", get(wifi_api_handler::wifi_api_handler))
        .route("/files", get(file_handler::list_files_handler))
        .route("/delete/:file_name", get(file_handler::delete_handler))
        .route("/clean-uploads", post(file_handler::clean_uploads_handler));

    let app = Router::new()
        .route("/", get(page_handler::index_handler))
        .nest("/api", api_routes)
        .route("/upload", post(file_handler::upload_handler))
        .layer(DefaultBodyLimit::max(5368709120))
        .nest_service("/files", ServeDir::new(upload_dir));

    println!("-------------------------------------------");
    println!("🚀 CONNECT OS v2: NODE STARTED (OPTIMIZED FOR LARGE DATA)");
    println!("📡 Listening on PORT: {}", port);
    let local_url = format!("http://localhost:{}", port);
    println!(
        "{}{}{}",
        Hyperlink::new(&local_url),
        format!("🏠 Local: {}", local_url),
        Hyperlink::END
    );
    let lan_url = format!("http://{}:{}", my_ip, port);
    println!(
        "{}{}{}",
        Hyperlink::new(&lan_url),
        format!("🌐 LAN:   {}", lan_url),
        Hyperlink::END
    );
    println!("-------------------------------------------");

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let lan_url = format!("http://{}:{}", my_ip, port);
            open_browser(&lan_url);
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            println!("❌ Port đang bị dùng: {}", e);
        }
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
