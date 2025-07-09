//! SwarmAgentic AI Implementation
//! 
//! Rust-native implementation of multi-agent swarm orchestration
//! inspired by OpenAI Swarm but optimized for trading

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, error, info};
use futures;
use sha2::{Sha256, Digest};

/// Agent function trait
#[async_trait]
pub trait AgentFunction: Send + Sync {
    async fn execute(&self, context: &AgentContext, args: serde_json::Value) -> Result<AgentResult>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

/// Agent context for function execution
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub agent_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Result of agent function execution
#[derive(Debug, Clone)]
pub struct AgentResult {
    pub value: serde_json::Value,
    pub transfer_to: Option<String>,
    pub context_updates: HashMap<String, serde_json::Value>,
}

/// Trading agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub strategy: String,
    pub instructions: String,
    pub risk_tolerance: f64,
    pub max_position_size: f64,
    pub functions: Vec<String>,
}

/// Individual agent candidate in the swarm
pub struct AgentCandidate {
    id: Uuid,
    config: AgentConfig,
    context: RwLock<AgentContext>,
    functions: HashMap<String, Box<dyn AgentFunction>>,
    performance_history: RwLock<Vec<f64>>,
}

impl std::fmt::Debug for AgentCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentCandidate")
            .field("id", &self.id)
            .field("config", &self.config)
            .field("functions_count", &self.functions.len())
            .finish()
    }
}

impl AgentCandidate {
    /// Create new agent candidate
    pub async fn new(strategy: &str) -> Result<Self> {
        let id = Uuid::new_v4();
        
        let config = match strategy {
            "conservative" => AgentConfig {
                name: "Conservative Trader".to_string(),
                strategy: strategy.to_string(),
                instructions: "Focus on low-risk, stable returns. Avoid high volatility trades.".to_string(),
                risk_tolerance: 0.2,
                max_position_size: 0.1,
                functions: vec!["analyze_sentiment".to_string(), "check_risk".to_string()],
            },
            "aggressive" => AgentConfig {
                name: "Aggressive Trader".to_string(),
                strategy: strategy.to_string(),
                instructions: "Maximize profits with higher risk tolerance. Look for high-yield opportunities.".to_string(),
                risk_tolerance: 0.8,
                max_position_size: 0.5,
                functions: vec!["momentum_analysis".to_string(), "volatility_trading".to_string()],
            },
            "momentum" => AgentConfig {
                name: "Momentum Trader".to_string(),
                strategy: strategy.to_string(),
                instructions: "Follow market trends and momentum. Enter positions based on strong directional moves.".to_string(),
                risk_tolerance: 0.6,
                max_position_size: 0.3,
                functions: vec!["trend_analysis".to_string(), "momentum_signals".to_string()],
            },
            "arbitrage" => AgentConfig {
                name: "Arbitrage Trader".to_string(),
                strategy: strategy.to_string(),
                instructions: "Find and exploit price differences across DEXes. Focus on risk-free profits.".to_string(),
                risk_tolerance: 0.1,
                max_position_size: 0.2,
                functions: vec!["price_comparison".to_string(), "arbitrage_detection".to_string()],
            },
            "experimental" => AgentConfig {
                name: "Experimental Trader".to_string(),
                strategy: strategy.to_string(),
                instructions: "Test new strategies and approaches. Higher risk for potential breakthrough discoveries.".to_string(),
                risk_tolerance: 0.9,
                max_position_size: 0.1,
                functions: vec!["experimental_signals".to_string(), "novel_strategies".to_string()],
            },
            _ => return Err(anyhow::anyhow!("Unknown strategy: {}", strategy)),
        };
        
        let context = AgentContext {
            variables: HashMap::new(),
            agent_id: id,
            timestamp: chrono::Utc::now(),
        };
        
        let mut functions: HashMap<String, Box<dyn AgentFunction>> = HashMap::new();
        
        // Register default functions
        functions.insert("analyze_sentiment".to_string(), Box::new(SentimentAnalysisFunction));
        functions.insert("check_risk".to_string(), Box::new(RiskAnalysisFunction));
        functions.insert("momentum_analysis".to_string(), Box::new(MomentumAnalysisFunction));
        functions.insert("volatility_trading".to_string(), Box::new(VolatilityTradingFunction));
        functions.insert("trend_analysis".to_string(), Box::new(TrendAnalysisFunction));
        functions.insert("momentum_signals".to_string(), Box::new(MomentumSignalsFunction));
        functions.insert("price_comparison".to_string(), Box::new(PriceComparisonFunction));
        functions.insert("arbitrage_detection".to_string(), Box::new(ArbitrageDetectionFunction));
        functions.insert("experimental_signals".to_string(), Box::new(ExperimentalSignalsFunction));
        functions.insert("novel_strategies".to_string(), Box::new(NovelStrategiesFunction));
        
        Ok(Self {
            id,
            config,
            context: RwLock::new(context),
            functions,
            performance_history: RwLock::new(Vec::new()),
        })
    }
    
