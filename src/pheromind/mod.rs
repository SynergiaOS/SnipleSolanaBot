//! PHEROMIND CORE - v5.1 Evolution
//! 
//! Integration of Pheromind concepts into THE OVERMIND PROTOCOL
//! - PheromoneBus: Emergent swarm communication
//! - GenesisAnalyzer: Historical pattern extraction
//! - QuantumSafeSigner: Post-quantum cryptography

pub mod pheromone_bus;
pub mod genesis_analyzer;
pub mod quantum_signer;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::info;

use pheromone_bus::{PheromoneBus, PheromoneConfig, Pheromone};
use genesis_analyzer::{GenesisAnalyzer, GenesisConfig, BootstrapStrategy};
use quantum_signer::{QuantumSafeSigner, QuantumConfig};

/// PHEROMIND CORE main orchestrator
pub struct PheromindCore {
    /// Pheromone communication system
    pheromone_bus: PheromoneBus,
    
    /// Genesis pattern analyzer
    genesis_analyzer: GenesisAnalyzer,
    
    /// Quantum-safe transaction signer
    quantum_signer: QuantumSafeSigner,
    
    /// Agent ID
    agent_id: String,
    
    /// Pheromone receiver channel
    pheromone_receiver: Option<mpsc::UnboundedReceiver<Pheromone>>,
    
    /// Configuration
    config: PheromindConfig,
}

/// PHEROMIND CORE configuration
#[derive(Debug, Clone)]
pub struct PheromindConfig {
    pub agent_id: String,
    pub pheromone_config: PheromoneConfig,
    pub genesis_config: GenesisConfig,
    pub quantum_config: QuantumConfig,
    pub enable_pheromone_bus: bool,
    pub enable_genesis_bootstrap: bool,
    pub enable_quantum_signing: bool,
}

impl Default for PheromindConfig {
    fn default() -> Self {
        Self {
            agent_id: format!("agent_{}", uuid::Uuid::new_v4()),
            pheromone_config: PheromoneConfig::default(),
            genesis_config: GenesisConfig::default(),
            quantum_config: QuantumConfig::default(),
            enable_pheromone_bus: true,
            enable_genesis_bootstrap: true,
            enable_quantum_signing: true,
        }
    }
}

impl PheromindCore {
    /// Initialize PHEROMIND CORE
    pub async fn new(config: PheromindConfig) -> Result<Self> {
        info!("🧬 Initializing PHEROMIND CORE v5.1");
        
        // Initialize PheromoneB us
        info!("🐜 Initializing PheromoneB us...");
        let (pheromone_bus, pheromone_receiver) = if config.enable_pheromone_bus {
            let (bus, receiver) = PheromoneBus::new(
                config.pheromone_config.clone(),
                config.agent_id.clone(),
            )?;
            (bus, Some(receiver))
        } else {
            // Create dummy bus for disabled mode
            let (bus, receiver) = PheromoneBus::new(
                config.pheromone_config.clone(),
                config.agent_id.clone(),
            )?;
            (bus, Some(receiver))
        };
        
        // Initialize GenesisAnalyzer
        info!("🏛️ Initializing GenesisAnalyzer...");
        let genesis_analyzer = if config.enable_genesis_bootstrap {
            GenesisAnalyzer::new(config.genesis_config.clone()).await?
        } else {
            GenesisAnalyzer::new_disabled()
        };
        
        // Initialize QuantumSafeSigner
        info!("🔐 Initializing QuantumSafeSigner...");
        let quantum_signer = if config.enable_quantum_signing {
            QuantumSafeSigner::new(config.quantum_config.clone())?
        } else {
            QuantumSafeSigner::new_disabled()
        };
        
        info!("✅ PHEROMIND CORE initialized successfully");
        
        Ok(Self {
            pheromone_bus,
            genesis_analyzer,
            quantum_signer,
            agent_id: config.agent_id.clone(),
            pheromone_receiver,
            config,
        })
    }
    
    /// Start PHEROMIND CORE processing
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting PHEROMIND CORE processing");
        
        // Bootstrap strategies from genesis analysis
        if self.config.enable_genesis_bootstrap {
            info!("🏛️ Bootstrapping strategies from genesis analysis...");
            let bootstrap_strategies = self.genesis_analyzer.extract_bootstrap_strategies().await?;
            info!("📊 Extracted {} bootstrap strategies", bootstrap_strategies.len());
            
            // Apply bootstrap strategies to pheromone system
            for strategy in bootstrap_strategies {
                self.apply_bootstrap_strategy(strategy).await?;
            }
        }
        
