use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 16;

pub struct CredentialStore {
    config_path: PathBuf,
    lotw_config_path: PathBuf,
    station_profile_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct CredentialFile {
    qrz_username: Option<String>,
    qrz_credentials: Option<String>,
    lotw_username: Option<String>,
    lotw_credentials: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LotwSyncState {
    pub last_qsl_timestamp: Option<String>,
    pub last_qsorx_timestamp: Option<String>,
    pub last_sync: Option<String>,
}

impl CredentialStore {
    pub fn new() -> Self {
        let config_dir = dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qsolog");

        std::fs::create_dir_all(&config_dir).ok();

        Self {
            config_path: config_dir.join("credentials.enc"),
            lotw_config_path: config_dir.join("lotw_sync_state.toml"),
            station_profile_path: config_dir.join("station_profile.toml"),
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

    fn encrypt_value(&self, value: &str, key_password: &str) -> Result<String, String> {
        let salt = [0u8; SALT_SIZE];
        OsRng.fill_bytes(&mut salt.clone());
        let salt_owned = salt;

        let key = Self::derive_key(key_password, &salt_owned);

        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher init failed: {}", e))?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher
            .encrypt(nonce, value.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let mut output = Vec::with_capacity(SALT_SIZE + NONCE_SIZE + encrypted.len());
        output.extend_from_slice(&salt_owned);
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&encrypted);

        Ok(BASE64.encode(&output))
    }

    fn decrypt_value(&self, encoded: &str, key_password: &str) -> Result<String, String> {
        let data = BASE64
            .decode(encoded)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < SALT_SIZE + NONCE_SIZE {
            return Err("Invalid encrypted data".to_string());
        }

        let salt = &data[..SALT_SIZE];
        let nonce_bytes = &data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
        let encrypted = &data[SALT_SIZE + NONCE_SIZE..];

        let key = Self::derive_key(key_password, salt);
        let cipher =
            Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher init failed: {}", e))?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        String::from_utf8(decrypted).map_err(|e| format!("UTF-8 decode failed: {}", e))
    }

    fn load_credential_file(&self) -> CredentialFile {
        let content = std::fs::read_to_string(&self.config_path).unwrap_or_default();
        let mut cf = CredentialFile::default();

        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "qrz_username" => cf.qrz_username = Some(value.to_string()),
                    "qrz_credentials" => cf.qrz_credentials = Some(value.to_string()),
                    "lotw_username" => cf.lotw_username = Some(value.to_string()),
                    "lotw_credentials" => cf.lotw_credentials = Some(value.to_string()),
                    _ => {}
                }
            }
        }
        cf
    }

    fn save_credential_file(&self, cf: &CredentialFile) -> Result<(), String> {
        let mut lines = Vec::new();
        if let Some(ref u) = cf.qrz_username {
            lines.push(format!("qrz_username={}", u));
        }
        if let Some(ref c) = cf.qrz_credentials {
            lines.push(format!("qrz_credentials={}", c));
        }
        if let Some(ref u) = cf.lotw_username {
            lines.push(format!("lotw_username={}", u));
        }
        if let Some(ref c) = cf.lotw_credentials {
            lines.push(format!("lotw_credentials={}", c));
        }

        let content = lines.join("\n");
        std::fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write credentials: {}", e))?;
        Ok(())
    }

    pub fn save_credentials(&self, username: &str, password: &str) -> Result<(), String> {
        let mut cf = self.load_credential_file();
        let encrypted = self.encrypt_value(password, username)?;
        cf.qrz_username = Some(username.to_string());
        cf.qrz_credentials = Some(encrypted);
        self.save_credential_file(&cf)?;
        log::info!("QRZ credentials saved securely");
        Ok(())
    }

    pub fn load_credentials(&self) -> Option<(String, String)> {
        let cf = self.load_credential_file();
        let username = cf.qrz_username?;
        let encrypted = cf.qrz_credentials?;
        let password = self.decrypt_value(&encrypted, &username).ok()?;
        log::info!("QRZ credentials loaded successfully");
        Some((username, password))
    }

    pub fn delete_credentials(&self) -> Result<(), String> {
        let mut cf = self.load_credential_file();
        cf.qrz_username = None;
        cf.qrz_credentials = None;
        if cf.qrz_username.is_none()
            && cf.qrz_credentials.is_none()
            && cf.lotw_username.is_none()
            && cf.lotw_credentials.is_none()
        {
            std::fs::remove_file(&self.config_path).ok();
        } else {
            self.save_credential_file(&cf)?;
        }
        log::info!("QRZ credentials deleted");
        Ok(())
    }

    pub fn has_credentials(&self) -> bool {
        let cf = self.load_credential_file();
        cf.qrz_username.is_some() && cf.qrz_credentials.is_some()
    }

    pub fn save_lotw_credentials(&self, username: &str, password: &str) -> Result<(), String> {
        let mut cf = self.load_credential_file();
        let encrypted = self.encrypt_value(password, username)?;
        cf.lotw_username = Some(username.to_string());
        cf.lotw_credentials = Some(encrypted);
        self.save_credential_file(&cf)?;
        log::info!("LoTW credentials saved securely");
        Ok(())
    }

    pub fn load_lotw_credentials(&self) -> Option<(String, String)> {
        let cf = self.load_credential_file();
        let username = cf.lotw_username?;
        let encrypted = cf.lotw_credentials?;
        let password = self.decrypt_value(&encrypted, &username).ok()?;
        log::info!("LoTW credentials loaded successfully");
        Some((username, password))
    }

    pub fn has_lotw_credentials(&self) -> bool {
        let cf = self.load_credential_file();
        cf.lotw_username.is_some() && cf.lotw_credentials.is_some()
    }

    pub fn delete_lotw_credentials(&self) -> Result<(), String> {
        let mut cf = self.load_credential_file();
        cf.lotw_username = None;
        cf.lotw_credentials = None;
        if cf.qrz_username.is_none()
            && cf.qrz_credentials.is_none()
            && cf.lotw_username.is_none()
            && cf.lotw_credentials.is_none()
        {
            std::fs::remove_file(&self.config_path).ok();
        } else {
            self.save_credential_file(&cf)?;
        }
        log::info!("LoTW credentials deleted");
        Ok(())
    }

    pub fn save_station_profile(
        &self,
        profile: &crate::models::StationProfile,
    ) -> Result<(), String> {
        let content = toml::to_string_pretty(profile)
            .map_err(|e| format!("Failed to serialize station profile: {}", e))?;
        std::fs::write(&self.station_profile_path, content)
            .map_err(|e| format!("Failed to write station profile: {}", e))?;
        log::info!("Station profile saved");
        Ok(())
    }

    pub fn load_station_profile(&self) -> Option<crate::models::StationProfile> {
        let content = std::fs::read_to_string(&self.station_profile_path).ok()?;
        let profile: crate::models::StationProfile = toml::from_str(&content).ok()?;
        log::info!("Station profile loaded");
        Some(profile)
    }

    pub fn has_station_profile(&self) -> bool {
        self.station_profile_path.exists()
    }

    pub fn save_lotw_sync_state(&self, state: &LotwSyncState) -> Result<(), String> {
        let content = toml::to_string_pretty(state)
            .map_err(|e| format!("Failed to serialize LoTW sync state: {}", e))?;
        std::fs::write(&self.lotw_config_path, content)
            .map_err(|e| format!("Failed to write LoTW sync state: {}", e))?;
        Ok(())
    }

    pub fn load_lotw_sync_state(&self) -> LotwSyncState {
        let content = match std::fs::read_to_string(&self.lotw_config_path) {
            Ok(c) => c,
            Err(_) => return LotwSyncState::default(),
        };
        toml::from_str(&content).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_for_log_short() {
        assert_eq!(sanitize_for_log("abc"), "****");
        assert_eq!(sanitize_for_log("abcd"), "****");
        assert_eq!(sanitize_for_log(""), "****");
    }

    #[test]
    fn test_sanitize_for_log_normal() {
        let result = sanitize_for_log("password123");
        assert!(result.ends_with("****"));
        assert!(result.starts_with("pa"));
        assert_eq!(result, "pa****");
    }

    #[test]
    fn test_sanitize_for_log_long() {
        let result = sanitize_for_log("superlongpassword");
        assert!(result.ends_with("****"));
        assert_eq!(&result[..4], "supe");
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = "testpassword";
        let salt = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let key1 = CredentialStore::derive_key(password, &salt);
        let key2 = CredentialStore::derive_key(password, &salt);

        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = "testpassword";
        let salt1 = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let salt2 = [2u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let key1 = CredentialStore::derive_key(password, &salt1);
        let key2 = CredentialStore::derive_key(password, &salt2);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let key1 = CredentialStore::derive_key("password1", &salt);
        let key2 = CredentialStore::derive_key("password2", &salt);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_lotw_sync_state_default() {
        let state = LotwSyncState::default();
        assert!(state.last_qsl_timestamp.is_none());
        assert!(state.last_qsorx_timestamp.is_none());
        assert!(state.last_sync.is_none());
    }
}
