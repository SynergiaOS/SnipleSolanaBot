//! THE OVERMIND PROTOCOL v4.1 "MONOLITH"
//!
//! All-Rust implementation of autonomous AI trading system for Solana blockchain
//!
//! This library provides a comprehensive trading system with the following features:
//! - THE OVERMIND CORTEX: AI Swarm + Knowledge Graph + Evolution
//! - Real-time market data ingestion with Helius integration
//! - Advanced MEV strategies and HFT execution
//! - Risk management and portfolio optimization
//! - Multi-wallet support with secure key management
//! - Continuous model optimization with NVIDIA Data Flywheel

#![allow(clippy::all)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(private_interfaces)]
#![allow(dead_code)]
#![allow(unused_variables)]

pub mod config;
pub mod models;
pub mod modules;
pub mod overmind;
pub mod geohot;
pub mod pheromind;
pub mod forge;
pub mod nexus;
pub mod agents;

// Re-export commonly used items
pub use config::Config;
pub use modules::ai_connector::AIConnector;
pub use modules::hft_engine::HftEngine;

// THE OVERMIND PROTOCOL v4.1 exports
pub use overmind::OvermindProtocol;
pub use overmind::cortex::Cortex;
pub use overmind::swarm::{AgentCandidate, SwarmOrchestrator};
pub use overmind::knowledge_graph::KnowledgeGraph;
pub use overmind::optimization::DataFlywheel;
pub use overmind::evolution::EvolutionEngine;

// GEOHOT CORE v4.4 exports - Pure Rust Implementation
pub use geohot::{GeohoteCore, GeohoteConfig, TradingDecision, GeohoteMetrics};
pub use geohot::ghost_protocol::{GhostFetcher, GhostConfig, GhostResponse};
pub use geohot::helius_stream::{HeliusStream, HeliusFilter, TransactionEvent};
pub use geohot::chimera_core::{Tensor, RiskModel, ChimeraConfig};

// PHEROMIND CORE v5.1 exports - Evolutionary Integration
pub use pheromind::{PheromindCore, PheromindConfig, PheromindIntegration};
pub use pheromind::pheromone_bus::{PheromoneBus, PheromoneSignal, Pheromone, PheromoneConfig};
pub use pheromind::genesis_analyzer::{GenesisAnalyzer, BootstrapStrategy, MarketPattern, GenesisConfig};
pub use pheromind::quantum_signer::{QuantumSafeSigner, PostQuantumSignature, QuantumConfig};

// OPERACJA "FORGE" - TensorZero Integration exports
pub use forge::{TheForge, ForgeConfig, EvolutionResult, ForgeMetrics};
pub use forge::tensorzero_gateway::{TensorZeroGateway, TensorZeroConfig, InferenceRequest, InferenceResponse};
pub use forge::dsl_generator::{StrategyDSLGenerator, StrategyDSL, StrategyType, GenerationMethod};
pub use forge::strategy_compiler::{StrategyCompiler, CompilerConfig, CompilationResult};
pub use forge::{CompiledArtifact};
pub use forge::hot_loader::{StrategyHotLoader, StrategyContainer, MarketData, HftContext, StrategyMetrics};

// DYNAMIC AGENT SYSTEM exports - FAZA 2 OPERACJI "FORGE"
pub use agents::{DynamicAgent, DynamicAgentConfig, DynamicAgentMetrics, AgentType, AgentState, AgentCommand};
pub use agents::{AgentManager, AgentManagerMetrics, AutoEvolutionConfig, RiskParameters, ExecutionParameters};
pub use agents::{RuntimeModuleLoader, CachedArtifact, LoadingMetrics, LoaderConfig, CacheStats};
