// THE OVERMIND PROTOCOL - Vault Module
// Secure credential management with AES-256 encryption and environment fallback

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use anyhow::{anyhow, Result};
// use generic_array::GenericArray; // Not needed for current implementation
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    pub master_key_env: String,
    pub secrets_dir: String,
    pub encryption_enabled: bool,
    pub fallback_to_env: bool,
    pub key_rotation_enabled: bool,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            master_key_env: "VAULT_MASTER_KEY".to_string(),
            secrets_dir: "secrets".to_string(),
            encryption_enabled: true,
            fallback_to_env: true,
            key_rotation_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub key_name: String,
    pub encrypted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
}

pub struct Vault {
    master_key: [u8; 32],
    config: VaultConfig,
    metadata_cache: HashMap<String, SecretMetadata>,
}

impl Vault {
    /// Create new vault instance with master key from environment
    pub fn new() -> Result<Self> {
        Self::with_config(VaultConfig::default())
    }

    /// Create new vault instance with custom configuration
    pub fn with_config(config: VaultConfig) -> Result<Self> {
        info!("🔐 Initializing Vault with AES-256 encryption");
        
        let key_env = std::env::var(&config.master_key_env)
            .map_err(|_| anyhow!("Master key environment variable '{}' not found", config.master_key_env))?;
        
        if key_env.len() < 32 {
            return Err(anyhow!("Master key must be at least 32 bytes, got {}", key_env.len()));
        }
        
        let mut master = [0u8; 32];
        master.copy_from_slice(&key_env.as_bytes()[..32]);
        
        // Create secrets directory if it doesn't exist
        if config.encryption_enabled {
            fs::create_dir_all(&config.secrets_dir)
                .map_err(|e| anyhow!("Failed to create secrets directory: {}", e))?;
        }

        let vault = Self {
            master_key: master,
            config,
            metadata_cache: HashMap::new(),
        };

        info!("✅ Vault initialized successfully");
        Ok(vault)
    }

