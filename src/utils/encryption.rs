use argon2::Argon2;
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, Key, KeyInit, Nonce};
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use tokio::fs as async_fs;
use tokio::io::AsyncReadExt;

#[allow(dead_code)]
pub fn derive_key(pin: &str, salt: &[u8]) -> Result<Key, Box<dyn std::error::Error>> {
    let mut key = Key::default();
    Argon2::default()
        .hash_password_into(pin.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Argon2 error: {}", e))?;
    Ok(key)
}

#[allow(dead_code)]
pub fn generate_nonce() -> Nonce {
    let mut nonce = Nonce::default();
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

#[allow(dead_code)]
pub fn encrypt_file(
    input_path: &str,
    output_path: &str,
    key: &Key,
) -> Result<Nonce, Box<dyn std::error::Error>> {
    let mut file_data = Vec::new();
    fs::File::open(input_path)?.read_to_end(&mut file_data)?;

    let cipher = ChaCha20Poly1305::new(key);
    let nonce = generate_nonce();
    let ciphertext = cipher
        .encrypt(&nonce, file_data.as_ref())
        .map_err(|e| format!("Encryption failed: {:?}", e))?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&nonce)?;
    output_file.write_all(&ciphertext)?;

    Ok(nonce)
}

#[allow(dead_code)]
pub async fn decrypt_file(
    input_path: &str,
    output_path: &str,
    key: &Key,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut input_file = async_fs::File::open(input_path).await?;
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data).await?;

    if encrypted_data.len() < 12 {
        return Err("Invalid encrypted file format".into());
    }

    let nonce = Nonce::from_slice(&encrypted_data[0..12]);
    let ciphertext = &encrypted_data[12..];

    let cipher = ChaCha20Poly1305::new(key);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    async_fs::write(output_path, &plaintext).await?;
    Ok(())
}

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}