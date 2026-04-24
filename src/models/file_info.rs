use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub url: String,
    pub size: u64,
}
