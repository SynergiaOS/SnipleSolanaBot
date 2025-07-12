//! THE OVERMIND PROTOCOL v5.4 "INSIGHT CORE"
//!
//! CryptoInsight AI Integration with Hyper-Data Ingestion & Cognitive Architecture
//! All-Rust implementation of autonomous AI trading system for Solana blockchain
//!
//! This library provides a comprehensive trading system with the following features:
//! - THE OVERMIND CORTEX: AI Swarm + Knowledge Graph + Evolution
//! - CryptoInsight AI: 4-layer AI (Seer, Inquisitor, Executioner, Whisper)
//! - Jito-aware streaming with <31ms latency and MEV protection
//! - Warden SPEX integration for verifiable AI predictions
//! - Hybrydowy Feature Store: Redis + ClickHouse + Arweave
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
pub mod swarmagentic;
pub mod autoschema;
pub mod security;
pub mod cryptoinsight;
pub mod memory;
pub mod neural_execution;
pub mod cargo_resolver;

// Re-export commonly used items
pub use config::Config;
pub use modules::ai_connector::AIConnector;
pub use modules::hft_engine::HftEngine;
pub use modules::micro_lightning::{MicroLightningOrchestrator, MicroLightningStrategy};

// THE OVERMIND PROTOCOL v5.4 exports
pub use overmind::OvermindProtocol;
pub use overmind::cortex::Cortex;
pub use overmind::swarm::{AgentCandidate, SwarmOrchestrator};
pub use overmind::knowledge_graph::KnowledgeGraph;
pub use overmind::optimization::DataFlywheel;
pub use overmind::evolution::EvolutionEngine;

// CryptoInsight AI v5.4 exports - Hyper-Data Ingestion & Cognitive Architecture
pub use cryptoinsight::{
    CryptoInsightCore, CryptoInsightConfig, CryptoInsightHealth,
    JitoAwareStreamer, GeyserService, QuicClient,
    CognitiveCortex, SeerLSTM, InquisitorGAN, ExecutionerRL, WhisperNLP,
    SharedKnowledgeBase, AIBattalion, AIPrediction,
    WardenSPEX, VerifiableAI, SPEXProof, ModelMetadata,
    HybridFeatureStore, FeatureSet, RedisVectorDB,
    MemecoinShield, DecoyFactory, JitoBundle,
    PumpFunMonitor, WashTradingDetector, PatternAnalyzer, PumpFunPattern
};

// Ultra-Low Latency Memory System exports - SolanaNoa TradeMaster
pub use memory::{
    UltraMemorySystem, MemoryConfig, MemoryMetrics, MemoryHealthStatus,
    WorkingMemory, MemoryBatch, TransactionContext, TransactionType,
    EpisodicStorage, SemanticIndex, MemorySnapshot,
    JitoMemoryWriter, BundleProcessor, MEVTagger,
    SecurityVault, PolicyEngine, MemoryAccess,
    GPUEmbeddings, CUDAProcessor, VectorBatch,
    BackupManager, RestoreProtocol, IntegrityVerifier
};

// Neural Execution Engine exports - Ultra-Low Latency Trading System
pub use neural_execution::{
    NeuralExecutionEngine, NeuralExecutionConfig, NeuralExecutionMetrics, NeuralExecutionHealth,
    ExecutionRequest, ExecutionResult, ExecutionStatus, ExecutionPriority,
    NeuralRouter, HardwareTopology, RoutingDecision, PathOptimizer,
    AtomicExecutor, ZeroCopyDispatcher, ExecutionPipeline, SIMDProcessor,
    NeuralPredictor, MLExecutionModel, TimingPredictor, ReinforcementLearner,
    HardwareAccelerator, FPGAInterface, ASICController, CustomSilicon,
    ExecutionMonitor, PerformanceProfiler, BottleneckDetector, MicrosecondAnalytics
};

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
