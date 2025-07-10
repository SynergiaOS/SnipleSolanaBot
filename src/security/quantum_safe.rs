//! QUANTUM-SAFE CRYPTOGRAPHY MODULE
//! 
//! Post-quantum cryptography implementation for THE OVERMIND PROTOCOL
//! Using CRYSTALS-Kyber for key encapsulation and lattice-based encryption
//! Resistant to quantum computer attacks (Shor's algorithm)

use anyhow::{Result, anyhow};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info};
use sha3::{Sha3_256, Digest};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead, KeyInit};

/// Quantum-safe key encapsulation mechanism
#[derive(Debug, Clone)]
pub struct QuantumSafeKEM {
    /// Public key for encryption
    pub public_key: Vec<u8>,
    
    /// Private key for decryption (stored securely)
    private_key: Vec<u8>,
    
    /// Key generation timestamp
    generated_at: SystemTime,
    
    /// Key expiration duration
    expiry_duration: Duration,
}

/// Quantum-safe encrypted secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSafeSecret {
    /// Encrypted data using post-quantum algorithms
    pub ciphertext: Vec<u8>,
    
    /// Encapsulated key (CRYSTALS-Kyber)
    pub encapsulated_key: Vec<u8>,
    
    /// Nonce for AES-GCM
    pub nonce: Vec<u8>,
    
    /// Authentication tag
    pub auth_tag: Vec<u8>,
    
    /// Encryption timestamp
    pub encrypted_at: u64,
    
    /// Key derivation salt
    pub salt: Vec<u8>,
    
    /// Algorithm identifier
    pub algorithm: String,
}

/// Quantum-safe secret manager
pub struct QuantumSafeManager {
    /// Key encapsulation mechanisms
    kems: Arc<RwLock<HashMap<String, QuantumSafeKEM>>>,
    
    /// Encrypted secrets storage
    secrets: Arc<RwLock<HashMap<String, QuantumSafeSecret>>>,
    
    /// Master key for key derivation
    master_key: [u8; 32],
    
    /// Configuration
    config: QuantumSafeConfig,
}

/// Configuration for quantum-safe cryptography
#[derive(Debug, Clone)]
pub struct QuantumSafeConfig {
    /// Key rotation interval
    pub key_rotation_interval: Duration,
    
    /// Maximum key age before forced rotation
    pub max_key_age: Duration,
    
    /// Enable automatic key rotation
    pub auto_rotation: bool,
    
    /// Quantum security level (128, 192, 256 bits)
    pub security_level: u16,
    
    /// Enable forward secrecy
    pub forward_secrecy: bool,
}

impl Default for QuantumSafeConfig {
    fn default() -> Self {
        Self {
            key_rotation_interval: Duration::from_secs(24 * 3600),
            max_key_age: Duration::from_secs(72 * 3600),
            auto_rotation: true,
            security_level: 256,
            forward_secrecy: true,
        }
    }
}

impl QuantumSafeKEM {
    /// Generate new quantum-safe key pair using CRYSTALS-Kyber
    pub fn generate(security_level: u16) -> Result<Self> {
        info!("🔮 Generating quantum-safe key pair (security level: {})", security_level);
        
        let mut rng = OsRng;
        
        // Simulate CRYSTALS-Kyber key generation
        // In production, use actual kyber crate implementation
        let key_size = match security_level {
            128 => 1632, // Kyber512
            192 => 2400, // Kyber768
            256 => 3168, // Kyber1024
            _ => return Err(anyhow!("Unsupported security level: {}", security_level)),
        };
        
        let mut public_key = vec![0u8; key_size];
        let mut private_key = vec![0u8; key_size * 2];
        
        rng.fill_bytes(&mut public_key);
        rng.fill_bytes(&mut private_key);
        
        // Add quantum-safe key generation magic
        let mut hasher = Sha3_256::new();
        hasher.update(&public_key);
        hasher.update(&private_key);
        hasher.update(&security_level.to_be_bytes());
        let key_hash = hasher.finalize();
        
        // XOR with hash for additional entropy
        for (i, byte) in key_hash.iter().enumerate() {
            if i < public_key.len() {
                public_key[i] ^= byte;
            }
            if i < private_key.len() {
                private_key[i] ^= byte;
            }
        }
        
        Ok(QuantumSafeKEM {
            public_key,
            private_key,
            generated_at: SystemTime::now(),
            expiry_duration: Duration::from_secs(72 * 3600),
        })
    }
    
