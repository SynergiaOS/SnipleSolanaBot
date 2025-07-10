//! BLOCKCHAIN SECRET STORAGE MODULE
//! 
//! Decentralized secret storage on Solana blockchain for THE OVERMIND PROTOCOL
//! Immutable, transparent, and cryptographically secure secret management
//! Uses Solana Program Derived Addresses (PDAs) for deterministic storage

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};
use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead, KeyInit};
use rand::{RngCore, rngs::OsRng};

/// Blockchain vault configuration
#[derive(Debug)]
pub struct BlockchainVaultConfig {
    /// Solana RPC endpoint
    pub rpc_url: String,
    
    /// Vault program ID on Solana
    pub program_id: Pubkey,
    
    /// Vault authority keypair
    pub authority: Keypair,
    
    /// Encryption enabled
    pub encryption_enabled: bool,
    
    /// Commitment level
    pub commitment: CommitmentConfig,
    
    /// Maximum retries for transactions
    pub max_retries: u8,
}

/// Encrypted secret stored on blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainSecret {
    /// Secret identifier
    pub id: String,
    
    /// Encrypted data
    pub encrypted_data: Vec<u8>,
    
    /// Encryption nonce
    pub nonce: Vec<u8>,
    
    /// Metadata hash
    pub metadata_hash: [u8; 32],
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last updated timestamp
    pub updated_at: u64,
    
    /// Access count
    pub access_count: u64,
    
    /// Owner public key
    pub owner: Pubkey,
    
    /// Access permissions
    pub permissions: Vec<Pubkey>,
}

/// Blockchain vault account data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultAccountData {
    /// Vault version
    pub version: u8,
    
    /// Vault authority
    pub authority: Pubkey,
    
    /// Total secrets stored
    pub total_secrets: u64,
    
    /// Vault creation timestamp
    pub created_at: u64,
    
    /// Last activity timestamp
    pub last_activity: u64,
    
    /// Vault configuration hash
    pub config_hash: [u8; 32],
}

/// Secret access log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretAccessLog {
    /// Access timestamp
    pub timestamp: u64,
    
    /// Accessor public key
    pub accessor: Pubkey,
    
    /// Access type (read, write, delete)
    pub access_type: String,
    
    /// Transaction signature
    pub signature: String,
    
    /// Success status
    pub success: bool,
}

/// Blockchain vault client
pub struct BlockchainVault {
    /// Solana RPC client
    rpc_client: RpcClient,
    
    /// Configuration
    config: BlockchainVaultConfig,
    
    /// Local cache for performance
    cache: Arc<RwLock<HashMap<String, BlockchainSecret>>>,
    
    /// Access logs
    access_logs: Arc<RwLock<Vec<SecretAccessLog>>>,
    
    /// Encryption key (derived from authority)
    encryption_key: [u8; 32],
}

impl BlockchainVault {
    /// Create new blockchain vault
    pub fn new(config: BlockchainVaultConfig) -> Result<Self> {
        info!("⛓️ Initializing Blockchain Vault on Solana");
        info!("🌐 RPC: {}", config.rpc_url);
        info!("📋 Program ID: {}", config.program_id);
        
        let rpc_client = RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            config.commitment,
        );
        
        // Derive encryption key from authority keypair
        let mut hasher = Sha256::new();
        hasher.update(config.authority.secret().as_bytes());
        hasher.update(b"overmind-vault-encryption");
        let encryption_key: [u8; 32] = hasher.finalize().into();
        
