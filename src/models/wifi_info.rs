use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct WifiInfo {
    pub ssid: String,
    pub signal: String,
    pub channel: String,
}
