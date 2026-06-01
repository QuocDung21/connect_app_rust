use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use mdns_sd::{ServiceDaemon, ServiceInfo};
// use osc8::Hyperlink;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs;
use tower_http::services::ServeDir;
// utils
mod utils;

// models
mod models;

// handlers
mod handlers;
use handlers::{file_handler, page_handler};

#[derive(Clone)]
struct AppState {
    pin: String,
    #[allow(dead_code)]
    salt: [u8; 32],
}

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    // 1. Khởi tạo thư mục và cấu hình cơ bản
    let upload_dir = "uploads";
    let _ = fs::create_dir_all(upload_dir).await;

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT phải là một con số");

    let my_ip = local_ip_address::local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let pin = env::var("PIN").unwrap_or_else(|_| {
        // Generate random 4-digit PIN if not set
        use rand::Rng;
        let mut rng = rand::thread_rng();
        format!("{:04}", rng.gen_range(1000..10000))
    });

    let salt = utils::encryption::generate_salt();
    println!("🔐 Access PIN: {}", pin);

    let state = Arc::new(AppState {
        pin: pin.clone(),
        salt,
    });

    // 2. Cấu hình mDNS (Đặt nickname bebu.local cho IP LAN)
    let instance_name = "bebu";
    let mdns = ServiceDaemon::new().expect("Không thể khởi tạo mDNS daemon");
    let service_type = "_http._tcp.local.";
    let host_name = format!("{}.local.", instance_name);

    let mut properties = HashMap::new();
    properties.insert("path".to_string(), "/".to_string());

    let my_service = ServiceInfo::new(
        service_type,
        instance_name,
        &host_name,
        my_ip.to_string(), // Đã sửa lỗi biến my_ip_str ở đây
        port,
        Some(properties),
    )
    .expect("Không thể tạo thông tin mDNS Service");

    mdns.register(my_service).expect("Không thể đăng ký mDNS");

    // 3. Xây dựng Route cho API
    let api_routes = Router::new()
        .route("/files", get(file_handler::list_files_handler))
        .route("/delete/:file_name", get(file_handler::delete_handler))
        .route("/clean-uploads", post(file_handler::clean_uploads_handler))
        .route("/auth", post(file_handler::auth_handler))
        .with_state(state.clone());

    // 4. Xây dựng App chính
    let app = Router::new()
        .route("/", get(page_handler::index_handler))
        .nest("/api", api_routes)
        .route("/upload", post(file_handler::upload_handler))
        .with_state(state)
        // Giới hạn 5GB cho dữ liệu lớn
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024 * 1024))
        .nest_service("/files", ServeDir::new(upload_dir))
        .nest_service("/locales", ServeDir::new("templates/locales"));

    // 5. In log thông báo xịn xò
    println!("-------------------------------------------");
    println!("🍭 GÓC CHIA SẺ BÉ BỰ ĐÃ SẴN SÀNG!");
    println!("📡 Đang lắng nghe tại cổng: {}", port);
    println!("🌐 LAN IP:  http://{}:{}", my_ip, port);
    println!("-------------------------------------------");

    // 6. Chạy Server và Tự động mở trình duyệt
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            let lan_url = format!("http://{}:{}", my_ip, port);
            open_browser(&lan_url);
            axum::serve(listener, app).await.unwrap();
        }
        Err(e) => {
            println!("❌ Lỗi Port {}: {}", port, e);
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
