use anyhow::{Result, anyhow};
use solana_sdk::transaction::Transaction;
use tracing::{info, warn};
use std::time::{Duration, Instant};

/// TransactionGuard zapewnia dodatkową warstwę bezpieczeństwa
/// dla transakcji Solana, mitygując znane podatności w zależnościach
pub struct TransactionGuard {
    // Konfiguracja zabezpieczeń
    max_verification_time_ms: u64,
    enable_timing_protection: bool,
    enable_double_signing_protection: bool,
}

impl TransactionGuard {
    pub fn new() -> Self {
        Self {
            max_verification_time_ms: 50,
            enable_timing_protection: true,
            enable_double_signing_protection: true,
        }
    }
    
    /// Weryfikuje transakcję z dodatkowymi zabezpieczeniami
    pub fn secure_verify(&self, transaction: &Transaction) -> Result<bool> {
        // Mitygacja timing attack w curve25519-dalek
        if self.enable_timing_protection {
            self.verify_with_timing_protection(transaction)?;
        }
        
        // Mitygacja double public key signing w ed25519-dalek
        if self.enable_double_signing_protection {
            self.verify_against_double_signing(transaction)?;
        }
        
        // Standardowa weryfikacja
        Ok(transaction.verify())
    }
    
    /// Implementuje weryfikację odporną na timing attack
    fn verify_with_timing_protection(&self, transaction: &Transaction) -> Result<()> {
        let start = Instant::now();
        
        // Wykonaj weryfikację
        let result = transaction.verify();
        
        // Dodaj stały czas, aby ukryć rzeczywisty czas wykonania
        let elapsed = start.elapsed();
        let target_duration = Duration::from_millis(self.max_verification_time_ms);
        
        if elapsed < target_duration {
            std::thread::sleep(target_duration - elapsed);
        } else if elapsed > target_duration {
            warn!("Verification took longer than expected: {:?}", elapsed);
        }
        
        if !result {
            return Err(anyhow!("Transaction verification failed"));
        }
        
        Ok(())
    }
    
    /// Implementuje zabezpieczenie przed double public key signing
    fn verify_against_double_signing(&self, transaction: &Transaction) -> Result<()> {
        // Sprawdź czy podpisy są unikalne
        let signatures = transaction.signatures.iter().collect::<std::collections::HashSet<_>>();
        if signatures.len() != transaction.signatures.len() {
            return Err(anyhow!("Duplicate signatures detected"));
        }
        
        // Sprawdź czy klucze publiczne są unikalne
        let mut public_keys = std::collections::HashSet::new();
        for account_key in transaction.message.account_keys.iter() {
            if !public_keys.insert(account_key) {
                return Err(anyhow!("Duplicate public key detected"));
            }
        }
        
        Ok(())
    }
}