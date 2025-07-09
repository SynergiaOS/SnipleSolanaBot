//! INFISICAL SECURITY CLIENT
//! 
//! Secure secret management using Infisical instead of plaintext .env files
//! Eliminates security vulnerabilities from storing sensitive data in plaintext

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::{debug, info, warn, error};
use super::dragonflydb_cache::DragonflyCache;

/// Infisical client for secure secret management with DragonflyDB cache
#[derive(Debug)]
pub struct InfisicalClient {
    /// HTTP client
    client: Client,

    /// Infisical configuration
    config: InfisicalConfig,

    /// Cached secrets (local cache)
    secrets_cache: Arc<RwLock<HashMap<String, CachedSecret>>>,

    /// Authentication token
    auth_token: Arc<RwLock<Option<String>>>,

    /// Last authentication time
    last_auth: Arc<RwLock<Option<Instant>>>,

    /// DragonflyDB cache layer (optional)
    dragonflydb_cache: Option<Arc<DragonflyCache>>,
}

/// Infisical configuration
#[derive(Debug, Clone)]
pub struct InfisicalConfig {
    /// Infisical API URL
    pub api_url: String,
    
    /// Project ID
    pub project_id: String,
    
    /// Environment (dev, staging, prod)
    pub environment: String,
    
    /// Client ID for machine identity
    pub client_id: String,
    
    /// Client secret for machine identity
    pub client_secret: String,

    /// Service token for direct authentication (alternative to client_id/secret)
    pub service_token: Option<String>,

    /// Cache TTL for secrets
    pub cache_ttl: Duration,

    /// Auto-refresh interval
    pub refresh_interval: Duration,
}

/// Cached secret
#[derive(Debug, Clone)]
pub struct CachedSecret {
    pub key: String,
    pub value: String,
    pub cached_at: Instant,
    pub ttl: Duration,
}

/// Authentication response
#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: u64,
    #[serde(rename = "tokenType")]
    pub token_type: String,
}

/// Secrets response
#[derive(Debug, Deserialize)]
pub struct SecretsResponse {
    pub secrets: Vec<Secret>,
}

/// Secret from Infisical
#[derive(Debug, Deserialize)]
pub struct Secret {
    #[serde(rename = "secretKey")]
    pub secret_key: String,
    #[serde(rename = "secretValue")]
    pub secret_value: String,
    #[serde(rename = "secretComment")]
    pub secret_comment: Option<String>,
}

/// Authentication request
#[derive(Debug, Serialize)]
pub struct AuthRequest {
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "clientSecret")]
    pub client_secret: String,
}