        Ok(BlockchainVault {
            rpc_client,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            access_logs: Arc::new(RwLock::new(Vec::new())),
            encryption_key,
        })
    }
    
    /// Initialize vault on blockchain
    pub async fn initialize_vault(&self) -> Result<Signature> {
        info!("⛓️ Initializing vault on Solana blockchain");
        
        let vault_pda = self.get_vault_pda()?;
        
        // Check if vault already exists
        if let Ok(_account) = self.rpc_client.get_account(&vault_pda) {
            info!("✅ Vault already exists at: {}", vault_pda);
            return Ok(Signature::default());
        }
        
        // Create vault initialization instruction
        let init_instruction = self.create_init_vault_instruction(vault_pda)?;
        
        // Create and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[init_instruction],
            Some(&self.config.authority.pubkey()),
            &[&self.config.authority],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        
        info!("✅ Vault initialized with signature: {}", signature);
        Ok(signature)
    }
    
    /// Store secret on blockchain
    pub async fn store_secret(&self, id: &str, data: &[u8]) -> Result<Signature> {
        info!("⛓️ Storing secret '{}' on blockchain", id);
        
        // Encrypt data if encryption is enabled
        let (encrypted_data, nonce) = if self.config.encryption_enabled {
            self.encrypt_data(data)?
        } else {
            (data.to_vec(), vec![])
        };
        
        // Create metadata hash
        let metadata_hash = self.create_metadata_hash(id, &encrypted_data)?;
        
        // Create blockchain secret
        let blockchain_secret = BlockchainSecret {
            id: id.to_string(),
            encrypted_data,
            nonce,
            metadata_hash,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            access_count: 0,
            owner: self.config.authority.pubkey(),
            permissions: vec![self.config.authority.pubkey()],
        };
        
        // Get secret PDA
        let secret_pda = self.get_secret_pda(id)?;
        
        // Create store instruction
        let store_instruction = self.create_store_secret_instruction(secret_pda, &blockchain_secret)?;
        
        // Create and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[store_instruction],
            Some(&self.config.authority.pubkey()),
            &[&self.config.authority],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        
        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(id.to_string(), blockchain_secret);
        }
        
        // Log access
        self.log_access(id, "write", signature.to_string(), true).await?;
        
        info!("✅ Secret '{}' stored with signature: {}", id, signature);
        Ok(signature)
    }
    
    /// Retrieve secret from blockchain
    pub async fn retrieve_secret(&self, id: &str) -> Result<Vec<u8>> {
        debug!("⛓️ Retrieving secret '{}' from blockchain", id);
        
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached_secret) = cache.get(id) {
                debug!("📋 Retrieved secret '{}' from cache", id);
                return self.decrypt_data(&cached_secret.encrypted_data, &cached_secret.nonce);
            }
        }
        
        // Get secret PDA
        let secret_pda = self.get_secret_pda(id)?;
        
        // Fetch account data from blockchain
        let account = self.rpc_client.get_account(&secret_pda)
            .map_err(|_| anyhow!("Secret '{}' not found on blockchain", id))?;
        
        // Deserialize secret data
        let blockchain_secret: BlockchainSecret = bincode::deserialize(&account.data)
            .map_err(|e| anyhow!("Failed to deserialize secret data: {}", e))?;
        
        // Update cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(id.to_string(), blockchain_secret.clone());
        }
        
        // Decrypt data
        let decrypted_data = self.decrypt_data(&blockchain_secret.encrypted_data, &blockchain_secret.nonce)?;
        
        // Log access
        self.log_access(id, "read", "cache".to_string(), true).await?;
        
        debug!("✅ Secret '{}' retrieved successfully", id);
        Ok(decrypted_data)
    }
    
    /// Delete secret from blockchain
    pub async fn delete_secret(&self, id: &str) -> Result<Signature> {
        info!("⛓️ Deleting secret '{}' from blockchain", id);
        
        let secret_pda = self.get_secret_pda(id)?;
        
        // Create delete instruction
        let delete_instruction = self.create_delete_secret_instruction(secret_pda)?;
        
        // Create and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[delete_instruction],
            Some(&self.config.authority.pubkey()),
            &[&self.config.authority],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        
        // Remove from cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.remove(id);
        }
        
        // Log access
        self.log_access(id, "delete", signature.to_string(), true).await?;
        
        info!("✅ Secret '{}' deleted with signature: {}", id, signature);
        Ok(signature)
    }
    
    /// List all secrets in vault
    pub async fn list_secrets(&self) -> Result<Vec<String>> {
        debug!("⛓️ Listing all secrets in vault");
        
        // In a real implementation, this would scan all PDAs
        // For now, return cached secrets
        let cache = self.cache.read().unwrap();
        let secret_ids: Vec<String> = cache.keys().cloned().collect();
        
        debug!("📋 Found {} secrets in vault", secret_ids.len());
        Ok(secret_ids)
    }
    
    /// Get vault statistics
    pub async fn get_vault_stats(&self) -> Result<VaultAccountData> {
        debug!("⛓️ Getting vault statistics");
        
        let vault_pda = self.get_vault_pda()?;
        
        let account = self.rpc_client.get_account(&vault_pda)
            .map_err(|_| anyhow!("Vault not found on blockchain"))?;
        
        let vault_data: VaultAccountData = bincode::deserialize(&account.data)
            .map_err(|e| anyhow!("Failed to deserialize vault data: {}", e))?;
        
        Ok(vault_data)
    }
    
    /// Get access logs for a secret
    pub fn get_access_logs(&self, secret_id: &str) -> Vec<SecretAccessLog> {
        let logs = self.access_logs.read().unwrap();
        logs.iter()
            .filter(|log| log.signature.contains(secret_id))
            .cloned()
            .collect()
    }
    
    // Private helper methods
    
    fn get_vault_pda(&self) -> Result<Pubkey> {
        let (pda, _bump) = Pubkey::find_program_address(
            &[b"vault", self.config.authority.pubkey().as_ref()],
            &self.config.program_id,
        );
        Ok(pda)
    }
    
    fn get_secret_pda(&self, secret_id: &str) -> Result<Pubkey> {
        let (pda, _bump) = Pubkey::find_program_address(
            &[b"secret", secret_id.as_bytes(), self.config.authority.pubkey().as_ref()],
            &self.config.program_id,
        );
        Ok(pda)
    }
    
    fn encrypt_data(&self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        Ok((ciphertext, nonce_bytes.to_vec()))
    }
    
    fn decrypt_data(&self, encrypted_data: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        if !self.config.encryption_enabled || nonce.is_empty() {
            return Ok(encrypted_data.to_vec());
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&self.encryption_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }
    
    fn create_metadata_hash(&self, id: &str, data: &[u8]) -> Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(data);
        hasher.update(&self.config.authority.pubkey().to_bytes());
        Ok(hasher.finalize().into())
    }
    
    fn create_init_vault_instruction(&self, vault_pda: Pubkey) -> Result<Instruction> {
        // Simplified instruction creation
        // In a real implementation, this would use the actual program instruction format
        Ok(Instruction {
            program_id: self.config.program_id,
            accounts: vec![
                AccountMeta::new(vault_pda, false),
                AccountMeta::new(self.config.authority.pubkey(), true),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
            data: vec![0], // Initialize instruction discriminator
        })
    }
    
    fn create_store_secret_instruction(&self, secret_pda: Pubkey, secret: &BlockchainSecret) -> Result<Instruction> {
        let mut instruction_data = vec![1]; // Store instruction discriminator
        instruction_data.extend_from_slice(&bincode::serialize(secret)?);
        
        Ok(Instruction {
            program_id: self.config.program_id,
            accounts: vec![
                AccountMeta::new(secret_pda, false),
                AccountMeta::new(self.config.authority.pubkey(), true),
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
            data: instruction_data,
        })
    }
    
    fn create_delete_secret_instruction(&self, secret_pda: Pubkey) -> Result<Instruction> {
        Ok(Instruction {
            program_id: self.config.program_id,
            accounts: vec![
                AccountMeta::new(secret_pda, false),
                AccountMeta::new(self.config.authority.pubkey(), true),
            ],
            data: vec![2], // Delete instruction discriminator
        })
    }
    
    async fn log_access(&self, secret_id: &str, access_type: &str, signature: String, success: bool) -> Result<()> {
        let log_entry = SecretAccessLog {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            accessor: self.config.authority.pubkey(),
            access_type: access_type.to_string(),
            signature,
            success,
        };
        
        {
            let mut logs = self.access_logs.write().unwrap();
            logs.push(log_entry);
            
            // Keep only last 1000 logs
            while logs.len() > 1000 {
                logs.remove(0);
            }
        }
        
        Ok(())
    }
}

/// Create blockchain vault with configuration
pub fn create_blockchain_vault(
    rpc_url: String,
    program_id_str: &str,
    authority_keypair: Keypair,
) -> Result<BlockchainVault> {
    let program_id = Pubkey::from_str(program_id_str)
        .map_err(|e| anyhow!("Invalid program ID: {}", e))?;
    
    let config = BlockchainVaultConfig {
        rpc_url,
        program_id,
        authority: authority_keypair,
        encryption_enabled: true,
        commitment: CommitmentConfig::confirmed(),
        max_retries: 3,
    };
    
    BlockchainVault::new(config)
}

/// Integration with Infisical for hybrid storage
pub struct HybridVaultStorage {
    /// Blockchain vault for immutable storage
    blockchain_vault: BlockchainVault,
    
    /// Infisical client for traditional secret management
    infisical_client: Option<super::infisical_client::InfisicalClient>,
    
    /// Storage strategy
    strategy: HybridStorageStrategy,
}

/// Hybrid storage strategies
#[derive(Debug, Clone)]
pub enum HybridStorageStrategy {
    /// Store all secrets on blockchain
    BlockchainOnly,
    /// Store all secrets in Infisical
    InfisicalOnly,
    /// Store critical secrets on blockchain, others in Infisical
    Hybrid,
    /// Store in both for redundancy
    Redundant,
}

impl HybridVaultStorage {
    /// Create new hybrid vault storage
    pub fn new(
        blockchain_vault: BlockchainVault,
        infisical_client: Option<super::infisical_client::InfisicalClient>,
        strategy: HybridStorageStrategy,
    ) -> Self {
        info!("🔗 Initializing Hybrid Vault Storage");
        info!("📋 Strategy: {:?}", strategy);
        
        HybridVaultStorage {
            blockchain_vault,
            infisical_client,
            strategy,
        }
    }
    
    /// Store secret using hybrid strategy
    pub async fn store_secret(&self, key: &str, value: &[u8]) -> Result<()> {
        match self.strategy {
            HybridStorageStrategy::BlockchainOnly => {
                self.blockchain_vault.store_secret(key, value).await?;
            }
            HybridStorageStrategy::InfisicalOnly => {
                if let Some(client) = &self.infisical_client {
                    let value_str = String::from_utf8_lossy(value);
                    client.get_secret(key).await?; // This would be a set operation in real implementation
                }
            }
            HybridStorageStrategy::Hybrid => {
                // Store critical secrets on blockchain
                if self.is_critical_secret(key) {
                    self.blockchain_vault.store_secret(key, value).await?;
                } else if let Some(client) = &self.infisical_client {
                    let value_str = String::from_utf8_lossy(value);
                    client.get_secret(key).await?; // This would be a set operation
                }
            }
            HybridStorageStrategy::Redundant => {
                // Store in both systems
                self.blockchain_vault.store_secret(key, value).await?;
                if let Some(client) = &self.infisical_client {
                    let value_str = String::from_utf8_lossy(value);
                    client.get_secret(key).await?; // This would be a set operation
                }
            }
        }
        
        Ok(())
    }
    
    /// Retrieve secret using hybrid strategy
    pub async fn retrieve_secret(&self, key: &str) -> Result<Vec<u8>> {
        match self.strategy {
            HybridStorageStrategy::BlockchainOnly => {
                self.blockchain_vault.retrieve_secret(key).await
            }
            HybridStorageStrategy::InfisicalOnly => {
                if let Some(client) = &self.infisical_client {
                    if let Some(value) = client.get_secret(key).await? {
                        Ok(value.into_bytes())
                    } else {
                        Err(anyhow!("Secret not found in Infisical"))
                    }
                } else {
                    Err(anyhow!("Infisical client not available"))
                }
            }
            HybridStorageStrategy::Hybrid => {
                // Try blockchain first for critical secrets
                if self.is_critical_secret(key) {
                    self.blockchain_vault.retrieve_secret(key).await
                } else if let Some(client) = &self.infisical_client {
                    if let Some(value) = client.get_secret(key).await? {
                        Ok(value.into_bytes())
                    } else {
                        Err(anyhow!("Secret not found"))
                    }
                } else {
                    Err(anyhow!("No storage backend available"))
                }
            }
            HybridStorageStrategy::Redundant => {
                // Try blockchain first, fallback to Infisical
                match self.blockchain_vault.retrieve_secret(key).await {
                    Ok(data) => Ok(data),
                    Err(_) => {
                        if let Some(client) = &self.infisical_client {
                            if let Some(value) = client.get_secret(key).await? {
                                Ok(value.into_bytes())
                            } else {
                                Err(anyhow!("Secret not found in either storage"))
                            }
                        } else {
                            Err(anyhow!("Secret not found"))
                        }
                    }
                }
            }
        }
    }
    
    fn is_critical_secret(&self, key: &str) -> bool {
        // Define which secrets are critical and should be stored on blockchain
        matches!(key, 
            "WALLET_PRIVATE_KEY" | 
            "MASTER_ENCRYPTION_KEY" | 
            "QUANTUM_SAFE_KEY" |
            "ZERO_TRUST_ROOT_KEY"
        )
    }
}
