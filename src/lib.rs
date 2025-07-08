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
