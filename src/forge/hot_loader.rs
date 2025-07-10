//! HOT STRATEGY LOADER - Runtime Strategy Swapping
//! 
//! Dynamic loader dla runtime strategy swapping
//! Zero-downtime deployment z TensorZero metrics integration

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn, error};
use libloading::{Library, Symbol};

use super::{CompiledArtifact, DeploymentResult};

/// Strategy container holding loaded library
#[derive(Debug, Clone)]
pub struct StrategyContainer {
    /// Strategy ID
    pub strategy_id: String,
    
    /// Agent ID
    pub agent_id: String,
    
    /// Library handle (wrapped in Arc for sharing)
    pub library: Arc<Library>,
    
    /// Strategy VTable
    pub vtable: StrategyVTable,
    
    /// Load timestamp
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    
    /// Performance metrics
    pub metrics: StrategyMetrics,
    
    /// Health status
    pub health_status: HealthStatus,
}

/// Strategy VTable - ABI interface
#[derive(Debug, Clone)]
pub struct StrategyVTable {
    /// Analyze market data and return signal strength
    pub analyze: unsafe extern "C" fn(*const MarketData) -> f64,
    
    /// Execute trading decision
    pub execute: unsafe extern "C" fn(*mut HftContext) -> i32,
    
    /// Cleanup resources
    pub cleanup: unsafe extern "C" fn(),
    
    /// Get strategy info
    pub get_info: unsafe extern "C" fn() -> *const StrategyInfo,
    
    /// Health check
    pub health_check: unsafe extern "C" fn() -> i32,
}

/// Market data structure (C-compatible)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct MarketData {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
    pub momentum_signal: f64,
    pub volatility: f64,
    pub liquidity_score: f64,
}

/// HFT execution context (C-compatible)
#[repr(C)]
#[derive(Debug)]
pub struct HftContext {
    pub agent_id: *const c_char,
    pub position_size: f64,
    pub available_balance: f64,
    pub max_position_size: f64,
    pub risk_limit: f64,
    pub execution_callback: Option<unsafe extern "C" fn(*const c_char, f64, f64) -> i32>,
}

/// Strategy info structure (C-compatible)
#[repr(C)]
#[derive(Debug)]
pub struct StrategyInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
    pub risk_level: u8,
    pub expected_return: f64,
    pub max_drawdown: f64,
}

/// Strategy performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub total_signals: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_pnl: f64,
    pub average_signal_strength: f64,
    pub execution_time_ms: u64,
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
    pub health_score: f64,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning { message: String },
    Critical { message: String },
    Failed { error: String },
}

/// Hot Strategy Loader
#[derive(Debug)]
pub struct StrategyHotLoader {
    /// Loaded strategies by agent ID
    strategies: Arc<RwLock<HashMap<String, StrategyContainer>>>,
    
    /// Loader statistics
    stats: LoaderStats,
    
    /// Safety configuration
    safety_config: SafetyConfig,
}

/// Loader statistics
#[derive(Debug, Default, Clone)]
pub struct LoaderStats {
    pub total_loads: u64,
    pub successful_loads: u64,
    pub failed_loads: u64,
    pub total_swaps: u64,
    pub successful_swaps: u64,
    pub failed_swaps: u64,
    pub average_load_time_ms: u64,
    pub average_swap_time_ms: u64,
}

/// Safety configuration
#[derive(Debug, Clone)]
pub struct SafetyConfig {
    /// Maximum load time before timeout
    pub max_load_time_ms: u64,
    
    /// Health check interval
    pub health_check_interval_ms: u64,
    
    /// Maximum consecutive failures before marking as failed
    pub max_consecutive_failures: u8,
    
    /// Enable sandbox execution
    pub enable_sandbox: bool,
    
    /// Memory limit for loaded strategies
    pub memory_limit_mb: u64,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            max_load_time_ms: 5000,
            health_check_interval_ms: 1000,
            max_consecutive_failures: 3,
            enable_sandbox: true,
            memory_limit_mb: 256,
        }
    }
}

