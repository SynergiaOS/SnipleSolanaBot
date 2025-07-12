use pqcrypto_mlkem::{keypair, encapsulate, decapsulate};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct QuantumSecureVault {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

impl QuantumSecureVault {
    pub fn new() -> Self {
        let (pk, sk) = keypair();
        Self {
            public_key: pk.to_vec(),
            private_key: sk.to_vec(),
        }
    }
    
    pub fn encrypt_secret(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        // ML-KEM (wcześniej CRYSTALS-Kyber) dla post-quantum key exchange
        let (ciphertext, shared_secret) = encapsulate(&self.public_key);
        
        // AES-GCM dla szyfrowania symetrycznego
        let key = Key::from_slice(&shared_secret);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&[0u8; 12]); // W produkcji użyj losowego nonce
        
        let encrypted = cipher.encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
            
        Ok((ciphertext, encrypted))
    }
    
    pub fn decrypt_secret(&self, ciphertext: &[u8], encrypted: &[u8]) -> Result<Vec<u8>> {
        // Odzyskaj shared secret
        let shared_secret = decapsulate(ciphertext, &self.private_key);
        
        // Deszyfruj dane
        let key = Key::from_slice(&shared_secret);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&[0u8; 12]); // Musi być taki sam jak przy szyfrowaniu
        
        let decrypted = cipher.decrypt(nonce, encrypted)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
            
        Ok(decrypted)
    }
}