//! DYNAMIC AGENT ARCHITECTURE - FAZA 2 OPERACJI "FORGE"
//! 
//! Przebudowa architektury agentów dla dynamicznego ładowania strategii
//! Hot-swapping logic bez restartu monolitu

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::forge::hot_loader::{
    StrategyHotLoader, StrategyContainer, MarketData, HftContext, StrategyMetrics
};
use crate::forge::{CompiledArtifact, TheForge};

/// Dynamic Agent - agent z hot-swappable strategy logic
#[derive(Debug)]
pub struct DynamicAgent {
    /// Unique agent ID
    pub agent_id: String,
    
    /// Agent configuration
    pub config: DynamicAgentConfig,
    
    /// Hot loader for strategy swapping
    strategy_loader: Arc<RwLock<StrategyHotLoader>>,
    
    /// Current strategy container
    current_strategy: Arc<RwLock<Option<StrategyContainer>>>,
    
    /// Agent metrics
    metrics: Arc<RwLock<DynamicAgentMetrics>>,
    
    /// Communication channels
    command_tx: mpsc::UnboundedSender<AgentCommand>,
    command_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<AgentCommand>>>>,
    
    /// Market data feed
    market_data_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<MarketData>>>>,
    
    /// Execution context
    execution_context: Arc<RwLock<HftContext>>,
    
    /// Agent state
    state: Arc<RwLock<AgentState>>,
}

/// Dynamic Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAgentConfig {
    /// Agent name
    pub name: String,
    
    /// Agent type
    pub agent_type: AgentType,
    
    /// Strategy update interval
    pub strategy_update_interval: Duration,
    
    /// Performance monitoring interval
    pub monitoring_interval: Duration,
    
    /// Auto-evolution settings
    pub auto_evolution: AutoEvolutionConfig,
    
    /// Risk parameters
    pub risk_params: RiskParameters,
    
    /// Execution parameters
    pub execution_params: ExecutionParameters,
}

/// Agent type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Sentiment,
    Momentum,
    Arbitrage,
    MarketMaking,
    RiskManagement,
    Custom(String),
}

/// Auto-evolution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEvolutionConfig {
    /// Enable automatic strategy evolution
    pub enabled: bool,
    
    /// Performance threshold for triggering evolution
    pub performance_threshold: f64,
    
    /// Minimum time between evolutions
    pub min_evolution_interval: Duration,
    
    /// Maximum consecutive failed evolutions
    pub max_failed_evolutions: u32,
}

/// Risk parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskParameters {
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub stop_loss_threshold: f64,
    pub take_profit_threshold: f64,
    pub correlation_limit: f64,
}

/// Execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionParameters {
    pub max_execution_time_ms: u64,
    pub max_slippage: f64,
    pub min_fill_ratio: f64,
    pub order_timeout: Duration,
}

/// Agent metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DynamicAgentMetrics {
    pub total_decisions: u64,
    pub successful_decisions: u64,
    pub failed_decisions: u64,
    pub total_pnl: f64,
    pub current_drawdown: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub average_decision_time_ms: u64,
    pub strategy_swaps: u64,
    pub last_evolution: Option<chrono::DateTime<chrono::Utc>>,
    pub evolution_success_rate: f64,
}

/// Agent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Initializing,
    Active,
    Paused,
    Evolving,
    Failed { error: String },
    Shutdown,
}

/// Agent commands
#[derive(Debug, Clone)]
pub enum AgentCommand {
    Start,
    Stop,
    Pause,
    Resume,
    UpdateStrategy { artifact: CompiledArtifact },
    TriggerEvolution,
    UpdateConfig { config: DynamicAgentConfig },
    GetMetrics { response_tx: mpsc::UnboundedSender<DynamicAgentMetrics> },
    GetState { response_tx: mpsc::UnboundedSender<AgentState> },
    Shutdown,
}