impl InfisicalClient {
    /// Create new Infisical client
    pub fn new(config: InfisicalConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            config,
            secrets_cache: Arc::new(RwLock::new(HashMap::new())),
            auth_token: Arc::new(RwLock::new(None)),
            last_auth: Arc::new(RwLock::new(None)),
            dragonflydb_cache: None,
        }
    }
    
    /// Initialize from environment variables with production defaults
    pub fn from_env() -> Result<Self> {
        let config = InfisicalConfig {
            api_url: env::var("INFISICAL_API_URL")
                .unwrap_or_else(|_| "https://app.infisical.com/api".to_string()),
            project_id: env::var("INFISICAL_PROJECT_ID")
                .unwrap_or_else(|_| "73c2f3cb-c922-4a46-a333-7b96fbc6301a".to_string()),
            environment: env::var("INFISICAL_ENVIRONMENT")
                .unwrap_or_else(|_| "production".to_string()),
            client_id: env::var("INFISICAL_CLIENT_ID").unwrap_or_default(),
            client_secret: env::var("INFISICAL_CLIENT_SECRET").unwrap_or_default(),
            service_token: env::var("INFISICAL_SERVICE_TOKEN").ok(),
            cache_ttl: Duration::from_secs(300), // 5 minutes
            refresh_interval: Duration::from_secs(240), // 4 minutes
        };
        
        Ok(Self::new(config))
    }

    /// Enable DragonflyDB cache layer
    pub fn with_dragonflydb_cache(mut self, cache: Arc<DragonflyCache>) -> Self {
        self.dragonflydb_cache = Some(cache);
        info!("🐉 DragonflyDB cache layer enabled for Infisical client");
        self
    }
    
    /// Start the client with automatic token refresh
    pub async fn start(&self) -> Result<()> {
        info!("🔐 Starting Infisical client for secure secret management");
        
        // Initial authentication
        self.authenticate().await?;
        
        // Load all secrets
        self.refresh_secrets().await?;
        
        // Start background refresh task
        self.start_refresh_task().await;
        
        info!("✅ Infisical client started successfully");
        Ok(())
    }
    
    /// Authenticate with Infisical (supports both service token and machine identity)
    async fn authenticate(&self) -> Result<()> {
        debug!("🔑 Authenticating with Infisical");

        // If service token is provided, use it directly
        if let Some(service_token) = &self.config.service_token {
            debug!("🔑 Using service token authentication");
            {
                let mut token = self.auth_token.write().unwrap();
                *token = Some(service_token.clone());
            }
            {
                let mut last_auth = self.last_auth.write().unwrap();
                *last_auth = Some(Instant::now());
            }
            info!("✅ Successfully authenticated with Infisical using service token");
            return Ok(());
        }

        // Fallback to machine identity authentication
        debug!("🔑 Using machine identity authentication");
        let auth_request = AuthRequest {
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
        };

        let response = self.client
            .post(&format!("{}/v1/auth/universal-auth/login", self.config.api_url))
            .json(&auth_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Authentication failed: {}", error_text));
        }

        let auth_response: AuthResponse = response.json().await?;

        {
            let mut token = self.auth_token.write().unwrap();
            *token = Some(auth_response.access_token);
        }
        
        {
            let mut last_auth = self.last_auth.write().unwrap();
            *last_auth = Some(Instant::now());
        }
        
        info!("✅ Successfully authenticated with Infisical");
        Ok(())
    }
    
    /// Refresh all secrets from Infisical
    async fn refresh_secrets(&self) -> Result<()> {
        debug!("🔄 Refreshing secrets from Infisical");
        
        let token = {
            let token_guard = self.auth_token.read().unwrap();
            token_guard.clone().ok_or_else(|| anyhow!("Not authenticated"))?
        };
        
        let response = self.client
            .get(&format!(
                "{}/v3/secrets/raw?environment={}&workspaceId={}",
                self.config.api_url,
                self.config.environment,
                self.config.project_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to fetch secrets: {}", error_text));
        }
        
        let secrets_response: SecretsResponse = response.json().await?;
        
        {
            let mut cache = self.secrets_cache.write().unwrap();
            cache.clear();
            
            for secret in secrets_response.secrets {
                let cached_secret = CachedSecret {
                    key: secret.secret_key.clone(),
                    value: secret.secret_value,
                    cached_at: Instant::now(),
                    ttl: self.config.cache_ttl,
                };
                
                cache.insert(secret.secret_key, cached_secret);
            }
        }
        
        let secret_count = {
            let cache = self.secrets_cache.read().unwrap();
            cache.len()
        };
        
        info!("✅ Refreshed {} secrets from Infisical", secret_count);
        Ok(())
    }
    
    /// Get secret value by key (with DragonflyDB cache support)
    pub async fn get_secret(&self, key: &str) -> Result<Option<String>> {
        // Check DragonflyDB cache first if available
        if let Some(dragonflydb) = &self.dragonflydb_cache {
            match dragonflydb.get(key).await {
                Ok(Some(value)) => {
                    debug!("🐉 Retrieved secret '{}' from DragonflyDB cache", key);
                    return Ok(Some(value));
                }
                Ok(None) => {
                    debug!("🐉 Secret '{}' not found in DragonflyDB cache", key);
                }
                Err(e) => {
                    warn!("⚠️ DragonflyDB cache error for '{}': {}", key, e);
                }
            }
        }

        // Check local cache
        {
            let cache = self.secrets_cache.read().unwrap();
            if let Some(cached_secret) = cache.get(key) {
                if cached_secret.cached_at.elapsed() < cached_secret.ttl {
                    debug!("📋 Retrieved secret '{}' from local cache", key);

                    // Update DragonflyDB cache if available
                    if let Some(dragonflydb) = &self.dragonflydb_cache {
                        let _ = dragonflydb.set(key, &cached_secret.value, Some(cached_secret.ttl)).await;
                    }

                    return Ok(Some(cached_secret.value.clone()));
                }
            }
        }

        // If not in cache or expired, refresh and try again
        self.refresh_secrets().await?;

        let cache = self.secrets_cache.read().unwrap();
        if let Some(cached_secret) = cache.get(key) {
            debug!("📋 Retrieved secret '{}' after refresh", key);

            // Update DragonflyDB cache if available
            if let Some(dragonflydb) = &self.dragonflydb_cache {
                let _ = dragonflydb.set(key, &cached_secret.value, Some(cached_secret.ttl)).await;
            }

            Ok(Some(cached_secret.value.clone()))
        } else {
            warn!("⚠️ Secret '{}' not found in Infisical", key);
            Ok(None)
        }
    }
    
    /// Get secret value or return default
    pub async fn get_secret_or_default(&self, key: &str, default: &str) -> String {
        match self.get_secret(key).await {
            Ok(Some(value)) => value,
            Ok(None) => {
                warn!("⚠️ Secret '{}' not found, using default", key);
                default.to_string()
            }
            Err(e) => {
                error!("❌ Failed to get secret '{}': {}, using default", key, e);
                default.to_string()
            }
        }
    }
    
    /// Get secret value or return environment variable fallback
    pub async fn get_secret_or_env(&self, key: &str, env_key: &str) -> Option<String> {
        // Try Infisical first
        match self.get_secret(key).await {
            Ok(Some(value)) => {
                debug!("📋 Retrieved '{}' from Infisical", key);
                Some(value)
            }
            Ok(None) | Err(_) => {
                // Fallback to environment variable
                if let Ok(env_value) = env::var(env_key) {
                    warn!("⚠️ Using environment fallback for '{}'", key);
                    Some(env_value)
                } else {
                    error!("❌ Secret '{}' not found in Infisical or environment", key);
                    None
                }
            }
        }
    }
    
    /// Get all secrets as HashMap
    pub async fn get_all_secrets(&self) -> Result<HashMap<String, String>> {
        let cache = self.secrets_cache.read().unwrap();
        let mut secrets = HashMap::new();
        
        for (key, cached_secret) in cache.iter() {
            if cached_secret.cached_at.elapsed() < cached_secret.ttl {
                secrets.insert(key.clone(), cached_secret.value.clone());
            }
        }
        
        Ok(secrets)
    }
    
    /// Start background refresh task
    async fn start_refresh_task(&self) {
        let client = self.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(client.config.refresh_interval);
            
            loop {
                interval.tick().await;
                
                // Check if we need to re-authenticate
                let needs_auth = {
                    let last_auth = client.last_auth.read().unwrap();
                    match *last_auth {
                        Some(time) => time.elapsed() > Duration::from_secs(3600), // 1 hour
                        None => true,
                    }
                };
                
                if needs_auth {
                    if let Err(e) = client.authenticate().await {
                        error!("❌ Failed to re-authenticate with Infisical: {}", e);
                        continue;
                    }
                }
                
                // Refresh secrets
                if let Err(e) = client.refresh_secrets().await {
                    error!("❌ Failed to refresh secrets from Infisical: {}", e);
                }
            }
        });
    }
    
    /// Check if client is authenticated
    pub fn is_authenticated(&self) -> bool {
        let token = self.auth_token.read().unwrap();
        token.is_some()
    }
    
    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.secrets_cache.read().unwrap();
        let total = cache.len();
        let expired = cache.values()
            .filter(|secret| secret.cached_at.elapsed() >= secret.ttl)
            .count();
        
        (total, expired)
    }
}

