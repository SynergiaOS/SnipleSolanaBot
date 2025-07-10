//! DYNAMIC AGENT SYSTEM - FAZA 2 OPERACJI "FORGE"
//! 
//! Przebudowana architektura agentów z hot-swappable strategy logic
//! Zero-downtime evolution through dynamic loading

pub mod dynamic_agent;
pub mod runtime_loader;

pub use dynamic_agent::{
    DynamicAgent, DynamicAgentConfig, DynamicAgentMetrics, AgentType, AgentState, AgentCommand,
    AutoEvolutionConfig, RiskParameters, ExecutionParameters, create_dynamic_agent
};
pub use runtime_loader::{
    RuntimeModuleLoader, CachedArtifact, LoadingMetrics, LoaderConfig, CacheStats, create_runtime_loader
};

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, warn, error};

use crate::forge::hot_loader::StrategyHotLoader;
use crate::forge::{TheForge, CompiledArtifact};

/// Agent Manager - zarządza rojem dynamic agents
#[derive(Debug)]
pub struct AgentManager {
    /// Registered agents
    agents: Arc<RwLock<HashMap<String, DynamicAgent>>>,
    
    /// Strategy hot loader
    strategy_loader: Arc<RwLock<StrategyHotLoader>>,
    
    /// FORGE integration for evolution
    forge: Option<Arc<RwLock<TheForge>>>,
    
    /// Manager metrics
    metrics: Arc<RwLock<AgentManagerMetrics>>,
}

/// Agent Manager metrics
#[derive(Debug, Default, Clone)]
pub struct AgentManagerMetrics {
    pub total_agents: u64,
    pub active_agents: u64,
    pub evolving_agents: u64,
    pub failed_agents: u64,
    pub total_strategy_swaps: u64,
    pub successful_evolutions: u64,
    pub failed_evolutions: u64,
}