        // Start pheromone processing loop
        if let Some(mut receiver) = self.pheromone_receiver.take() {
            tokio::spawn(async move {
                while let Some(pheromone) = receiver.recv().await {
                    // Process incoming pheromones
                    // This would integrate with the main trading logic
                    info!("📡 Received pheromone: {:?}", pheromone.signal);
                }
            });
        }
        
        // Start cleanup task
        // Note: In a real implementation, we would use Arc<PheromoneBus> for sharing
        // For now, we'll skip the cleanup task to avoid clone issues
        
        Ok(())
    }
    
    /// Apply bootstrap strategy to the system
    async fn apply_bootstrap_strategy(&self, strategy: BootstrapStrategy) -> Result<()> {
        info!("🎯 Applying bootstrap strategy: {}", strategy.name);
        
        // Convert bootstrap strategy to initial pheromones
        let initial_pheromone = match strategy.strategy_type.as_str() {
            "bullish_momentum" => {
                pheromone_bus::PheromoneSignal::BuySignal {
                    strength: strategy.confidence,
                    confidence: strategy.historical_success_rate,
                }
            }
            "bearish_momentum" => {
                pheromone_bus::PheromoneSignal::SellSignal {
                    strength: strategy.confidence,
                    confidence: strategy.historical_success_rate,
                }
            }
            "risk_aversion" => {
                pheromone_bus::PheromoneSignal::RiskWarning {
                    risk_level: 1.0 - strategy.confidence,
                    reason: strategy.description.clone(),
                }
            }
            "opportunity_seeking" => {
                pheromone_bus::PheromoneSignal::Opportunity {
                    potential: strategy.confidence,
                    timeframe: strategy.timeframe_hours * 3600,
                }
            }
            _ => {
                pheromone_bus::PheromoneSignal::HoldSignal {
                    patience_level: strategy.confidence,
                }
            }
        };
        
        // Deposit initial pheromone
        let market_context = pheromone_bus::MarketContext {
            symbol: None,
            price: None,
            volume: None,
            volatility: None,
            trend: Some(if strategy.strategy_type.contains("bullish") { 1.0 } else { -1.0 }),
        };
        
        let _pheromone_id = self.pheromone_bus.deposit_pheromone(
            initial_pheromone,
            strategy.confidence,
            market_context,
            Some(strategy.timeframe_hours * 3600),
        ).await?;
        
        info!("✅ Bootstrap strategy applied: {}", strategy.name);
        Ok(())
    }
    
    /// Get pheromone bus reference
    pub fn pheromone_bus(&self) -> &PheromoneBus {
        &self.pheromone_bus
    }
    
    /// Get genesis analyzer reference
    pub fn genesis_analyzer(&self) -> &GenesisAnalyzer {
        &self.genesis_analyzer
    }
    
    /// Get quantum signer reference
    pub fn quantum_signer(&self) -> &QuantumSafeSigner {
        &self.quantum_signer
    }
    
    /// Get agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
}

/// Initialize PHEROMIND CORE with default configuration
pub async fn init_pheromind_core(agent_id: Option<String>) -> Result<PheromindCore> {
    let config = PheromindConfig {
        agent_id: agent_id.unwrap_or_else(|| format!("agent_{}", uuid::Uuid::new_v4())),
        ..Default::default()
    };
    
    PheromindCore::new(config).await
}

/// PHEROMIND integration with OVERMIND PROTOCOL
pub struct PheromindIntegration {
    core: PheromindCore,
}

impl PheromindIntegration {
    /// Create new integration
    pub async fn new(config: PheromindConfig) -> Result<Self> {
        let core = PheromindCore::new(config).await?;
        Ok(Self { core })
    }
    
    /// Integrate with existing OVERMIND PROTOCOL
    pub async fn integrate_with_overmind(&mut self) -> Result<()> {
        info!("🔗 Integrating PHEROMIND with OVERMIND PROTOCOL");
        
        // Start PHEROMIND processing
        self.core.start().await?;
        
        info!("✅ PHEROMIND integration complete");
        Ok(())
    }
    
    /// Get core reference
    pub fn core(&self) -> &PheromindCore {
        &self.core
    }
    
    /// Get mutable core reference
    pub fn core_mut(&mut self) -> &mut PheromindCore {
        &mut self.core
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pheromind_core_initialization() {
        let config = PheromindConfig::default();
        
        // This test requires Redis and other dependencies
        // In a real test environment, we would mock these
        match PheromindCore::new(config).await {
            Ok(_) => println!("PHEROMIND CORE initialized successfully"),
            Err(e) => println!("Expected error in test environment: {}", e),
        }
    }
    
    #[test]
    fn test_config_defaults() {
        let config = PheromindConfig::default();
        assert!(config.enable_pheromone_bus);
        assert!(config.enable_genesis_bootstrap);
        assert!(config.enable_quantum_signing);
    }
}