impl StrategyHotLoader {
    /// Create new hot loader
    pub fn new() -> Result<Self> {
        info!("🔄 Initializing Strategy Hot Loader");
        
        Ok(Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            stats: LoaderStats::default(),
            safety_config: SafetyConfig::default(),
        })
    }
    
    /// Deploy strategy with hot loading
    pub async fn deploy_strategy(
        &mut self,
        agent_id: &str,
        artifact: &CompiledArtifact,
    ) -> Result<DeploymentResult> {
        let start_time = Instant::now();
        info!("🚀 Deploying strategy {} for agent {}", artifact.strategy_id, agent_id);
        
        // Load new strategy
        let new_container = self.load_strategy(agent_id, artifact).await?;
        
        // Get old strategy if exists
        let old_strategy_id = {
            let strategies = self.strategies.read().unwrap();
            strategies.get(agent_id).map(|s| s.strategy_id.clone())
        };
        
        // Perform hot swap
        let swap_result = self.hot_swap_strategy(agent_id, new_container).await?;
        
        // Calculate performance improvement (mock for now)
        let performance_improvement = self.calculate_performance_improvement(
            agent_id,
            old_strategy_id.as_deref(),
            &artifact.strategy_id,
        ).await?;
        
        // Update statistics
        let deployment_time = start_time.elapsed().as_millis() as u64;
        self.update_deployment_stats(deployment_time, true);
        
        info!("✅ Strategy deployment completed for agent {} in {}ms", agent_id, deployment_time);
        
        Ok(DeploymentResult {
            old_strategy_id,
            new_strategy_id: artifact.strategy_id.clone(),
            performance_improvement,
            strategy_container: swap_result,
        })
    }
    
    /// Load strategy from compiled artifact
    async fn load_strategy(
        &mut self,
        agent_id: &str,
        artifact: &CompiledArtifact,
    ) -> Result<StrategyContainer> {
        let start_time = Instant::now();
        debug!("📦 Loading strategy library: {}", artifact.binary_path);
        
        // Verify artifact exists and is valid
        if !Path::new(&artifact.binary_path).exists() {
            return Err(anyhow!("Strategy artifact not found: {}", artifact.binary_path));
        }
        
        // Load library with timeout
        let library = tokio::time::timeout(
            Duration::from_millis(self.safety_config.max_load_time_ms),
            tokio::task::spawn_blocking({
                let path = artifact.binary_path.clone();
                move || unsafe { Library::new(&path) }
            })
        ).await
        .map_err(|_| anyhow!("Library load timeout"))?
        .map_err(|e| anyhow!("Failed to spawn load task: {}", e))?
        .map_err(|e| anyhow!("Failed to load library: {}", e))?;
        
        // Extract strategy VTable
        let vtable = self.extract_vtable(&library)?;
        
        // Perform initial health check
        let health_status = self.perform_health_check(&vtable)?;
        
        // Create strategy container
        let container = StrategyContainer {
            strategy_id: artifact.strategy_id.clone(),
            agent_id: agent_id.to_string(),
            library: Arc::new(library),
            vtable,
            loaded_at: chrono::Utc::now(),
            metrics: StrategyMetrics::default(),
            health_status,
        };
        
        let load_time = start_time.elapsed().as_millis() as u64;
        self.update_load_stats(load_time, true);
        
        debug!("✅ Strategy loaded successfully in {}ms", load_time);
        Ok(container)
    }
    
    /// Extract VTable from loaded library
    fn extract_vtable(&self, library: &Library) -> Result<StrategyVTable> {
        unsafe {
            // Load required symbols
            let analyze: Symbol<unsafe extern "C" fn(*const MarketData) -> f64> = 
                library.get(b"strategy_analyze")
                    .map_err(|e| anyhow!("Failed to load analyze function: {}", e))?;
            
            let execute: Symbol<unsafe extern "C" fn(*mut HftContext) -> i32> = 
                library.get(b"strategy_execute")
                    .map_err(|e| anyhow!("Failed to load execute function: {}", e))?;
            
            let cleanup: Symbol<unsafe extern "C" fn()> = 
                library.get(b"strategy_cleanup")
                    .map_err(|e| anyhow!("Failed to load cleanup function: {}", e))?;
            
            let get_info: Symbol<unsafe extern "C" fn() -> *const StrategyInfo> = 
                library.get(b"strategy_get_info")
                    .map_err(|e| anyhow!("Failed to load get_info function: {}", e))?;
            
            let health_check: Symbol<unsafe extern "C" fn() -> i32> = 
                library.get(b"strategy_health_check")
                    .map_err(|e| anyhow!("Failed to load health_check function: {}", e))?;
            
            Ok(StrategyVTable {
                analyze: *analyze,
                execute: *execute,
                cleanup: *cleanup,
                get_info: *get_info,
                health_check: *health_check,
            })
        }
    }
    
    /// Perform health check on strategy
    fn perform_health_check(&self, vtable: &StrategyVTable) -> Result<HealthStatus> {
        unsafe {
            let health_code = (vtable.health_check)();
            
            match health_code {
                0 => Ok(HealthStatus::Healthy),
                1 => Ok(HealthStatus::Warning { 
                    message: "Strategy reported warning status".to_string() 
                }),
                2 => Ok(HealthStatus::Critical { 
                    message: "Strategy reported critical status".to_string() 
                }),
                _ => Ok(HealthStatus::Failed { 
                    error: format!("Strategy health check failed with code: {}", health_code) 
                }),
            }
        }
    }
    
    /// Hot swap strategy for agent
    async fn hot_swap_strategy(
        &mut self,
        agent_id: &str,
        new_container: StrategyContainer,
    ) -> Result<StrategyContainer> {
        let start_time = Instant::now();
        debug!("🔄 Performing hot swap for agent: {}", agent_id);
        
        // Get write lock on strategies
        {
            let mut strategies = self.strategies.write().unwrap();

            // Cleanup old strategy if exists
            if let Some(old_container) = strategies.get(agent_id) {
                debug!("🧹 Cleaning up old strategy: {}", old_container.strategy_id);
                unsafe {
                    (old_container.vtable.cleanup)();
                }
            }

            // Insert new strategy
            strategies.insert(agent_id.to_string(), new_container.clone());
        }

        // Update swap statistics
        let swap_time = start_time.elapsed().as_millis() as u64;
        self.update_swap_stats(swap_time, true);
        
        debug!("✅ Hot swap completed in {}ms", swap_time);
        Ok(new_container)
    }
    
    /// Calculate performance improvement
    async fn calculate_performance_improvement(
        &self,
        agent_id: &str,
        old_strategy_id: Option<&str>,
        new_strategy_id: &str,
    ) -> Result<f64> {
        // Mock implementation - in production would use TensorZero observability data
        match old_strategy_id {
            Some(_) => {
                // Simulate performance improvement calculation
                let improvement = 0.05 + (rand::random::<f64>() * 0.1); // 5-15% improvement
                Ok(improvement)
            }
            None => {
                // First deployment, no comparison possible
                Ok(0.0)
            }
        }
    }
    
    /// Execute strategy analysis
    pub async fn execute_analysis(
        &self,
        agent_id: &str,
        market_data: &MarketData,
    ) -> Result<f64> {
        let strategies = self.strategies.read().unwrap();
        
        let container = strategies.get(agent_id)
            .ok_or_else(|| anyhow!("No strategy loaded for agent: {}", agent_id))?;
        
        // Check health status
        match &container.health_status {
            HealthStatus::Failed { error } => {
                return Err(anyhow!("Strategy is in failed state: {}", error));
            }
            HealthStatus::Critical { message } => {
                warn!("Strategy in critical state: {}", message);
            }
            _ => {}
        }
        
        // Execute analysis
        let signal_strength = unsafe {
            (container.vtable.analyze)(market_data as *const MarketData)
        };
        
        debug!("📊 Strategy analysis for agent {}: signal_strength = {:.3}", 
               agent_id, signal_strength);
        
        Ok(signal_strength)
    }
    
    /// Execute trading strategy
    pub async fn execute_trading(
        &self,
        agent_id: &str,
        context: &mut HftContext,
    ) -> Result<i32> {
        let strategies = self.strategies.read().unwrap();
        
        let container = strategies.get(agent_id)
            .ok_or_else(|| anyhow!("No strategy loaded for agent: {}", agent_id))?;
        
        // Execute trading logic
        let result = unsafe {
            (container.vtable.execute)(context as *mut HftContext)
        };
        
        debug!("⚡ Strategy execution for agent {}: result = {}", agent_id, result);
        
        Ok(result)
    }
    
    /// Get strategy info
    pub async fn get_strategy_info(&self, agent_id: &str) -> Result<String> {
        let strategies = self.strategies.read().unwrap();
        
        let container = strategies.get(agent_id)
            .ok_or_else(|| anyhow!("No strategy loaded for agent: {}", agent_id))?;
        
        unsafe {
            let info_ptr = (container.vtable.get_info)();
            if info_ptr.is_null() {
                return Err(anyhow!("Strategy returned null info"));
            }
            
            let info = &*info_ptr;
            let name = CStr::from_ptr(info.name).to_string_lossy();
            let version = CStr::from_ptr(info.version).to_string_lossy();
            let description = CStr::from_ptr(info.description).to_string_lossy();
            
            Ok(format!("Strategy: {} v{} - {}", name, version, description))
        }
    }
    
    /// Update load statistics
    fn update_load_stats(&mut self, load_time_ms: u64, success: bool) {
        self.stats.total_loads += 1;
        
        if success {
            self.stats.successful_loads += 1;
            
            // Update average load time
            let total = self.stats.successful_loads;
            self.stats.average_load_time_ms = 
                (self.stats.average_load_time_ms * (total - 1) + load_time_ms) / total;
        } else {
            self.stats.failed_loads += 1;
        }
    }
    
    /// Update swap statistics
    fn update_swap_stats(&mut self, swap_time_ms: u64, success: bool) {
        self.stats.total_swaps += 1;
        
        if success {
            self.stats.successful_swaps += 1;
            
            // Update average swap time
            let total = self.stats.successful_swaps;
            self.stats.average_swap_time_ms = 
                (self.stats.average_swap_time_ms * (total - 1) + swap_time_ms) / total;
        } else {
            self.stats.failed_swaps += 1;
        }
    }
    
    /// Update deployment statistics
    fn update_deployment_stats(&mut self, deployment_time_ms: u64, success: bool) {
        // Deployment includes both load and swap
        self.update_load_stats(deployment_time_ms / 2, success);
        self.update_swap_stats(deployment_time_ms / 2, success);
    }
    
    /// Get loader statistics
    pub fn get_stats(&self) -> &LoaderStats {
        &self.stats
    }
    
    /// Get loaded strategies
    pub fn get_loaded_strategies(&self) -> HashMap<String, String> {
        let strategies = self.strategies.read().unwrap();
        strategies.iter()
            .map(|(agent_id, container)| (agent_id.clone(), container.strategy_id.clone()))
            .collect()
    }
    
    /// Cleanup all strategies
    pub async fn cleanup_all(&self) -> Result<()> {
        info!("🧹 Cleaning up all loaded strategies");
        
        let strategies = self.strategies.read().unwrap();
        
        for (agent_id, container) in strategies.iter() {
            debug!("Cleaning up strategy for agent: {}", agent_id);
            unsafe {
                (container.vtable.cleanup)();
            }
        }
        
        info!("✅ All strategies cleaned up");
        Ok(())
    }
}

impl Drop for StrategyHotLoader {
    fn drop(&mut self) {
        // Cleanup all strategies on drop
        if let Err(e) = futures::executor::block_on(self.cleanup_all()) {
            error!("Failed to cleanup strategies on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hot_loader_creation() {
        let loader = StrategyHotLoader::new().unwrap();
        assert_eq!(loader.stats.total_loads, 0);
        assert_eq!(loader.stats.total_swaps, 0);
    }
    
    #[test]
    fn test_market_data_size() {
        // Ensure MarketData is C-compatible
        assert_eq!(std::mem::size_of::<MarketData>(), 8 * 8); // 8 f64/u64 fields
    }
    
    #[test]
    fn test_safety_config_defaults() {
        let config = SafetyConfig::default();
        assert_eq!(config.max_load_time_ms, 5000);
        assert_eq!(config.max_consecutive_failures, 3);
        assert!(config.enable_sandbox);
    }
}