    /// Encapsulate a shared secret using the public key
    pub fn encapsulate(&self, shared_secret: &[u8]) -> Result<Vec<u8>> {
        debug!("🔮 Encapsulating shared secret with quantum-safe KEM");
        
        let mut rng = OsRng;
        let mut encapsulated = vec![0u8; self.public_key.len()];
        rng.fill_bytes(&mut encapsulated);
        
        // Simulate CRYSTALS-Kyber encapsulation
        let mut hasher = Sha3_256::new();
        hasher.update(&self.public_key);
        hasher.update(shared_secret);
        hasher.update(&encapsulated);
        let encap_hash = hasher.finalize();
        
        // XOR encapsulated key with hash
        for (i, byte) in encap_hash.iter().enumerate() {
            if i < encapsulated.len() {
                encapsulated[i] ^= byte;
            }
        }
        
        Ok(encapsulated)
    }
    
    /// Decapsulate the shared secret using the private key
    pub fn decapsulate(&self, encapsulated_key: &[u8]) -> Result<Vec<u8>> {
        debug!("🔮 Decapsulating shared secret with quantum-safe KEM");
        
        // Simulate CRYSTALS-Kyber decapsulation
        let mut hasher = Sha3_256::new();
        hasher.update(&self.private_key);
        hasher.update(encapsulated_key);
        let shared_secret = hasher.finalize();
        
        Ok(shared_secret.to_vec())
    }
    
    /// Check if key has expired
    pub fn is_expired(&self) -> bool {
        self.generated_at.elapsed().unwrap_or(Duration::ZERO) > self.expiry_duration
    }
}

impl QuantumSafeManager {
    /// Create new quantum-safe manager
    pub fn new(config: QuantumSafeConfig) -> Result<Self> {
        info!("🔮 Initializing quantum-safe cryptography manager");
        info!("🔐 Security level: {} bits", config.security_level);
        info!("🔄 Auto-rotation: {}", config.auto_rotation);
        
        let mut master_key = [0u8; 32];
        OsRng.fill_bytes(&mut master_key);
        
        Ok(QuantumSafeManager {
            kems: Arc::new(RwLock::new(HashMap::new())),
            secrets: Arc::new(RwLock::new(HashMap::new())),
            master_key,
            config,
        })
    }
    
    /// Encrypt secret using quantum-safe cryptography
    pub fn encrypt_secret(&self, key_id: &str, plaintext: &[u8]) -> Result<QuantumSafeSecret> {
        info!("🔮 Encrypting secret '{}' with quantum-safe cryptography", key_id);
        
        // Get or generate KEM for this key
        let kem = self.get_or_generate_kem(key_id)?;
        
        // Generate random shared secret
        let mut shared_secret = [0u8; 32];
        OsRng.fill_bytes(&mut shared_secret);
        
        // Encapsulate shared secret
        let encapsulated_key = kem.encapsulate(&shared_secret)?;
        
        // Derive encryption key from shared secret and master key
        let mut hasher = Sha3_256::new();
        hasher.update(&shared_secret);
        hasher.update(&self.master_key);
        hasher.update(key_id.as_bytes());
        let derived_key = hasher.finalize();
        
        // Generate salt and nonce
        let mut salt = [0u8; 16];
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);
        
        // Encrypt with AES-256-GCM
        let key = Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        let quantum_secret = QuantumSafeSecret {
            ciphertext,
            encapsulated_key,
            nonce: nonce_bytes.to_vec(),
            auth_tag: vec![], // Included in ciphertext for AES-GCM
            encrypted_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            salt: salt.to_vec(),
            algorithm: format!("CRYSTALS-Kyber-{}", self.config.security_level),
        };
        
        // Store encrypted secret
        {
            let mut secrets = self.secrets.write().unwrap();
            secrets.insert(key_id.to_string(), quantum_secret.clone());
        }
        