    /// Get agent ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get agent name
    pub fn name(&self) -> &str {
        &self.config.name
    }
    
    /// Process trading signal
    pub async fn process_signal(&self, signal: serde_json::Value) -> Result<Option<serde_json::Value>> {
        debug!("🤖 Agent {} processing signal", self.config.name);
        
        // Update context with signal data
        {
            let mut context = self.context.write().await;
            context.variables.insert("signal".to_string(), signal.clone());
            context.timestamp = chrono::Utc::now();
        }
        
        // Execute relevant functions based on strategy
        let mut results = Vec::new();
        
        for function_name in &self.config.functions {
            if let Some(function) = self.functions.get(function_name) {
                let context = self.context.read().await;
                match function.execute(&context, signal.clone()).await {
                    Ok(result) => {
                        results.push(result);
                    }
                    Err(e) => {
                        error!("❌ Function {} failed: {}", function_name, e);
                    }
                }
            }
        }
        
        // Combine results and make trading decision
        if !results.is_empty() {
            let decision = self.make_trading_decision(results).await?;
            return Ok(Some(decision));
        }
        
        Ok(None)
    }
    
    /// Make trading decision based on function results
    async fn make_trading_decision(&self, results: Vec<AgentResult>) -> Result<serde_json::Value> {
        // Simple decision logic - can be enhanced with LLM integration
        let mut confidence = 0.0;
        let mut action = "hold";
        
        for result in results {
            if let Some(conf) = result.value.get("confidence").and_then(|v| v.as_f64()) {
                confidence += conf;
            }
            
            if let Some(act) = result.value.get("action").and_then(|v| v.as_str()) {
                if act == "buy" && confidence > 0.7 {
                    action = "buy";
                } else if act == "sell" && confidence < -0.7 {
                    action = "sell";
                }
            }
        }
        
        Ok(serde_json::json!({
            "agent_id": self.id,
            "agent_name": self.config.name,
            "action": action,
            "confidence": confidence,
            "timestamp": chrono::Utc::now()
        }))
    }
    
    /// Get performance score
    pub async fn get_performance_score(&self) -> Result<f64> {
        let history = self.performance_history.read().await;
        
        if history.is_empty() {
            return Ok(0.0);
        }
        
        // Calculate average performance over last 10 trades
        let recent_trades: Vec<f64> = history.iter().rev().take(10).cloned().collect();
        let avg_performance: f64 = recent_trades.iter().sum::<f64>() / recent_trades.len() as f64;
        
        Ok(avg_performance)
    }
    
    /// Record performance
    pub async fn record_performance(&self, score: f64) -> Result<()> {
        let mut history = self.performance_history.write().await;
        history.push(score);
        
        // Keep only last 100 records
        if history.len() > 100 {
            history.remove(0);
        }
        
        Ok(())
    }
}

