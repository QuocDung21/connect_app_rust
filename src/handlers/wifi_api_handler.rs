use axum::{Json, response::IntoResponse};

use crate::models::wifi_info::WifiInfo;

pub async fn wifi_api_handler() -> impl IntoResponse {
    match wifiscanner::scan() {
        Ok(networks) => {
            let info: Vec<WifiInfo> = networks
                .into_iter()
                .take(10)
                .map(|n| WifiInfo {
                    ssid: n.ssid,
                    signal: n.signal_level,
                    channel: n.channel,
                })
                .collect();
            Json(info).into_response()
        }
        Err(e) => {
            eprintln!("⚠️ Wifi Scan Failed: {:?}", e);
            Json(vec![WifiInfo {
                ssid: "Không tìm thấy thiết bị quét".into(),
                signal: "0".into(),
                channel: "0".into(),
            }])
            .into_response()
        }
    }
}