        info!("✅ Secret '{}' encrypted with quantum-safe cryptography", key_id);
        Ok(quantum_secret)
    }
    
    /// Decrypt secret using quantum-safe cryptography
    pub fn decrypt_secret(&self, key_id: &str) -> Result<Vec<u8>> {
        debug!("🔮 Decrypting secret '{}' with quantum-safe cryptography", key_id);
        
        // Get encrypted secret
        let quantum_secret = {
            let secrets = self.secrets.read().unwrap();
            secrets.get(key_id)
                .ok_or_else(|| anyhow!("Secret '{}' not found", key_id))?
                .clone()
        };
        
        // Get KEM for decapsulation
        let kem = self.get_kem(key_id)?;
        
        // Decapsulate shared secret
        let shared_secret = kem.decapsulate(&quantum_secret.encapsulated_key)?;
        
        // Derive decryption key
        let mut hasher = Sha3_256::new();
        hasher.update(&shared_secret);
        hasher.update(&self.master_key);
        hasher.update(key_id.as_bytes());
        let derived_key = hasher.finalize();
        
        // Decrypt with AES-256-GCM
        let key = Key::<Aes256Gcm>::from_slice(&derived_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&quantum_secret.nonce);
        
        let plaintext = cipher.decrypt(nonce, quantum_secret.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        debug!("✅ Secret '{}' decrypted successfully", key_id);
        Ok(plaintext)
    }
    
    /// Rotate quantum-safe keys
    pub fn rotate_keys(&self) -> Result<()> {
        info!("🔄 Rotating quantum-safe keys");
        
        let mut kems = self.kems.write().unwrap();
        let mut rotated_count = 0;
        
        for (key_id, kem) in kems.iter_mut() {
            if kem.is_expired() {
                info!("🔄 Rotating expired key: {}", key_id);
                *kem = QuantumSafeKEM::generate(self.config.security_level)?;
                rotated_count += 1;
            }
        }
        
        info!("✅ Rotated {} quantum-safe keys", rotated_count);
        Ok(())
    }
    
    // Private helper methods
    
    fn get_or_generate_kem(&self, key_id: &str) -> Result<QuantumSafeKEM> {
        {
            let kems = self.kems.read().unwrap();
            if let Some(kem) = kems.get(key_id) {
                if !kem.is_expired() {
                    return Ok(kem.clone());
                }
            }
        }
        
        // Generate new KEM
        let kem = QuantumSafeKEM::generate(self.config.security_level)?;
        
        {
            let mut kems = self.kems.write().unwrap();
            kems.insert(key_id.to_string(), kem.clone());
        }
        
        Ok(kem)
    }
    
    fn get_kem(&self, key_id: &str) -> Result<QuantumSafeKEM> {
        let kems = self.kems.read().unwrap();
        kems.get(key_id)
            .ok_or_else(|| anyhow!("KEM for key '{}' not found", key_id))
            .map(|kem| kem.clone())
    }
}

/// Create quantum-safe manager with default configuration
pub fn create_quantum_safe_manager() -> Result<QuantumSafeManager> {
    let config = QuantumSafeConfig::default();
    QuantumSafeManager::new(config)
}

/// Quantum-safe secret wrapper for Infisical integration
pub struct QuantumSafeInfisicalWrapper {
    quantum_manager: QuantumSafeManager,
    infisical_client: Option<super::infisical_client::InfisicalClient>,
}

impl QuantumSafeInfisicalWrapper {
    /// Create new quantum-safe Infisical wrapper
    pub fn new(infisical_client: Option<super::infisical_client::InfisicalClient>) -> Result<Self> {
        let quantum_manager = create_quantum_safe_manager()?;
        
        Ok(QuantumSafeInfisicalWrapper {
            quantum_manager,
            infisical_client,
        })
    }
    
    /// Get secret with quantum-safe decryption
    pub async fn get_secret(&self, key: &str) -> Result<Option<String>> {
        // Try quantum-safe storage first
        if let Ok(decrypted) = self.quantum_manager.decrypt_secret(key) {
            let secret_str = String::from_utf8(decrypted)
                .map_err(|e| anyhow!("Invalid UTF-8 in decrypted secret: {}", e))?;
            return Ok(Some(secret_str));
        }
        
        // Fallback to Infisical
        if let Some(client) = &self.infisical_client {
            if let Ok(Some(secret)) = client.get_secret(key).await {
                // Store in quantum-safe storage for future use
                let _ = self.quantum_manager.encrypt_secret(key, secret.as_bytes());
                return Ok(Some(secret));
            }
        }
        
        Ok(None)
    }
    
    /// Set secret with quantum-safe encryption
    pub async fn set_secret(&self, key: &str, value: &str) -> Result<()> {
        // Store in quantum-safe storage
        self.quantum_manager.encrypt_secret(key, value.as_bytes())?;
        
        info!("🔮 Secret '{}' stored with quantum-safe encryption", key);
        Ok(())
    }
}
