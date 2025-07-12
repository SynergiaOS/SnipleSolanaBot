use reqwest::Client;
use serde_json::{json, Value};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

pub struct InfisicalVault {
    client: Client,
    service_token: String,
    project_id: String,
    environment: String,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl InfisicalVault {
    pub fn new(service_token: String, project_id: String, environment: String) -> Self {
        Self {
            client: Client::new(),
            service_token,
            project_id,
            environment,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_secret(&self, key: &str) -> Result<String> {
        // Sprawdź cache
        {
            let cache = self.cache.read().await;
            if let Some(value) = cache.get(key) {
                return Ok(value.clone());
            }
        }
        
        // Pobierz z Infisical
        info!("Fetching secret {} from Infisical", key);
        
        let response = self.client
            .get("https://app.infisical.com/api/v3/secrets/raw")
            .header("Authorization", format!("Bearer {}", self.service_token))
            .query(&[
                ("environment", &self.environment),
                ("secretPath", &format!("/{}", key)),
                ("projectId", &self.project_id),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;
        
        // Parsuj odpowiedź
        if let Some(secret) = response.get("secret") {
            if let Some(value) = secret.get("value") {
                if let Some(value_str) = value.as_str() {
                    // Aktualizuj cache
                    let mut cache = self.cache.write().await;
                    cache.insert(key.to_string(), value_str.to_string());
                    
                    return Ok(value_str.to_string());
                }
            }
        }
        
        error!("Failed to parse Infisical response: {:?}", response);
        Err(anyhow!("Invalid Infisical response format"))
    }
}