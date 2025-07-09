use dotenv::dotenv;
use std::env;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use crate::security::infisical_client::SecureEnvLoader;

pub struct EnvLoader {
    pub is_loaded: bool,
    pub secure_loader: Option<SecureEnvLoader>,
}

impl EnvLoader {
    pub fn new() -> Self {
        Self {
            is_loaded: false,
            secure_loader: None,
        }
    }

    /// Create new EnvLoader with Infisical integration
    pub async fn new_secure() -> Self {
        let secure_loader = SecureEnvLoader::new(true).await;
        Self {
            is_loaded: false,
            secure_loader: Some(secure_loader),
        }
    }
    
    pub fn load(&mut self) -> Result<()> {
        // Załaduj zmienne środowiskowe z pliku .env
        match dotenv() {
            Ok(_) => {
                self.is_loaded = true;
                info!("Zmienne środowiskowe załadowane z pliku .env");
            },
            Err(e) => {
                warn!("Nie udało się załadować pliku .env: {}", e);
                warn!("Używanie zmiennych środowiskowych systemu");
            }
        }
        
        // Sprawdź czy wymagane zmienne są dostępne
        self.validate_required_vars()?;
        
        Ok(())
    }
    
    fn validate_required_vars(&self) -> Result<()> {
        let required_vars = vec![
            "OPENAI_API_KEY",
            "HELIUS_API_KEY",
            "QUICKNODE_API_KEY",
        ];
        
        let mut missing_vars = Vec::new();
        
        for var in required_vars {
            if env::var(var).is_err() {
                missing_vars.push(var);
            }
        }
        
        if !missing_vars.is_empty() {
            return Err(anyhow!("Brakujące wymagane zmienne środowiskowe: {:?}", missing_vars));
        }
        
        Ok(())
    }
    
    /// Get API key with Infisical support
    pub async fn get_api_key(&self, key_name: &str) -> Result<String> {
        // Try Infisical first if available
        if let Some(secure_loader) = &self.secure_loader {
            if let Some(value) = secure_loader.get(key_name).await {
                info!("🔐 Retrieved '{}' from Infisical", key_name);
                return Ok(value);
            }
        }

        // Fallback to environment variable
        env::var(key_name)
            .map_err(|_| anyhow!("Nie znaleziono klucza API: {}", key_name))
    }
    
    /// Get RPC URL with Infisical support
    pub async fn get_rpc_url(&self, provider: &str) -> Result<String> {
        let env_var = format!("{}_RPC_URL", provider.to_uppercase());

        // Try Infisical first if available
        if let Some(secure_loader) = &self.secure_loader {
            if let Some(value) = secure_loader.get(&env_var).await {
                info!("🔐 Retrieved RPC URL for '{}' from Infisical", provider);
                return Ok(value);
            }
        }

        // Fallback to environment variable
        env::var(&env_var)
            .map_err(|_| anyhow!("Nie znaleziono URL RPC dla: {}", provider))
    }

    /// Get WebSocket URL with Infisical support
    pub async fn get_ws_url(&self, provider: &str) -> Result<String> {
        let env_var = format!("{}_WS_URL", provider.to_uppercase());

        // Try Infisical first if available
        if let Some(secure_loader) = &self.secure_loader {
            if let Some(value) = secure_loader.get(&env_var).await {
                info!("🔐 Retrieved WS URL for '{}' from Infisical", provider);
                return Ok(value);
            }
        }

        // Fallback to environment variable
        env::var(&env_var)
            .map_err(|_| anyhow!("Nie znaleziono URL WebSocket dla: {}", provider))
    }
}