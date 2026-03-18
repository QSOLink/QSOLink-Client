use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct CredentialStore {
    config_path: PathBuf,
}

impl CredentialStore {
    pub fn new() -> Self {
        let config_dir = dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qsolog");

        std::fs::create_dir_all(&config_dir).ok();

        Self {
            config_path: config_dir.join("credentials.enc"),
        }
    }

    fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        hasher.update(b"qsolog-credential-key-v1");

        let mut key = [0u8; 32];
        key.copy_from_slice(&hasher.finalize());
        key
    }

    pub fn save_credentials(&self, username: &str, password: &str) -> Result<(), String> {
        let mut salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt);

        let key = Self::derive_key(password, &salt);

        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher init failed: {}", e))?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher
            .encrypt(nonce, password.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + encrypted.len());
        output.extend_from_slice(&salt);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&encrypted);

        let encoded = BASE64.encode(&output);

        let config_content = format!("username={}\ncredentials={}", username, encoded);

        std::fs::write(&self.config_path, config_content)
            .map_err(|e| format!("Failed to write credentials: {}", e))?;

        log::info!("Credentials saved securely");
        Ok(())
    }

    pub fn load_credentials(&self) -> Option<(String, String)> {
        let content = std::fs::read_to_string(&self.config_path).ok()?;

        let mut username = String::new();
        let mut encoded = String::new();

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "username" => username = value.to_string(),
                    "credentials" => encoded = value.to_string(),
                    _ => {}
                }
            }
        }

        if username.is_empty() || encoded.is_empty() {
            return None;
        }

        let data = BASE64.decode(&encoded).ok()?;

        if data.len() < SALT_SIZE + NONCE_SIZE {
            return None;
        }

        let salt = &data[..SALT_SIZE];
        let nonce_bytes = &data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
        let encrypted = &data[SALT_SIZE + NONCE_SIZE..];

        let key = Self::derive_key(&username, salt);
        let cipher = Aes256Gcm::new_from_slice(&key).ok()?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = cipher.decrypt(nonce, encrypted).ok()?;
        let password = String::from_utf8(decrypted).ok()?;

        log::info!("Credentials loaded successfully");
        Some((username, password))
    }

    pub fn delete_credentials(&self) -> Result<(), String> {
        if self.config_path.exists() {
            std::fs::remove_file(&self.config_path)
                .map_err(|e| format!("Failed to delete credentials: {}", e))?;
            log::info!("Credentials deleted");
        }
        Ok(())
    }

    pub fn has_credentials(&self) -> bool {
        self.config_path.exists()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sanitize_for_log(input: &str) -> String {
    if input.len() <= 4 {
        return "****".to_string();
    }
    let visible = std::cmp::min(4, input.len() / 4);
    format!("{}****", &input[..visible])
}
