use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct FileInfo {
    pub name: String,
    pub url: String,
    pub size: u64,
}