/// Enhanced performance metrics for FAZA 11 Swarm Genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: Uuid,
    pub agent_name: String,
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_profit: f64,
    pub average_confidence: f64,
    pub last_active: chrono::DateTime<chrono::Utc>,
    pub performance_score: f64,
    pub risk_adjusted_return: f64,

    // FAZA 11 Enhanced Metrics
    pub roi: f64,                   // Return on Investment
    pub sharpe_ratio: f64,          // Risk-adjusted returns
    pub win_rate: f64,              // % Winning trades
    pub latency_p90: Duration,      // 90th percentile decision latency
    pub capital_efficiency: f64,    // Active capital percentage
    pub hotz_score: f64,            // Synthetic Hotz performance metric
}

/// Candidate status for lifecycle management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CandidateStatus {
    Booting,
    Active,
    Evolving,
    Paused,
    Failed,
    Terminated,
}

/// Enhanced SystemCandidate for FAZA 11 Swarm Genesis
pub struct SystemCandidate {
    pub id: Uuid,
    pub config_path: PathBuf,
    pub performance_metrics: AgentMetrics,
    pub configuration_hash: [u8; 32],
    pub status: CandidateStatus,
    pub instance_handle: Option<tokio::task::JoinHandle<()>>,

    // Legacy compatibility
    pub legacy_candidate: AgentCandidate,
}

impl std::fmt::Debug for SystemCandidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemCandidate")
            .field("id", &self.id)
            .field("config_path", &self.config_path)
            .field("status", &self.status)
            .field("hotz_score", &self.performance_metrics.hotz_score)
            .finish()
    }
}

impl Clone for SystemCandidate {
    fn clone(&self) -> Self {
        // Create a minimal legacy candidate for cloning
        let legacy_candidate = AgentCandidate {
            id: self.id,
            config: self.legacy_candidate.config.clone(),
            context: RwLock::new(AgentContext {
                variables: HashMap::new(),
                agent_id: self.id,
                timestamp: chrono::Utc::now(),
            }),
            functions: HashMap::new(), // Empty functions for clone
            performance_history: RwLock::new(Vec::new()),
        };

        Self {
            id: self.id,
            config_path: self.config_path.clone(),
            performance_metrics: self.performance_metrics.clone(),
            configuration_hash: self.configuration_hash,
            status: self.status.clone(),
            instance_handle: None, // Cannot clone JoinHandle
            legacy_candidate,
        }
    }
}

impl SystemCandidate {
    /// Create new SystemCandidate with enhanced capabilities
    pub async fn spawn(config_template: &str, strategy: &str) -> Result<Self> {
        let id = Uuid::new_v4();
        let config_dir = format!("/var/swarm/{}", id);
        let config_path = PathBuf::from(format!("{}/config.toml", config_dir));

        // Create swarm directory
        std::fs::create_dir_all(&config_dir)?;

        // Write configuration template
        std::fs::write(&config_path, config_template)?;

        // Calculate configuration hash
        let configuration_hash = Self::compute_config_hash(config_template);

        // Create legacy candidate for compatibility
        let legacy_candidate = AgentCandidate::new(strategy).await?;

        // Initialize enhanced metrics
        let performance_metrics = AgentMetrics {
            agent_id: id,
            agent_name: format!("{} Candidate", strategy),
            total_trades: 0,
            successful_trades: 0,
            total_profit: 0.0,
            average_confidence: 0.0,
            last_active: chrono::Utc::now(),
            performance_score: 0.0,
            risk_adjusted_return: 0.0,
            roi: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            latency_p90: Duration::from_millis(100), // Default 100ms
            capital_efficiency: 0.0,
            hotz_score: 50.0, // Default neutral score
        };

        // Spawn isolated instance
        let instance_handle = Some(tokio::spawn(async move {
            // Simulate agent instance running
            info!("🤖 SystemCandidate {} started in isolated environment", id);

            // Agent core loop would go here
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                // Agent processing logic...
            }
        }));

