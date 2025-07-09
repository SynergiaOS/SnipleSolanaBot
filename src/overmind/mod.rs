//! THE OVERMIND PROTOCOL v4.1 "MONOLITH"
//! 
//! All-Rust implementation of autonomous AI trading system
//! 
//! Architecture:
//! - Layer 3: Cortex (AI Swarm + Knowledge Graph + Optimization)
//! - Layer 4: Execution Engine (HFT + Jito)
//! - Layer 2: Data Ingestion (Helius + Market Data)

pub mod cortex;
pub mod swarm;
pub mod knowledge_graph;
pub mod optimization;
pub mod evolution;
pub mod ai_engine;
pub mod leaderboard;
pub mod genetic_modifier;
pub mod mutation_guard;
pub mod validation_protocol;

use anyhow::Result;

use tracing::{info, error};
use self::ai_engine::{AIEngine, AIEngineConfig};
use self::swarm::SwarmOrchestrator;
use self::evolution::EvolutionEngine;

/// THE OVERMIND PROTOCOL main orchestrator
pub struct OvermindProtocol {
    cortex: cortex::Cortex,
    ai_engine: Option<AIEngine>,
    swarm: SwarmOrchestrator,
    evolution: EvolutionEngine,
    running: bool,
}

impl OvermindProtocol {
    /// Initialize THE OVERMIND PROTOCOL
    pub async fn new() -> Result<Self> {
        info!("🧠 Initializing THE OVERMIND PROTOCOL v4.1 'MONOLITH'");
        
        let cortex = cortex::Cortex::new().await?;
        let swarm = SwarmOrchestrator::new().await?;
        let evolution = EvolutionEngine::new().await?;

        Ok(Self {
            cortex,
            ai_engine: None,
            swarm,
            evolution,
            running: false,
        })
    }
    
    /// Start THE OVERMIND PROTOCOL
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting THE OVERMIND PROTOCOL");
        
        self.running = true;
        
        // Start the cortex (AI brain)
        let cortex_handle = tokio::spawn(async move {
            // Cortex main loop will be implemented here
        });
        
        // Wait for shutdown signal
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("🛑 Shutdown signal received");
                self.running = false;
            }
            result = cortex_handle => {
                if let Err(e) = result {
                    error!("❌ Cortex error: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Stop THE OVERMIND PROTOCOL
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping THE OVERMIND PROTOCOL");
        self.running = false;
        Ok(())
    }

    /// Check if THE OVERMIND PROTOCOL is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Add entity to knowledge graph
    pub async fn add_entity(&self, entity: knowledge_graph::Entity) -> Result<()> {
        self.cortex.add_entity(entity).await
    }

    /// Search entities in knowledge graph
    pub async fn search_entities(&self, query: &str, limit: u64) -> Result<Vec<knowledge_graph::Entity>> {
        self.cortex.search_entities(query, limit).await
    }

    /// Initialize AI Engine with Candle
    pub async fn initialize_ai_engine(&mut self, config: Option<AIEngineConfig>) -> Result<()> {
        info!("🧠 Initializing AI Engine with Candle framework");

        let ai_config = config.unwrap_or_default();
        let mut ai_engine = AIEngine::new(ai_config).await?;
        ai_engine.initialize_model().await?;

        self.ai_engine = Some(ai_engine);
        info!("✅ AI Engine initialized successfully");
        Ok(())
    }

    /// Get AI Engine reference
    pub fn ai_engine(&self) -> Option<&AIEngine> {
        self.ai_engine.as_ref()
    }

    /// Check if AI Engine is available
    pub fn has_ai_engine(&self) -> bool {
        self.ai_engine.is_some()
    }

    /// Get Swarm Orchestrator reference
    pub fn swarm(&self) -> &SwarmOrchestrator {
        &self.swarm
    }

    /// Get Evolution Engine reference
    pub fn evolution(&self) -> &EvolutionEngine {
        &self.evolution
    }

    /// Get Cortex reference
    pub fn cortex(&self) -> &cortex::Cortex {
        &self.cortex
    }

    /// Initialize swarm with default agents
    pub async fn initialize_swarm(&self) -> Result<()> {
        info!("🤖 Initializing Swarm with default agents");

        let strategies = ["conservative", "aggressive", "momentum", "arbitrage", "experimental"];

        for strategy in strategies {
            match self.swarm.add_agent(strategy).await {
                Ok(agent_id) => {
                    info!("✅ Added {} agent: {}", strategy, agent_id);
                }
                Err(e) => {
                    error!("❌ Failed to add {} agent: {}", strategy, e);
                }
            }
        }

        info!("✅ Swarm initialized with {} agents", strategies.len());
        Ok(())
    }
}
