use std::io;
use tokio::fs;

pub async fn clean_uploads() -> Result<(), io::Error> {
    let upload_dir = "uploads";

    // Read all entries in the uploads directory
    let mut entries = fs::read_dir(upload_dir).await?;

    // Remove each file
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(&path).await?;
            println!("🗑️ Removed: {}", path.display());
        }
    }

    println!("✅ Uploads directory cleaned");
    Ok(())
}