        Ok(SystemCandidate {
            id,
            config_path,
            performance_metrics,
            configuration_hash,
            status: CandidateStatus::Booting,
            instance_handle,
            legacy_candidate,
        })
    }

    /// Compute SHA256 hash of configuration
    fn compute_config_hash(config_content: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(config_content.as_bytes());
        hasher.finalize().into()
    }

    /// Update candidate status
    pub fn set_status(&mut self, status: CandidateStatus) {
        self.status = status;
        info!("🔄 SystemCandidate {} status changed to {:?}", self.id, self.status);
    }

    /// Calculate Hotz score based on performance metrics
    pub fn calculate_hotz_score(&mut self) {
        // Hotz philosophy: latency + efficiency + profitability
        let latency_score = if self.performance_metrics.latency_p90.as_millis() < 1000 {
            100.0 - (self.performance_metrics.latency_p90.as_millis() as f64 / 10.0)
        } else {
            0.0
        };

        let efficiency_score = self.performance_metrics.capital_efficiency * 100.0;
        let profit_score = self.performance_metrics.roi.max(0.0).min(100.0);

        self.performance_metrics.hotz_score = (latency_score * 0.4) + (efficiency_score * 0.3) + (profit_score * 0.3);
    }

    /// Terminate candidate instance
    pub async fn terminate(&mut self) -> Result<()> {
        if let Some(handle) = self.instance_handle.take() {
            handle.abort();
            info!("🛑 SystemCandidate {} terminated", self.id);
        }
        self.status = CandidateStatus::Terminated;
        Ok(())
    }
}

/// Communication message between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from_agent: Uuid,
    pub to_agent: Option<Uuid>, // None for broadcast
    pub message_type: MessageType,
    pub content: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    MarketSignal,
    TradingDecision,
    PerformanceUpdate,
    CoordinationRequest,
    StrategyShare,
}

/// Swarm orchestrator managing multiple agent candidates
pub struct SwarmOrchestrator {
    agents: RwLock<HashMap<Uuid, AgentCandidate>>,
    metrics: RwLock<HashMap<Uuid, AgentMetrics>>,
    message_queue: RwLock<Vec<AgentMessage>>,
    coordination_rules: RwLock<HashMap<String, f64>>,
    performance_threshold: f64,
    max_agents: usize,
}

impl SwarmOrchestrator {
    pub async fn new() -> Result<Self> {
        let mut coordination_rules = HashMap::new();

        // Default coordination rules
        coordination_rules.insert("min_consensus_threshold".to_string(), 0.6);
        coordination_rules.insert("max_position_overlap".to_string(), 0.3);
        coordination_rules.insert("performance_weight".to_string(), 0.7);
        coordination_rules.insert("diversity_weight".to_string(), 0.3);

        Ok(Self {
            agents: RwLock::new(HashMap::new()),
            metrics: RwLock::new(HashMap::new()),
            message_queue: RwLock::new(Vec::new()),
            coordination_rules: RwLock::new(coordination_rules),
            performance_threshold: 0.5,
            max_agents: 10,
        })
    }

    /// Add agent to swarm
    pub async fn add_agent(&self, strategy: &str) -> Result<Uuid> {
        let agent = AgentCandidate::new(strategy).await?;
        let agent_id = agent.id();

        // Initialize metrics with FAZA 11 enhanced fields
        let metrics = AgentMetrics {
            agent_id,
            agent_name: agent.config.name.clone(),
            total_trades: 0,
            successful_trades: 0,
            total_profit: 0.0,
            average_confidence: 0.0,
            last_active: chrono::Utc::now(),
            performance_score: 0.0,
            risk_adjusted_return: 0.0,
            // FAZA 11 Enhanced Metrics
            roi: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            latency_p90: Duration::from_millis(100),
            capital_efficiency: 0.0,
            hotz_score: 50.0, // Default neutral score
        };

        {
            let mut agents = self.agents.write().await;
            let mut agent_metrics = self.metrics.write().await;

            agents.insert(agent_id, agent);
            agent_metrics.insert(agent_id, metrics);
        }

        debug!("✅ Added agent {} to swarm", strategy);
        Ok(agent_id)
    }