impl Default for DynamicAgentConfig {
    fn default() -> Self {
        Self {
            name: "DefaultAgent".to_string(),
            agent_type: AgentType::Sentiment,
            strategy_update_interval: Duration::from_secs(60),
            monitoring_interval: Duration::from_secs(10),
            auto_evolution: AutoEvolutionConfig {
                enabled: true,
                performance_threshold: 0.05, // 5% improvement threshold
                min_evolution_interval: Duration::from_secs(6 * 3600),
                max_failed_evolutions: 3,
            },
            risk_params: RiskParameters {
                max_position_size: 10000.0,
                max_daily_loss: 1000.0,
                stop_loss_threshold: 0.02,
                take_profit_threshold: 0.05,
                correlation_limit: 0.7,
            },
            execution_params: ExecutionParameters {
                max_execution_time_ms: 100,
                max_slippage: 0.001,
                min_fill_ratio: 0.95,
                order_timeout: Duration::from_secs(30),
            },
        }
    }
}

impl DynamicAgent {
    /// Create new dynamic agent
    pub async fn new(
        agent_id: String,
        config: DynamicAgentConfig,
        strategy_loader: Arc<RwLock<StrategyHotLoader>>,
    ) -> Result<Self> {
        info!("🧬 Creating dynamic agent: {}", agent_id);
        
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (market_data_tx, market_data_rx) = mpsc::unbounded_channel();
        
        // Initialize execution context
        let agent_id_cstr = std::ffi::CString::new(agent_id.clone())?;
        let execution_context = HftContext {
            agent_id: agent_id_cstr.as_ptr(),
            position_size: 0.0,
            available_balance: config.risk_params.max_position_size,
            max_position_size: config.risk_params.max_position_size,
            risk_limit: config.risk_params.max_daily_loss,
            execution_callback: Some(execution_callback),
        };
        
        Ok(Self {
            agent_id,
            config,
            strategy_loader,
            current_strategy: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(DynamicAgentMetrics::default())),
            command_tx,
            command_rx: Arc::new(RwLock::new(Some(command_rx))),
            market_data_rx: Arc::new(RwLock::new(Some(market_data_rx))),
            execution_context: Arc::new(RwLock::new(execution_context)),
            state: Arc::new(RwLock::new(AgentState::Initializing)),
        })
    }
    
    /// Start agent execution loop
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting dynamic agent: {}", self.agent_id);
        
        // Update state
        {
            let mut state = self.state.write().unwrap();
            *state = AgentState::Active;
        }
        
        // Start command processing loop
        let command_rx = {
            let mut rx_guard = self.command_rx.write().unwrap();
            rx_guard.take().ok_or_else(|| anyhow!("Command receiver already taken"))?
        };

        let agent_id = self.agent_id.clone();
        let state = self.state.clone();
        let metrics = self.metrics.clone();
        let current_strategy = self.current_strategy.clone();

        tokio::spawn(async move {
            Self::command_loop(
                agent_id,
                command_rx,
                state,
                metrics,
                current_strategy,
            ).await;
        });
        
        // Start market data processing loop
        let market_data_rx = {
            let mut rx_guard = self.market_data_rx.write().unwrap();
            rx_guard.take().ok_or_else(|| anyhow!("Market data receiver already taken"))?
        };

        let agent_id = self.agent_id.clone();
        let state = self.state.clone();
        let metrics = self.metrics.clone();
        let current_strategy = self.current_strategy.clone();

        tokio::spawn(async move {
            Self::market_data_loop(
                agent_id,
                market_data_rx,
                state,
                metrics,
                current_strategy,
            ).await;
        });
        
        info!("✅ Dynamic agent started: {}", self.agent_id);
        Ok(())
    }
    
    /// Command processing loop
    async fn command_loop(
        agent_id: String,
        mut command_rx: mpsc::UnboundedReceiver<AgentCommand>,
        state: Arc<RwLock<AgentState>>,
        metrics: Arc<RwLock<DynamicAgentMetrics>>,
        current_strategy: Arc<RwLock<Option<StrategyContainer>>>,
    ) {
        info!("🔄 Starting command loop for agent: {}", agent_id);
        
        while let Some(command) = command_rx.recv().await {
            match command {
                AgentCommand::Start => {
                    info!("📡 Agent {} received START command", agent_id);
                    let mut state_guard = state.write().unwrap();
                    *state_guard = AgentState::Active;
                }
                
                AgentCommand::Stop => {
                    info!("⏹️ Agent {} received STOP command", agent_id);
                    let mut state_guard = state.write().unwrap();
                    *state_guard = AgentState::Paused;
                }
                
                AgentCommand::UpdateStrategy { artifact } => {
                    info!("🔄 Agent {} updating strategy: {}", agent_id, artifact.strategy_id);
                    
                    // TODO: Implement strategy hot-swapping
                    // This will be implemented in the next step
                    
                    let mut metrics_guard = metrics.write().unwrap();
                    metrics_guard.strategy_swaps += 1;
                }
                
                AgentCommand::TriggerEvolution => {
                    info!("🧬 Agent {} triggering evolution", agent_id);
                    let mut state_guard = state.write().unwrap();
                    *state_guard = AgentState::Evolving;
                    
                    // TODO: Implement evolution trigger
                    // This will integrate with TheForge
                }
                
                AgentCommand::GetMetrics { response_tx } => {
                    let metrics_guard = metrics.read().unwrap();
                    let _ = response_tx.send(metrics_guard.clone());
                }
                
                AgentCommand::GetState { response_tx } => {
                    let state_guard = state.read().unwrap();
                    let _ = response_tx.send(state_guard.clone());
                }
                
                AgentCommand::Shutdown => {
                    info!("🛑 Agent {} shutting down", agent_id);
                    let mut state_guard = state.write().unwrap();
                    *state_guard = AgentState::Shutdown;
                    break;
                }
                
                _ => {
                    debug!("📨 Agent {} received command: {:?}", agent_id, command);
                }
            }
        }
        
        info!("✅ Command loop ended for agent: {}", agent_id);
    }
    
    /// Market data processing loop
    async fn market_data_loop(
        agent_id: String,
        mut market_data_rx: mpsc::UnboundedReceiver<MarketData>,
        state: Arc<RwLock<AgentState>>,
        metrics: Arc<RwLock<DynamicAgentMetrics>>,
        current_strategy: Arc<RwLock<Option<StrategyContainer>>>,
    ) {
        info!("📊 Starting market data loop for agent: {}", agent_id);
        
        while let Some(market_data) = market_data_rx.recv().await {
            // Check if agent is active
            {
                let state_guard = state.read().unwrap();
                match *state_guard {
                    AgentState::Active => {},
                    _ => continue,
                }
            }
            
            // Process market data with current strategy
            let decision_start = Instant::now();
            
            let strategy_result = {
                let strategy_guard = current_strategy.read().unwrap();
                if let Some(strategy) = strategy_guard.as_ref() {
                    // Execute strategy analysis
                    let signal_strength = unsafe {
                        (strategy.vtable.analyze)(&market_data as *const MarketData)
                    };
                    
                    Some((signal_strength, strategy.strategy_id.clone()))
                } else {
                    None
                }
            };
            
            if let Some((signal_strength, strategy_id)) = strategy_result {
                let decision_time = decision_start.elapsed().as_millis() as u64;
                
                debug!("📊 Agent {} strategy {} signal: {:.3} ({}ms)", 
                       agent_id, strategy_id, signal_strength, decision_time);
                
                // Update metrics
                {
                    let mut metrics_guard = metrics.write().unwrap();
                    metrics_guard.total_decisions += 1;
                    
                    // Update average decision time
                    let total = metrics_guard.total_decisions;
                    metrics_guard.average_decision_time_ms = 
                        (metrics_guard.average_decision_time_ms * (total - 1) + decision_time) / total;
                    
                    // TODO: Update other metrics based on signal strength and execution results
                }
            }
        }
        
        info!("✅ Market data loop ended for agent: {}", agent_id);
    }
    
    /// Send command to agent
    pub async fn send_command(&self, command: AgentCommand) -> Result<()> {
        self.command_tx.send(command)
            .map_err(|e| anyhow!("Failed to send command: {}", e))?;
        Ok(())
    }
    
    /// Get agent metrics
    pub async fn get_metrics(&self) -> Result<DynamicAgentMetrics> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.send_command(AgentCommand::GetMetrics { response_tx: tx }).await?;
        
        rx.recv().await.ok_or_else(|| anyhow!("Failed to receive metrics"))
    }
    
    /// Get agent state
    pub async fn get_state(&self) -> Result<AgentState> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.send_command(AgentCommand::GetState { response_tx: tx }).await?;
        
        rx.recv().await.ok_or_else(|| anyhow!("Failed to receive state"))
    }
    
    /// Update strategy (hot-swap)
    pub async fn update_strategy(&self, artifact: CompiledArtifact) -> Result<()> {
        info!("🔄 Hot-swapping strategy for agent: {}", self.agent_id);
        self.send_command(AgentCommand::UpdateStrategy { artifact }).await
    }
    
    /// Trigger evolution
    pub async fn trigger_evolution(&self) -> Result<()> {
        info!("🧬 Triggering evolution for agent: {}", self.agent_id);
        self.send_command(AgentCommand::TriggerEvolution).await
    }
}