    /// Encrypt plaintext using AES-256-GCM
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        if !self.config.encryption_enabled {
            return Ok(plaintext.as_bytes().to_vec());
        }

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| anyhow!("Failed to initialize cipher: {}", e))?;

        // Generate random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // Encrypt the plaintext
        let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        debug!("🔒 Encrypted {} bytes of data", plaintext.len());
        Ok(result)
    }

    /// Decrypt ciphertext using AES-256-GCM
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<String> {
        if !self.config.encryption_enabled {
            return String::from_utf8(ciphertext.to_vec())
                .map_err(|e| anyhow!("UTF-8 decode error: {}", e));
        }

        if ciphertext.len() < 12 {
            return Err(anyhow!("Invalid ciphertext: too short"));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| anyhow!("Failed to initialize cipher: {}", e))?;

        // Extract nonce and encrypted data
        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt the data
        let plaintext = cipher.decrypt(nonce, encrypted)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("UTF-8 decode error: {}", e))
    }

    /// Store encrypted secret to file
    pub fn store_secret(&mut self, key: &str, value: &str) -> Result<()> {
        info!("💾 Storing secret: {}", key);
        
        if !self.config.encryption_enabled {
            warn!("⚠️ Encryption disabled, storing secret in plaintext");
        }

        let encrypted_data = self.encrypt(value)?;
        let file_path = Path::new(&self.config.secrets_dir).join(format!("{}.enc", key));
        
        fs::write(&file_path, encrypted_data)
            .map_err(|e| anyhow!("Failed to write secret file: {}", e))?;

        // Update metadata
        let metadata = SecretMetadata {
            key_name: key.to_string(),
            encrypted: self.config.encryption_enabled,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 0,
        };
        
        self.metadata_cache.insert(key.to_string(), metadata);
        self.save_metadata()?;

        info!("✅ Secret '{}' stored successfully", key);
        Ok(())
    }

    /// Get secret with environment variable fallback
    pub fn get_env_or_vault(&mut self, key: &str) -> Result<String> {
        debug!("🔍 Retrieving secret: {}", key);
        
        // Priority 1: Environment variables
        if let Ok(env_value) = std::env::var(key) {
            debug!("📝 Found secret '{}' in environment", key);
            return Ok(env_value);
        }
        
        // Priority 2: Vault encrypted secrets
        if self.config.encryption_enabled {
            match self.get_vault_secret(key) {
                Ok(value) => {
                    debug!("🔓 Retrieved secret '{}' from vault", key);
                    return Ok(value);
                }
                Err(e) => {
                    if self.config.fallback_to_env {
                        warn!("⚠️ Failed to get secret '{}' from vault: {}", key, e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow!("Secret '{}' not found in environment or vault", key))
    }

    /// Get secret from vault file
    fn get_vault_secret(&mut self, key: &str) -> Result<String> {
        let file_path = Path::new(&self.config.secrets_dir).join(format!("{}.enc", key));
        
        if !file_path.exists() {
            return Err(anyhow!("Secret file not found: {}", file_path.display()));
        }

        let encrypted_data = fs::read(&file_path)
            .map_err(|e| anyhow!("Failed to read secret file: {}", e))?;
            
        let decrypted = self.decrypt(&encrypted_data)?;
        
        // Update access metadata
        if let Some(metadata) = self.metadata_cache.get_mut(key) {
            metadata.last_accessed = chrono::Utc::now();
            metadata.access_count += 1;
        }

        Ok(decrypted)
    }

    /// List all available secrets
    pub fn list_secrets(&self) -> Result<Vec<String>> {
        let mut secrets = Vec::new();
        
        // Add environment variables (common secret patterns)
        for (key, _) in std::env::vars() {
            if key.contains("KEY") || key.contains("SECRET") || key.contains("TOKEN") {
                secrets.push(format!("env:{}", key));
            }
        }
        
        // Add vault secrets
        if self.config.encryption_enabled && Path::new(&self.config.secrets_dir).exists() {
            let entries = fs::read_dir(&self.config.secrets_dir)
                .map_err(|e| anyhow!("Failed to read secrets directory: {}", e))?;
                
            for entry in entries {
                let entry = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.ends_with(".enc") {
                            let secret_name = name_str.trim_end_matches(".enc");
                            secrets.push(format!("vault:{}", secret_name));
                        }
                    }
                }
            }
        }
        
        Ok(secrets)
    }

    /// Get secret metadata
    pub fn get_secret_metadata(&self, key: &str) -> Option<&SecretMetadata> {
        self.metadata_cache.get(key)
    }

    /// Save metadata to file
    fn save_metadata(&self) -> Result<()> {
        if !self.config.encryption_enabled {
            return Ok(());
        }

        let metadata_path = Path::new(&self.config.secrets_dir).join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.metadata_cache)
            .map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;
        
        fs::write(metadata_path, metadata_json)
            .map_err(|e| anyhow!("Failed to write metadata: {}", e))?;
        
        Ok(())
    }

    /// Load metadata from file
    pub fn load_metadata(&mut self) -> Result<()> {
        if !self.config.encryption_enabled {
            return Ok(());
        }

        let metadata_path = Path::new(&self.config.secrets_dir).join("metadata.json");
        
        if !metadata_path.exists() {
            return Ok(()); // No metadata file yet
        }

        let metadata_json = fs::read_to_string(metadata_path)
            .map_err(|e| anyhow!("Failed to read metadata: {}", e))?;
        
        self.metadata_cache = serde_json::from_str(&metadata_json)
            .map_err(|e| anyhow!("Failed to parse metadata: {}", e))?;
        
        debug!("📊 Loaded metadata for {} secrets", self.metadata_cache.len());
        Ok(())
    }

    /// Rotate master key (advanced feature)
    pub fn rotate_master_key(&mut self, new_key: &str) -> Result<()> {
        if !self.config.key_rotation_enabled {
            return Err(anyhow!("Key rotation is disabled"));
        }

        info!("🔄 Starting master key rotation");
        
        // This would involve re-encrypting all secrets with the new key
        // Implementation would be more complex in production
        warn!("⚠️ Key rotation not fully implemented - use with caution");
        
        Ok(())
    }

    /// Validate vault integrity
    pub fn validate_integrity(&mut self) -> Result<bool> {
        info!("🔍 Validating vault integrity");
        
        let secrets = self.list_secrets()?;
        let mut valid_count = 0;
        let mut total_count = 0;
        
        for secret in secrets {
            if secret.starts_with("vault:") {
                total_count += 1;
                let key = secret.trim_start_matches("vault:");
                
                match self.get_vault_secret(key) {
                    Ok(_) => valid_count += 1,
                    Err(e) => {
                        error!("❌ Failed to decrypt secret '{}': {}", key, e);
                    }
                }
            }
        }
        
        let integrity_ok = valid_count == total_count;
        
        if integrity_ok {
            info!("✅ Vault integrity check passed: {}/{} secrets valid", valid_count, total_count);
        } else {
            error!("❌ Vault integrity check failed: {}/{} secrets valid", valid_count, total_count);
        }
        
        Ok(integrity_ok)
    }
}