    /// Remove underperforming agent
    pub async fn remove_agent(&self, agent_id: Uuid) -> Result<()> {
        let mut agents = self.agents.write().await;
        let mut metrics = self.metrics.write().await;

        agents.remove(&agent_id);
        metrics.remove(&agent_id);

        debug!("🗑️ Removed agent {} from swarm", agent_id);
        Ok(())
    }

    /// Process market signal through swarm
    pub async fn process_market_signal(&self, signal: serde_json::Value) -> Result<Vec<serde_json::Value>> {
        let agents = self.agents.read().await;
        let mut decisions = Vec::new();

        // Process signal through all agents in parallel
        let mut tasks = Vec::new();

        for (agent_id, agent) in agents.iter() {
            let signal_clone = signal.clone();
            let agent_id = *agent_id;

            let task = async move {
                match agent.process_signal(signal_clone).await {
                    Ok(Some(decision)) => Some((agent_id, decision)),
                    Ok(None) => None,
                    Err(e) => {
                        error!("❌ Agent {} failed to process signal: {}", agent_id, e);
                        None
                    }
                }
            };

            tasks.push(task);
        }

        // Wait for all agents to complete
        let results = futures::future::join_all(tasks).await;

        for result in results {
            if let Some((agent_id, decision)) = result {
                decisions.push(decision);
                self.update_agent_activity(agent_id).await?;
            }
        }

        // Apply coordination logic
        let coordinated_decisions = self.coordinate_decisions(decisions).await?;

        Ok(coordinated_decisions)
    }

    /// Coordinate decisions from multiple agents
    async fn coordinate_decisions(&self, decisions: Vec<serde_json::Value>) -> Result<Vec<serde_json::Value>> {
        if decisions.is_empty() {
            return Ok(vec![]);
        }

        let rules = self.coordination_rules.read().await;
        let consensus_threshold = rules.get("min_consensus_threshold").unwrap_or(&0.6);

        // Group decisions by action type
        let mut buy_decisions = Vec::new();
        let mut sell_decisions = Vec::new();
        let mut hold_decisions = Vec::new();

        for decision in &decisions {
            if let Some(action) = decision.get("action").and_then(|v| v.as_str()) {
                match action {
                    "buy" => buy_decisions.push(decision.clone()),
                    "sell" => sell_decisions.push(decision.clone()),
                    _ => hold_decisions.push(decision.clone()),
                }
            }
        }

        let total_decisions = decisions.len() as f64;
        let buy_consensus = buy_decisions.len() as f64 / total_decisions;
        let sell_consensus = sell_decisions.len() as f64 / total_decisions;

        // Apply consensus rules
        let mut coordinated = Vec::new();

        if buy_consensus >= *consensus_threshold {
            // Strong buy consensus - combine buy decisions
            let combined_confidence: f64 = buy_decisions.iter()
                .filter_map(|d| d.get("confidence").and_then(|v| v.as_f64()))
                .sum::<f64>() / buy_decisions.len() as f64;

            coordinated.push(serde_json::json!({
                "action": "buy",
                "confidence": combined_confidence,
                "consensus": buy_consensus,
                "participating_agents": buy_decisions.len(),
                "timestamp": chrono::Utc::now()
            }));
        } else if sell_consensus >= *consensus_threshold {
            // Strong sell consensus - combine sell decisions
            let combined_confidence: f64 = sell_decisions.iter()
                .filter_map(|d| d.get("confidence").and_then(|v| v.as_f64()))
                .sum::<f64>() / sell_decisions.len() as f64;

            coordinated.push(serde_json::json!({
                "action": "sell",
                "confidence": combined_confidence,
                "consensus": sell_consensus,
                "participating_agents": sell_decisions.len(),
                "timestamp": chrono::Utc::now()
            }));
        } else {
            // No strong consensus - return individual high-confidence decisions
            for decision in decisions {
                if let Some(confidence) = decision.get("confidence").and_then(|v| v.as_f64()) {
                    if confidence > 0.8 {
                        coordinated.push(decision);
                    }
                }
            }
        }

        Ok(coordinated)
    }