/// Execution callback for strategy
unsafe extern "C" fn execution_callback(
    agent_id: *const std::os::raw::c_char,
    position_size: f64,
    price: f64,
) -> i32 {
    if agent_id.is_null() {
        return -1;
    }
    
    let agent_id_str = std::ffi::CStr::from_ptr(agent_id).to_string_lossy();
    
    // TODO: Implement actual execution logic
    // For now, just log the execution request
    println!("⚡ Execution request: agent={}, size={:.2}, price={:.2}", 
             agent_id_str, position_size, price);
    
    // Simulate successful execution
    0
}

/// Create dynamic agent from configuration
pub async fn create_dynamic_agent(
    agent_type: AgentType,
    config: Option<DynamicAgentConfig>,
    strategy_loader: Arc<RwLock<StrategyHotLoader>>,
) -> Result<DynamicAgent> {
    let agent_id = Uuid::new_v4().to_string();
    
    let mut agent_config = config.unwrap_or_default();
    agent_config.agent_type = agent_type.clone();
    agent_config.name = format!("{:?}Agent_{}", agent_type, &agent_id[..8]);
    
    DynamicAgent::new(agent_id, agent_config, strategy_loader).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::hot_loader::StrategyHotLoader;
    
    #[tokio::test]
    async fn test_dynamic_agent_creation() {
        let strategy_loader = Arc::new(RwLock::new(StrategyHotLoader::new().unwrap()));
        
        let agent = create_dynamic_agent(
            AgentType::Sentiment,
            None,
            strategy_loader,
        ).await.unwrap();
        
        assert!(agent.agent_id.len() > 0);
        assert!(matches!(agent.config.agent_type, AgentType::Sentiment));
    }
    
    #[tokio::test]
    async fn test_agent_command_processing() {
        let strategy_loader = Arc::new(RwLock::new(StrategyHotLoader::new().unwrap()));
        
        let agent = create_dynamic_agent(
            AgentType::Momentum,
            None,
            strategy_loader,
        ).await.unwrap();
        
        // Start agent
        agent.start().await.unwrap();
        
        // Test state changes
        agent.send_command(AgentCommand::Start).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let state = agent.get_state().await.unwrap();
        assert!(matches!(state, AgentState::Active));
        
        // Test metrics
        let metrics = agent.get_metrics().await.unwrap();
        assert_eq!(metrics.total_decisions, 0);
    }
}