impl AgentManager {
    /// Create new agent manager
    pub async fn new(
        strategy_loader: Arc<RwLock<StrategyHotLoader>>,
        forge: Option<Arc<RwLock<TheForge>>>,
    ) -> Result<Self> {
        info!("🧬 Initializing Agent Manager with dynamic architecture");
        
        Ok(Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            strategy_loader,
            forge,
            metrics: Arc::new(RwLock::new(AgentManagerMetrics::default())),
        })
    }
    
    /// Create and register new dynamic agent
    pub async fn create_agent(
        &self,
        agent_type: AgentType,
        config: Option<DynamicAgentConfig>,
    ) -> Result<String> {
        let agent = create_dynamic_agent(
            agent_type,
            config,
            self.strategy_loader.clone(),
        ).await?;
        
        let agent_id = agent.agent_id.clone();
        
        // Start the agent
        agent.start().await?;
        
        // Register agent
        {
            let mut agents = self.agents.write().unwrap();
            agents.insert(agent_id.clone(), agent);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_agents += 1;
            metrics.active_agents += 1;
        }
        
        info!("✅ Created and registered dynamic agent: {}", agent_id);
        Ok(agent_id)
    }
    
    /// Get agent by ID (returns reference to avoid cloning)
    pub async fn get_agent(&self, agent_id: &str) -> bool {
        let agents = self.agents.read().unwrap();
        agents.contains_key(agent_id)
    }
    
    /// List all agents
    pub async fn list_agents(&self) -> Vec<String> {
        let agents = self.agents.read().unwrap();
        agents.keys().cloned().collect()
    }
    
    /// Update agent strategy (hot-swap)
    pub async fn update_agent_strategy(
        &self,
        agent_id: &str,
        artifact: CompiledArtifact,
    ) -> Result<()> {
        let agent_exists = {
            let agents = self.agents.read().unwrap();
            agents.contains_key(agent_id)
        };
        
        if agent_exists {
            // Agent exists, update strategy
            let agents = self.agents.read().unwrap();
            if let Some(agent) = agents.get(agent_id) {
                agent.update_strategy(artifact).await?;
            }
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.total_strategy_swaps += 1;
            }
            
            info!("🔄 Updated strategy for agent: {}", agent_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", agent_id))
        }
    }
    
    /// Trigger evolution for agent
    pub async fn evolve_agent(&self, agent_id: &str) -> Result<()> {
        if let Some(forge) = &self.forge {
            info!("🧬 Triggering evolution for agent: {}", agent_id);
            
            // Update agent state to evolving
            if self.get_agent(agent_id).await {
                // Agent exists, trigger evolution
                let agents = self.agents.read().unwrap();
                if let Some(agent) = agents.get(agent_id) {
                    agent.trigger_evolution().await?;
                }
            }
            
            // Trigger FORGE evolution
            let mut forge_guard = forge.write().unwrap();
            match forge_guard.evolve_strategy(agent_id).await {
                Ok(evolution_result) => {
                    info!("✅ Evolution completed for agent {}: improvement {:.2}%", 
                          agent_id, evolution_result.performance_improvement * 100.0);
                    
                    // Update metrics
                    {
                        let mut metrics = self.metrics.write().unwrap();
                        metrics.successful_evolutions += 1;
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    error!("❌ Evolution failed for agent {}: {}", agent_id, e);
                    
                    // Update metrics
                    {
                        let mut metrics = self.metrics.write().unwrap();
                        metrics.failed_evolutions += 1;
                    }
                    
                    Err(e)
                }
            }
        } else {
            Err(anyhow::anyhow!("FORGE not available for evolution"))
        }
    }
    
    /// Get agent metrics
    pub async fn get_agent_metrics(&self, agent_id: &str) -> Result<DynamicAgentMetrics> {
        let agent_exists = {
            let agents = self.agents.read().unwrap();
            agents.contains_key(agent_id)
        };
        
        if agent_exists {
            // Agent exists, get metrics
            let agents = self.agents.read().unwrap();
            if let Some(agent) = agents.get(agent_id) {
                agent.get_metrics().await
            } else {
                Err(anyhow::anyhow!("Agent not found: {}", agent_id))
            }
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", agent_id))
        }
    }
    
    /// Get manager metrics
    pub async fn get_manager_metrics(&self) -> AgentManagerMetrics {
        // Update active agents count
        let active_count = {
            let agents = self.agents.read().unwrap();
            let mut active = 0;
            let mut evolving = 0;
            let mut failed = 0;
            
            // For now, just count total agents as active
            // In real implementation, would check each agent's state
            active = agents.len() as u64;
            
            (active, evolving, failed)
        };
        
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.active_agents = active_count.0;
            metrics.evolving_agents = active_count.1;
            metrics.failed_agents = active_count.2;
            metrics.clone()
        }
    }
    
    /// Remove agent
    pub async fn remove_agent(&self, agent_id: &str) -> Result<()> {
        let agent = {
            let mut agents = self.agents.write().unwrap();
            agents.remove(agent_id)
        };
        
        if let Some(agent) = agent {
            // Shutdown agent
            agent.send_command(AgentCommand::Shutdown).await?;
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().unwrap();
                metrics.total_agents = metrics.total_agents.saturating_sub(1);
                metrics.active_agents = metrics.active_agents.saturating_sub(1);
            }
            
            info!("🗑️ Removed agent: {}", agent_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", agent_id))
        }
    }
    
    /// Shutdown all agents
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("🛑 Shutting down all agents");
        
        let agent_ids: Vec<String> = {
            let agents = self.agents.read().unwrap();
            agents.keys().cloned().collect()
        };
        
        for agent_id in agent_ids {
            if let Err(e) = self.remove_agent(&agent_id).await {
                warn!("Failed to shutdown agent {}: {}", agent_id, e);
            }
        }
        
        info!("✅ All agents shutdown completed");
        Ok(())
    }
}

/// Initialize agent manager with default configuration
pub async fn init_agent_manager(
    strategy_loader: Arc<RwLock<StrategyHotLoader>>,
    forge: Option<Arc<RwLock<TheForge>>>,
) -> Result<AgentManager> {
    AgentManager::new(strategy_loader, forge).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::hot_loader::StrategyHotLoader;
    
    #[tokio::test]
    async fn test_agent_manager_creation() {
        let strategy_loader = Arc::new(RwLock::new(StrategyHotLoader::new().unwrap()));
        let manager = AgentManager::new(strategy_loader, None).await.unwrap();
        
        let metrics = manager.get_manager_metrics().await;
        assert_eq!(metrics.total_agents, 0);
    }
    
    #[tokio::test]
    async fn test_agent_creation_and_management() {
        let strategy_loader = Arc::new(RwLock::new(StrategyHotLoader::new().unwrap()));
        let manager = AgentManager::new(strategy_loader, None).await.unwrap();
        
        // Create agent
        let agent_id = manager.create_agent(AgentType::Sentiment, None).await.unwrap();
        assert!(!agent_id.is_empty());
        
        // Check metrics
        let metrics = manager.get_manager_metrics().await;
        assert_eq!(metrics.total_agents, 1);
        
        // Get agent
        let agent_exists = manager.get_agent(&agent_id).await;
        assert!(agent_exists);
        
        // Remove agent
        manager.remove_agent(&agent_id).await.unwrap();
        
        let metrics = manager.get_manager_metrics().await;
        assert_eq!(metrics.total_agents, 0);
    }
}