    /// Update agent activity timestamp
    async fn update_agent_activity(&self, agent_id: Uuid) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(agent_metrics) = metrics.get_mut(&agent_id) {
            agent_metrics.last_active = chrono::Utc::now();
        }
        Ok(())
    }

    /// Get swarm performance metrics
    pub async fn get_swarm_metrics(&self) -> Result<serde_json::Value> {
        let metrics = self.metrics.read().await;
        let agents = self.agents.read().await;

        let total_agents = agents.len();
        let active_agents = metrics.values()
            .filter(|m| chrono::Utc::now().signed_duration_since(m.last_active).num_minutes() < 60)
            .count();

        let avg_performance: f64 = if !metrics.is_empty() {
            metrics.values().map(|m| m.performance_score).sum::<f64>() / metrics.len() as f64
        } else {
            0.0
        };

        let total_trades: u64 = metrics.values().map(|m| m.total_trades).sum();
        let total_profit: f64 = metrics.values().map(|m| m.total_profit).sum();

        Ok(serde_json::json!({
            "total_agents": total_agents,
            "active_agents": active_agents,
            "average_performance": avg_performance,
            "total_trades": total_trades,
            "total_profit": total_profit,
            "coordination_rules": *self.coordination_rules.read().await,
            "timestamp": chrono::Utc::now()
        }))
    }

    /// Get individual agent metrics
    pub async fn get_agent_metrics(&self, agent_id: Uuid) -> Result<Option<AgentMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(&agent_id).cloned())
    }

    /// Update agent performance
    pub async fn update_agent_performance(&self, agent_id: Uuid, trade_result: f64, confidence: f64) -> Result<()> {
        let mut metrics = self.metrics.write().await;

        if let Some(agent_metrics) = metrics.get_mut(&agent_id) {
            agent_metrics.total_trades += 1;
            agent_metrics.total_profit += trade_result;

            if trade_result > 0.0 {
                agent_metrics.successful_trades += 1;
            }

            // Update average confidence
            let total_confidence = agent_metrics.average_confidence * (agent_metrics.total_trades - 1) as f64 + confidence;
            agent_metrics.average_confidence = total_confidence / agent_metrics.total_trades as f64;

            // Calculate performance score (success rate * average profit)
            let success_rate = agent_metrics.successful_trades as f64 / agent_metrics.total_trades as f64;
            let avg_profit = agent_metrics.total_profit / agent_metrics.total_trades as f64;
            agent_metrics.performance_score = success_rate * avg_profit;

            // Calculate risk-adjusted return (Sharpe-like ratio)
            agent_metrics.risk_adjusted_return = if agent_metrics.total_trades > 10 {
                // Simplified risk adjustment - can be enhanced
                agent_metrics.performance_score / (1.0 + (confidence.abs() - 0.5).abs())
            } else {
                agent_metrics.performance_score
            };
        }

        // Check if agent should be removed due to poor performance
        if let Some(agent_metrics) = metrics.get(&agent_id) {
            if agent_metrics.total_trades > 20 && agent_metrics.performance_score < self.performance_threshold {
                drop(metrics); // Release lock before calling remove_agent
                self.remove_agent(agent_id).await?;
            }
        }

        Ok(())
    }

    /// Send message between agents
    pub async fn send_message(&self, message: AgentMessage) -> Result<()> {
        let mut queue = self.message_queue.write().await;
        queue.push(message);

        // Keep only last 1000 messages
        if queue.len() > 1000 {
            queue.remove(0);
        }

        Ok(())
    }

    /// Get recent messages for an agent
    pub async fn get_messages_for_agent(&self, agent_id: Uuid, limit: usize) -> Result<Vec<AgentMessage>> {
        let queue = self.message_queue.read().await;

        let messages: Vec<AgentMessage> = queue.iter()
            .filter(|msg| msg.to_agent.is_none() || msg.to_agent == Some(agent_id))
            .rev()
            .take(limit)
            .cloned()
            .collect();

        Ok(messages)
    }

    /// Evolve swarm by adding new agents or removing poor performers
    pub async fn evolve_swarm(&self) -> Result<()> {
        let metrics = self.metrics.read().await;
        let agents_count = metrics.len();

        // Remove poor performers
        let poor_performers: Vec<Uuid> = metrics.iter()
            .filter(|(_, m)| m.total_trades > 20 && m.performance_score < self.performance_threshold)
            .map(|(id, _)| *id)
            .collect();

        drop(metrics);

        for agent_id in poor_performers {
            self.remove_agent(agent_id).await?;
        }

        // Add new agents if below max capacity
        if agents_count < self.max_agents {
            let strategies = ["conservative", "aggressive", "momentum", "arbitrage", "experimental"];
            let strategy = strategies[agents_count % strategies.len()];
            self.add_agent(strategy).await?;
        }

        Ok(())
    }

    /// Get best performing agent
    pub async fn get_best_agent(&self) -> Result<Option<(Uuid, AgentMetrics)>> {
        let metrics = self.metrics.read().await;

        let best = metrics.iter()
            .max_by(|(_, a), (_, b)| a.performance_score.partial_cmp(&b.performance_score).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, metrics)| (*id, metrics.clone()));

        Ok(best)
    }
}

