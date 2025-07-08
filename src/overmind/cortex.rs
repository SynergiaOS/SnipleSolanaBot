//! THE OVERMIND CORTEX - Layer 3 Implementation
//! 
//! The brain of THE OVERMIND PROTOCOL, containing:
//! - SwarmAgentic Orchestrator
//! - Knowledge Graph
//! - Evolution Engine
//! - Performance Optimization

use anyhow::Result;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, debug, error};
use dashmap::DashMap;
use uuid::Uuid;

use super::swarm::{SwarmOrchestrator, AgentCandidate};
use super::knowledge_graph::KnowledgeGraph;
use super::optimization::DataFlywheel;
use super::evolution::EvolutionEngine;

/// THE OVERMIND CORTEX - Main AI brain
pub struct Cortex {
    /// Swarm orchestrator managing agent candidates
    swarm: Arc<SwarmOrchestrator>,
    
    /// Knowledge graph for long-term memory
    knowledge_graph: Arc<KnowledgeGraph>,
    
    /// Data flywheel for model optimization
    data_flywheel: Arc<DataFlywheel>,
    
    /// Evolution engine for system improvement
    evolution_engine: Arc<EvolutionEngine>,
    
    /// Active agent candidates
    candidates: Arc<DashMap<Uuid, AgentCandidate>>,
    
    /// Performance leaderboard
    leaderboard: Arc<RwLock<Vec<(Uuid, f64)>>>,
}

impl Cortex {
    /// Create new Cortex instance
    pub async fn new() -> Result<Self> {
        info!("🧠 Initializing THE OVERMIND CORTEX");
        
        // Initialize components
        let swarm = Arc::new(SwarmOrchestrator::new().await?);
        let knowledge_graph = Arc::new(KnowledgeGraph::new().await?);
        let data_flywheel = Arc::new(DataFlywheel::new().await?);
        let evolution_engine = Arc::new(EvolutionEngine::new().await?);
        
        let candidates = Arc::new(DashMap::new());
        let leaderboard = Arc::new(RwLock::new(Vec::new()));
        
        // Initialize default agent candidates
        let cortex = Self {
            swarm,
            knowledge_graph,
            data_flywheel,
            evolution_engine,
            candidates,
            leaderboard,
        };
        
        cortex.initialize_candidates().await?;
        
        Ok(cortex)
    }
    
    /// Initialize default agent candidates
    async fn initialize_candidates(&self) -> Result<()> {
        info!("🤖 Initializing agent candidates");
        
        // Create 5 initial candidates with different strategies
        let strategies = vec![
            "conservative",
            "aggressive", 
            "momentum",
            "arbitrage",
            "experimental"
        ];
        
        for strategy in strategies {
            let candidate = AgentCandidate::new(strategy).await?;
            let id = candidate.id();
            self.candidates.insert(id, candidate);
            debug!("✅ Created candidate: {} ({})", strategy, id);
        }
        
        Ok(())
    }
    
    /// Main cortex processing loop
    pub async fn run(&self) -> Result<()> {
        info!("🚀 Starting CORTEX main loop");
        
        let mut evolution_interval = tokio::time::interval(
            tokio::time::Duration::from_secs(3600) // Evolve every hour
        );
        
        let mut performance_interval = tokio::time::interval(
            tokio::time::Duration::from_secs(300) // Check performance every 5 minutes
        );
        
        loop {
            tokio::select! {
                _ = evolution_interval.tick() => {
                    if let Err(e) = self.evolution_cycle().await {
                        error!("❌ Evolution cycle failed: {}", e);
                    }
                }
                
                _ = performance_interval.tick() => {
                    if let Err(e) = self.performance_evaluation().await {
                        error!("❌ Performance evaluation failed: {}", e);
                    }
                }
                
                // Handle shutdown
                _ = tokio::signal::ctrl_c() => {
                    info!("🛑 CORTEX shutdown requested");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Run evolution cycle
    async fn evolution_cycle(&self) -> Result<()> {
        info!("🧬 Starting evolution cycle");
        
        // Get current leaderboard
        let leaderboard = self.leaderboard.read().await;
        
        if leaderboard.is_empty() {
            debug!("📊 No performance data yet, skipping evolution");
            return Ok(());
        }
        
        // Find best and worst performers
        let best_candidate = leaderboard.first().unwrap();
        let worst_candidate = leaderboard.last().unwrap();
        
        info!("🏆 Best performer: {} (score: {:.4})", best_candidate.0, best_candidate.1);
        info!("📉 Worst performer: {} (score: {:.4})", worst_candidate.0, worst_candidate.1);
        
        // Evolve worst performers
        self.evolution_engine.evolve_candidate(worst_candidate.0).await?;
        
        Ok(())
    }
    
    /// Evaluate candidate performance
    async fn performance_evaluation(&self) -> Result<()> {
        debug!("📊 Evaluating candidate performance");
        
        let mut scores = Vec::new();
        
        // Collect performance scores from all candidates
        for candidate_ref in self.candidates.iter() {
            let candidate = candidate_ref.value();
            let score = candidate.get_performance_score().await?;
            scores.push((candidate.id(), score));
        }
        
        // Sort by performance (descending)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Update leaderboard
        let mut leaderboard = self.leaderboard.write().await;
        *leaderboard = scores;
        
        Ok(())
    }
    
    /// Get current best candidate
    pub async fn get_best_candidate(&self) -> Option<Uuid> {
        let leaderboard = self.leaderboard.read().await;
        leaderboard.first().map(|(id, _)| *id)
    }
    
    /// Process trading signal through swarm
    pub async fn process_signal(&self, signal: serde_json::Value) -> Result<Option<serde_json::Value>> {
        // Get best performing candidate
        if let Some(best_id) = self.get_best_candidate().await {
            if let Some(candidate) = self.candidates.get(&best_id) {
                return candidate.process_signal(signal).await;
            }
        }
        
        // Fallback to first available candidate
        if let Some(candidate_ref) = self.candidates.iter().next() {
            return candidate_ref.value().process_signal(signal).await;
        }
        
        Ok(None)
    }

    /// Add entity to knowledge graph
    pub async fn add_entity(&self, entity: super::knowledge_graph::Entity) -> Result<()> {
        self.knowledge_graph.upsert_entity(entity).await
    }

    /// Search entities in knowledge graph
    pub async fn search_entities(&self, query: &str, limit: u64) -> Result<Vec<super::knowledge_graph::Entity>> {
        self.knowledge_graph.search_entities(query, limit).await
    }

    /// Get agent count
    pub async fn get_agent_count(&self) -> usize {
        self.candidates.len()
    }

    /// Get all agents (returns agent info, not full objects)
    pub async fn get_all_agents(&self) -> Result<Vec<(Uuid, String)>> {
        let agents: Vec<(Uuid, String)> = self.candidates
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().name().to_string()))
            .collect();
        Ok(agents)
    }

    /// Get knowledge graph reference
    pub fn knowledge_graph(&self) -> &KnowledgeGraph {
        &self.knowledge_graph
    }
}
