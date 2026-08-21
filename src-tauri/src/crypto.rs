use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use crate::error::AppError;

/// 加密密钥长度（字节）
const KEY_LEN: usize = 32;
/// Nonce 长度（字节）
const NONCE_LEN: usize = 12;

/// 从应用数据目录派生加密密钥
/// 使用 machine-id + app-id 作为密钥来源
fn derive_key(app_data_dir: &std::path::Path) -> Result<[u8; KEY_LEN], AppError> {
    let key_file = app_data_dir.join(".encryption_key");

    if key_file.exists() {
        let data = std::fs::read(&key_file)
            .map_err(|e| AppError::Internal(format!("Failed to read encryption key: {}", e)))?;
        if data.len() == KEY_LEN {
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(&data);
            return Ok(key);
        }
    }

    // 生成新的密钥
    let mut key = [0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);

    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create app data dir: {}", e)))?;
    std::fs::write(&key_file, &key)
        .map_err(|e| AppError::Internal(format!("Failed to write encryption key: {}", e)))?;

    Ok(key)
}

/// 加密 API Key
/// 返回格式：base64(nonce + ciphertext)
pub fn encrypt(plain_key: &str, app_data_dir: &std::path::Path) -> Result<String, AppError> {
    let key = derive_key(app_data_dir)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Internal(format!("Failed to create cipher: {}", e)))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plain_key.as_bytes())
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

    // nonce + ciphertext 拼接后 base64 编码
    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// 解密 API Key
pub fn decrypt(encrypted: &str, app_data_dir: &std::path::Path) -> Result<String, AppError> {
    let key = derive_key(app_data_dir)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| AppError::Internal(format!("Failed to create cipher: {}", e)))?;

    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| AppError::Internal(format!("Failed to decode encrypted key: {}", e)))?;

    if combined.len() < NONCE_LEN {
        return Err(AppError::Internal("Invalid encrypted key format".to_string()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("Invalid UTF-8 in decrypted key: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let temp_dir = std::env::temp_dir().join("linden_test_crypto");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let plain = "sk-test-1234567890abcdef";
        let encrypted = encrypt(plain, &temp_dir).unwrap();
        assert_ne!(encrypted, plain);

        let decrypted = decrypt(&encrypted, &temp_dir).unwrap();
        assert_eq!(decrypted, plain);

        // 清理
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
