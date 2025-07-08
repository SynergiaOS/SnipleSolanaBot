// THE OVERMIND PROTOCOL - Cluster Orchestrator Module
// Enhanced RPC cluster management with Redis state persistence and intelligent rotation

use anyhow::{anyhow, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct ClusterOrchestrator {
    pub current_rpc: Arc<Mutex<String>>,
    pub backup_rpcs: Vec<String>,
    pub redis_conn: Arc<Mutex<redis::aio::Connection>>,
    pub jito_enabled: bool,
    pub health_check_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcHealthStatus {
    pub endpoint: String,
    pub is_healthy: bool,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
    pub error_count: u32,
}

impl ClusterOrchestrator {
    /// Create new cluster orchestrator with Redis state management
    pub async fn new(
        redis_url: &str,
        main_rpc: String,
        backups: Vec<String>,
        jito: bool,
    ) -> Result<Self> {
        info!("🌐 Initializing ClusterOrchestrator with {} backup RPCs", backups.len());
        
        let client = redis::Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
        
        let conn = client.get_async_connection().await
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;

        let orchestrator = Self {
            current_rpc: Arc::new(Mutex::new(main_rpc.clone())),
            backup_rpcs: backups,
            redis_conn: Arc::new(Mutex::new(conn)),
            jito_enabled: jito,
            health_check_interval: Duration::from_secs(30),
        };

        // Initialize health status in Redis
        orchestrator.initialize_health_status().await?;
        
        info!("✅ ClusterOrchestrator initialized successfully");
        Ok(orchestrator)
    }

    /// Initialize health status for all RPC endpoints
    async fn initialize_health_status(&self) -> Result<()> {
        let mut conn = self.redis_conn.lock().await;
        
        // Initialize main RPC
        let main_rpc = self.current_rpc.lock().await.clone();
        let main_status = RpcHealthStatus {
            endpoint: main_rpc.clone(),
            is_healthy: true,
            last_check: chrono::Utc::now(),
            response_time_ms: 0,
            error_count: 0,
        };
        
        let status_json = serde_json::to_string(&main_status)?;
        let _: () = conn.set_ex(
            format!("rpc_health:{}", main_rpc),
            status_json,
            3600, // 1 hour TTL
        ).await?;

        // Initialize backup RPCs
        for rpc in &self.backup_rpcs {
            let status = RpcHealthStatus {
                endpoint: rpc.clone(),
                is_healthy: true,
                last_check: chrono::Utc::now(),
                response_time_ms: 0,
                error_count: 0,
            };
            
            let status_json = serde_json::to_string(&status)?;
            let _: () = conn.set_ex(
                format!("rpc_health:{}", rpc),
                status_json,
                3600,
            ).await?;
        }

        info!("📊 Initialized health status for {} RPC endpoints", self.backup_rpcs.len() + 1);
        Ok(())
    }

    /// Intelligent RPC rotation with Redis state persistence
    pub async fn rotate_rpc(&self) -> Result<String> {
        let current = self.current_rpc.lock().await.clone();
        let backups = &self.backup_rpcs;
        
        // Find next healthy RPC
        let next_rpc = match backups.iter().position(|r| r == &current) {
            Some(pos) if pos < backups.len() - 1 => &backups[pos + 1],
            _ => &backups[0],
        };

        // Check if next RPC is healthy
        if self.is_rpc_healthy(next_rpc).await? {
            // Save active RPC state in Redis
            let mut conn = self.redis_conn.lock().await;
            let _: () = conn.set_ex("active_rpc", next_rpc, 600).await?;

            let mut guard = self.current_rpc.lock().await;
            *guard = next_rpc.clone();
            
            info!("🔄 Rotated to RPC: {}", next_rpc);
            Ok(next_rpc.clone())
        } else {
            warn!("⚠️ Next RPC {} is unhealthy, finding alternative", next_rpc);
            self.find_healthy_rpc().await
        }
    }

    /// Find the healthiest available RPC endpoint
    pub async fn find_healthy_rpc(&self) -> Result<String> {
        let mut best_rpc = None;
        let mut best_response_time = u64::MAX;

        // Check current RPC first
        let current = self.current_rpc.lock().await.clone();
        if let Ok(status) = self.get_rpc_health_status(&current).await {
            if status.is_healthy {
                best_rpc = Some(current.clone());
                best_response_time = status.response_time_ms;
            }
        }

        // Check backup RPCs
        for rpc in &self.backup_rpcs {
            if let Ok(status) = self.get_rpc_health_status(rpc).await {
                if status.is_healthy && status.response_time_ms < best_response_time {
                    best_rpc = Some(rpc.clone());
                    best_response_time = status.response_time_ms;
                }
            }
        }

        match best_rpc {
            Some(rpc) => {
                // Update current RPC if different
                let mut current_guard = self.current_rpc.lock().await;
                if *current_guard != rpc {
                    *current_guard = rpc.clone();
                    
                    // Save to Redis
                    let mut conn = self.redis_conn.lock().await;
                    let _: () = conn.set_ex("active_rpc", &rpc, 600).await?;
                    
                    info!("🎯 Selected healthiest RPC: {} ({}ms)", rpc, best_response_time);
                }
                Ok(rpc)
            }
            None => {
                error!("❌ No healthy RPC endpoints available");
                Err(anyhow!("No healthy RPC endpoints available"))
            }
        }
    }

    /// Check if specific RPC endpoint is healthy
    pub async fn is_rpc_healthy(&self, rpc_url: &str) -> Result<bool> {
        match self.get_rpc_health_status(rpc_url).await {
            Ok(status) => Ok(status.is_healthy),
            Err(_) => Ok(false), // Assume unhealthy if can't get status
        }
    }

    /// Get RPC health status from Redis
    pub async fn get_rpc_health_status(&self, rpc_url: &str) -> Result<RpcHealthStatus> {
        let mut conn = self.redis_conn.lock().await;
        let status_json: String = conn.get(format!("rpc_health:{}", rpc_url)).await
            .map_err(|e| anyhow!("Failed to get RPC health status: {}", e))?;
        
        let status: RpcHealthStatus = serde_json::from_str(&status_json)
            .map_err(|e| anyhow!("Failed to parse RPC health status: {}", e))?;
        
        Ok(status)
    }

    /// Update RPC health status in Redis
    pub async fn update_rpc_health(&self, rpc_url: &str, is_healthy: bool, response_time_ms: u64) -> Result<()> {
        let mut status = self.get_rpc_health_status(rpc_url).await
            .unwrap_or_else(|_| RpcHealthStatus {
                endpoint: rpc_url.to_string(),
                is_healthy: true,
                last_check: chrono::Utc::now(),
                response_time_ms: 0,
                error_count: 0,
            });

        status.is_healthy = is_healthy;
        status.last_check = chrono::Utc::now();
        status.response_time_ms = response_time_ms;
        
        if !is_healthy {
            status.error_count += 1;
        } else {
            status.error_count = 0; // Reset on successful health check
        }

        let status_json = serde_json::to_string(&status)?;
        let mut conn = self.redis_conn.lock().await;
        let _: () = conn.set_ex(
            format!("rpc_health:{}", rpc_url),
            status_json,
            3600,
        ).await?;

        debug!("📊 Updated health status for {}: healthy={}, response_time={}ms", 
               rpc_url, is_healthy, response_time_ms);
        Ok(())
    }

    /// Get current active RPC endpoint
    pub async fn get_current_rpc(&self) -> String {
        self.current_rpc.lock().await.clone()
    }

    /// Get all RPC endpoints with their health status
    pub async fn get_all_rpc_status(&self) -> Result<Vec<RpcHealthStatus>> {
        let mut statuses = Vec::new();
        
        // Get current RPC status
        let current = self.current_rpc.lock().await.clone();
        if let Ok(status) = self.get_rpc_health_status(&current).await {
            statuses.push(status);
        }

        // Get backup RPC statuses
        for rpc in &self.backup_rpcs {
            if let Ok(status) = self.get_rpc_health_status(rpc).await {
                statuses.push(status);
            }
        }

        Ok(statuses)
    }

    /// Start background health monitoring
    pub async fn start_health_monitoring(&self) -> Result<()> {
        let orchestrator = self.clone();
        
        tokio::spawn(async move {
            loop {
                if let Err(e) = orchestrator.perform_health_checks().await {
                    error!("❌ Health check failed: {}", e);
                }
                sleep(orchestrator.health_check_interval).await;
            }
        });

        info!("🔍 Started background health monitoring");
        Ok(())
    }

    /// Perform health checks on all RPC endpoints
    async fn perform_health_checks(&self) -> Result<()> {
        let client = reqwest::Client::new();
        
        // Check current RPC
        let current = self.current_rpc.lock().await.clone();
        self.check_single_rpc(&client, &current).await;

        // Check backup RPCs
        for rpc in &self.backup_rpcs {
            self.check_single_rpc(&client, rpc).await;
        }

        Ok(())
    }

    /// Check health of a single RPC endpoint
    async fn check_single_rpc(&self, client: &reqwest::Client, rpc_url: &str) {
        let start_time = std::time::Instant::now();
        
        let health_check_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getHealth"
        });

        match client
            .post(rpc_url)
            .json(&health_check_request)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                let is_healthy = response.status().is_success();
                
                if let Err(e) = self.update_rpc_health(rpc_url, is_healthy, response_time).await {
                    error!("Failed to update RPC health for {}: {}", rpc_url, e);
                }
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                warn!("RPC health check failed for {}: {}", rpc_url, e);
                
                if let Err(e) = self.update_rpc_health(rpc_url, false, response_time).await {
                    error!("Failed to update RPC health for {}: {}", rpc_url, e);
                }
            }
        }
    }
}