// Agent function implementations
struct SentimentAnalysisFunction;
struct RiskAnalysisFunction;
struct MomentumAnalysisFunction;
struct VolatilityTradingFunction;
struct TrendAnalysisFunction;
struct MomentumSignalsFunction;
struct PriceComparisonFunction;
struct ArbitrageDetectionFunction;
struct ExperimentalSignalsFunction;
struct NovelStrategiesFunction;

// Implement AgentFunction for each function type
#[async_trait]
impl AgentFunction for SentimentAnalysisFunction {
    async fn execute(&self, _context: &AgentContext, signal: serde_json::Value) -> Result<AgentResult> {
        // Placeholder implementation
        Ok(AgentResult {
            value: serde_json::json!({"confidence": 0.5, "action": "hold"}),
            transfer_to: None,
            context_updates: HashMap::new(),
        })
    }
    
    fn name(&self) -> &str { "analyze_sentiment" }
    fn description(&self) -> &str { "Analyze market sentiment" }
}

// Similar implementations for other functions...
// (Will be expanded in next iteration)

macro_rules! impl_agent_function {
    ($struct_name:ident, $name:expr, $desc:expr) => {
        #[async_trait]
        impl AgentFunction for $struct_name {
            async fn execute(&self, _context: &AgentContext, _signal: serde_json::Value) -> Result<AgentResult> {
                Ok(AgentResult {
                    value: serde_json::json!({"confidence": 0.5, "action": "hold"}),
                    transfer_to: None,
                    context_updates: HashMap::new(),
                })
            }
            
            fn name(&self) -> &str { $name }
            fn description(&self) -> &str { $desc }
        }
    };
}

impl_agent_function!(RiskAnalysisFunction, "check_risk", "Analyze trading risk");
impl_agent_function!(MomentumAnalysisFunction, "momentum_analysis", "Analyze market momentum");
impl_agent_function!(VolatilityTradingFunction, "volatility_trading", "Execute volatility-based trades");
impl_agent_function!(TrendAnalysisFunction, "trend_analysis", "Analyze market trends");
impl_agent_function!(MomentumSignalsFunction, "momentum_signals", "Generate momentum signals");
impl_agent_function!(PriceComparisonFunction, "price_comparison", "Compare prices across DEXes");
impl_agent_function!(ArbitrageDetectionFunction, "arbitrage_detection", "Detect arbitrage opportunities");
impl_agent_function!(ExperimentalSignalsFunction, "experimental_signals", "Generate experimental signals");
impl_agent_function!(NovelStrategiesFunction, "novel_strategies", "Execute novel trading strategies");