impl Clone for InfisicalClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            secrets_cache: Arc::clone(&self.secrets_cache),
            auth_token: Arc::clone(&self.auth_token),
            last_auth: Arc::clone(&self.last_auth),
        }
    }
}

/// Create Infisical client from environment
pub async fn create_infisical_client() -> Result<InfisicalClient> {
    let client = InfisicalClient::from_env()?;
    client.start().await?;
    Ok(client)
}

/// Secure environment loader using Infisical with DragonflyDB cache
pub struct SecureEnvLoader {
    infisical_client: Option<InfisicalClient>,
    dragonflydb_cache: Option<Arc<DragonflyCache>>,
    fallback_to_env: bool,
}

impl SecureEnvLoader {
    /// Create new secure environment loader with optional DragonflyDB cache
    pub async fn new(fallback_to_env: bool) -> Self {
        // Initialize DragonflyDB cache if configured
        let dragonflydb_cache = match super::dragonflydb_cache::create_dragonflydb_cache().await {
            Ok(cache) => {
                info!("🐉 DragonflyDB cache initialized successfully");
                Some(Arc::new(cache))
            }
            Err(e) => {
                warn!("⚠️ Failed to initialize DragonflyDB cache: {}", e);
                None
            }
        };

        // Initialize Infisical client
        let infisical_client = match create_infisical_client().await {
            Ok(mut client) => {
                // Enable DragonflyDB cache if available
                if let Some(cache) = &dragonflydb_cache {
                    client = client.with_dragonflydb_cache(cache.clone());
                }
                info!("✅ Infisical client initialized successfully");
                Some(client)
            }
            Err(e) => {
                if fallback_to_env {
                    warn!("⚠️ Failed to initialize Infisical, falling back to environment: {}", e);
                } else {
                    error!("❌ Failed to initialize Infisical: {}", e);
                }
                None
            }
        };

        Self {
            infisical_client,
            dragonflydb_cache,
            fallback_to_env,
        }
    }
    
    /// Get secret value
    pub async fn get(&self, key: &str) -> Option<String> {
        if let Some(client) = &self.infisical_client {
            if let Ok(Some(value)) = client.get_secret(key).await {
                return Some(value);
            }
        }
        
        if self.fallback_to_env {
            env::var(key).ok()
        } else {
            None
        }
    }
    
    /// Get secret value or default
    pub async fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key).await.unwrap_or_else(|| default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_infisical_config_from_env() {
        // This test requires environment variables to be set
        // Skip if not available
        if env::var("INFISICAL_PROJECT_ID").is_err() {
            return;
        }
        
        let client = InfisicalClient::from_env().unwrap();
        assert!(!client.config.project_id.is_empty());
    }
}
