// THE OVERMIND PROTOCOL v5.4 'INSIGHT CORE'
// CryptoInsight AI Integration - Hyper-Data Ingestion & Cognitive Architecture
// 
// Architektura:
// 1. Warstwa Sensoryczna: JitoAwareStreamer + PumpFunMonitor
// 2. Cognitive Cortex: 4-layer AI (Seer, Inquisitor, Executioner, Whisper)
// 3. Warden SPEX Integration: Verifiable AI predictions
// 4. Hybrydowy Feature Store: Redis + ClickHouse + Arweave
// 5. Anti-MEV Shield: Decoy transactions + Jito bundles

pub mod jito_streamer;
pub mod cognitive_cortex;
pub mod warden_integration;
pub mod feature_store;
pub mod anti_mev_shield;
pub mod pump_fun_monitor;

// Re-exports for convenience
pub use jito_streamer::{JitoAwareStreamer, GeyserService, QuicClient, SolanaTx};
pub use cognitive_cortex::{
    CognitiveCortex, SeerLSTM, InquisitorGAN, ExecutionerRL, WhisperNLP,
    SharedKnowledgeBase, AIBattalion, AIPrediction
};
pub use warden_integration::{WardenSPEX, VerifiableAI, SPEXProof, ModelMetadata};
pub use feature_store::{HybridFeatureStore, FeatureSet, RedisVectorDB};
pub use anti_mev_shield::{MemecoinShield, DecoyFactory, JitoBundle};
pub use pump_fun_monitor::{PumpFunMonitor, WashTradingDetector, PatternAnalyzer, PumpFunPattern};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// CryptoInsight AI Core Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoInsightConfig {
    /// Jito-aware streaming configuration
    pub jito_config: JitoStreamingConfig,
    
    /// AI model configurations
    pub ai_config: CognitiveCortexConfig,
    
    /// Warden SPEX integration settings
    pub warden_config: WardenConfig,
    
    /// Feature store configuration
    pub feature_store_config: FeatureStoreConfig,
    
    /// Anti-MEV shield settings
    pub anti_mev_config: AntiMEVConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitoStreamingConfig {
    /// Geyser plugin endpoint
    pub geyser_endpoint: String,
    
    /// QUIC client configuration
    pub quic_config: QuicConfig,
    
    /// Jito bundle endpoint
    pub jito_endpoint: String,
    
    /// Maximum latency target (ms)
    pub max_latency_ms: u64,
    
    /// Bundle size for optimal throughput
    pub optimal_bundle_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    /// QUIC endpoint
    pub endpoint: String,
    
    /// Connection timeout
    pub timeout_ms: u64,
    
    /// Keep-alive interval
    pub keep_alive_ms: u64,
    
    /// Max concurrent streams
    pub max_streams: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveCortexConfig {
    /// Seer LSTM configuration
    pub seer_config: SeerConfig,
    
    /// Inquisitor GAN configuration
    pub inquisitor_config: InquisitorConfig,
    
    /// Executioner RL configuration
    pub executioner_config: ExecutionerConfig,
    
    /// Whisper NLP configuration
    pub whisper_config: WhisperConfig,
    
    /// Shared knowledge base settings
    pub knowledge_base_config: KnowledgeBaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeerConfig {
    /// LSTM model path
    pub model_path: String,
    
    /// Sequence length for time series
    pub sequence_length: usize,
    
    /// Hidden layer size
    pub hidden_size: usize,
    
    /// Number of LSTM layers
    pub num_layers: usize,
    
    /// Prediction horizon (minutes)
    pub prediction_horizon: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InquisitorConfig {
    /// GAN model path
    pub model_path: String,
    
    /// Anomaly detection threshold
    pub anomaly_threshold: f64,
    
    /// Generator latent dimension
    pub latent_dim: usize,
    
    /// Discriminator learning rate
    pub discriminator_lr: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionerConfig {
    /// RL model path
    pub model_path: String,
    
    /// Action space size
    pub action_space: usize,
    
    /// Reward discount factor
    pub gamma: f64,
    
    /// Exploration rate
    pub epsilon: f64,
    
    /// Learning rate
    pub learning_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperConfig {
    /// NLP model path
    pub model_path: String,
    
    /// Sentiment analysis threshold
    pub sentiment_threshold: f64,
    
    /// Maximum text length
    pub max_text_length: usize,
    
    /// Language models to use
    pub language_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    /// Vector database endpoint
    pub vector_db_endpoint: String,
    
    /// Embedding dimension
    pub embedding_dim: usize,
    
    /// Knowledge retention period (hours)
    pub retention_hours: u64,
    
    /// Similarity threshold for retrieval
    pub similarity_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardenConfig {
    /// Warden SPEX endpoint
    pub spex_endpoint: String,
    
    /// Verification timeout (seconds)
    pub verification_timeout: u64,
    
    /// ZK proof generation settings
    pub zk_config: ZKConfig,
    
    /// Model verification requirements
    pub verification_requirements: VerificationRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKConfig {
    /// Proof system to use
    pub proof_system: String,
    
    /// Circuit complexity limit
    pub max_circuit_size: usize,
    
    /// Proof generation timeout
    pub proof_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequirements {
    /// Minimum accuracy required
    pub min_accuracy: f64,
    
    /// Maximum inference time (ms)
    pub max_inference_time: u64,
    
    /// Required compliance standards
    pub compliance_standards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStoreConfig {
    /// Redis vector DB configuration
    pub redis_config: RedisVectorConfig,
    
    /// ClickHouse cluster configuration
    pub clickhouse_config: ClickHouseConfig,
    
    /// Arweave backend configuration
    pub arweave_config: ArweaveConfig,
    
    /// Feature TTL settings
    pub ttl_config: TTLConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisVectorConfig {
    /// Redis cluster endpoints
    pub endpoints: Vec<String>,
    
    /// Vector dimension
    pub vector_dim: usize,
    
    /// Index type (FLAT, HNSW, etc.)
    pub index_type: String,
    
    /// Distance metric
    pub distance_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickHouseConfig {
    /// ClickHouse cluster endpoints
    pub endpoints: Vec<String>,
    
    /// Database name
    pub database: String,
    
    /// Connection pool size
    pub pool_size: usize,
    
    /// Query timeout (seconds)
    pub query_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArweaveConfig {
    /// Arweave gateway URL
    pub gateway_url: String,
    
    /// Wallet key path
    pub wallet_path: String,
    
    /// Upload timeout (seconds)
    pub upload_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TTLConfig {
    /// Onchain data TTL (seconds)
    pub onchain_ttl: u64,
    
    /// Offchain data TTL (seconds)
    pub offchain_ttl: u64,
    
    /// ZK proof TTL (seconds)
    pub proof_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiMEVConfig {
    /// Number of decoy transactions
    pub decoy_count: usize,
    
    /// Jito bundle expiry slots
    pub bundle_expiry_slots: u64,
    
    /// Camouflage strategy type
    pub camouflage_strategy: String,
    
    /// MEV protection level (1-10)
    pub protection_level: u8,
}

impl Default for CryptoInsightConfig {
    fn default() -> Self {
        Self {
            jito_config: JitoStreamingConfig {
                geyser_endpoint: "ws://localhost:8900".to_string(),
                quic_config: QuicConfig {
                    endpoint: "quic://localhost:8901".to_string(),
                    timeout_ms: 5000,
                    keep_alive_ms: 30000,
                    max_streams: 1000,
                },
                jito_endpoint: "https://mainnet.block-engine.jito.wtf".to_string(),
                max_latency_ms: 31, // Target <31ms latency
                optimal_bundle_size: 5,
            },
            ai_config: CognitiveCortexConfig {
                seer_config: SeerConfig {
                    model_path: "./models/seer_lstm_v6.onnx".to_string(),
                    sequence_length: 100,
                    hidden_size: 256,
                    num_layers: 3,
                    prediction_horizon: 15, // 15 minutes
                },
                inquisitor_config: InquisitorConfig {
                    model_path: "./models/inquisitor_gan_v4.onnx".to_string(),
                    anomaly_threshold: 0.85,
                    latent_dim: 128,
                    discriminator_lr: 0.0002,
                },
                executioner_config: ExecutionerConfig {
                    model_path: "./models/executioner_rl_v3.onnx".to_string(),
                    action_space: 10,
                    gamma: 0.99,
                    epsilon: 0.1,
                    learning_rate: 0.001,
                },
                whisper_config: WhisperConfig {
                    model_path: "./models/whisper_nlp_v5.onnx".to_string(),
                    sentiment_threshold: 0.7,
                    max_text_length: 512,
                    language_models: vec!["bert-base".to_string(), "roberta-large".to_string()],
                },
                knowledge_base_config: KnowledgeBaseConfig {
                    vector_db_endpoint: "redis://localhost:6379".to_string(),
                    embedding_dim: 768,
                    retention_hours: 168, // 1 week
                    similarity_threshold: 0.8,
                },
            },
            warden_config: WardenConfig {
                spex_endpoint: "https://api.warden.network/spex/v1".to_string(),
                verification_timeout: 30,
                zk_config: ZKConfig {
                    proof_system: "groth16".to_string(),
                    max_circuit_size: 1000000,
                    proof_timeout_ms: 10000,
                },
                verification_requirements: VerificationRequirements {
                    min_accuracy: 0.94,
                    max_inference_time: 10, // <10ms inference
                    compliance_standards: vec!["MiCA".to_string(), "GDPR".to_string()],
                },
            },
            feature_store_config: FeatureStoreConfig {
                redis_config: RedisVectorConfig {
                    endpoints: vec!["redis://localhost:6379".to_string()],
                    vector_dim: 768,
                    index_type: "HNSW".to_string(),
                    distance_metric: "COSINE".to_string(),
                },
                clickhouse_config: ClickHouseConfig {
                    endpoints: vec!["http://localhost:8123".to_string()],
                    database: "cryptoinsight".to_string(),
                    pool_size: 10,
                    query_timeout: 30,
                },
                arweave_config: ArweaveConfig {
                    gateway_url: "https://arweave.net".to_string(),
                    wallet_path: "./keys/arweave-wallet.json".to_string(),
                    upload_timeout: 60,
                },
                ttl_config: TTLConfig {
                    onchain_ttl: 15, // 15 seconds for ultra-fast access
                    offchain_ttl: 300, // 5 minutes
                    proof_ttl: 3600, // 1 hour
                },
            },
            anti_mev_config: AntiMEVConfig {
                decoy_count: 10,
                bundle_expiry_slots: 2,
                camouflage_strategy: "adaptive".to_string(),
                protection_level: 9, // Maximum protection
            },
        }
    }
}

/// Main CryptoInsight AI Core
pub struct CryptoInsightCore {
    /// Configuration
    config: CryptoInsightConfig,
    
    /// Jito-aware data streamer
    jito_streamer: Arc<RwLock<JitoAwareStreamer>>,
    
    /// Cognitive cortex (AI brain)
    cognitive_cortex: Arc<RwLock<CognitiveCortex>>,
    
    /// Warden SPEX integration
    warden_spex: Arc<RwLock<WardenSPEX>>,
    
    /// Hybrid feature store
    feature_store: Arc<RwLock<HybridFeatureStore>>,
    
    /// Anti-MEV shield
    anti_mev_shield: Arc<RwLock<MemecoinShield>>,
    
    /// PumpFun monitor
    pump_fun_monitor: Arc<RwLock<PumpFunMonitor>>,
}

impl CryptoInsightCore {
    /// Create new CryptoInsight AI Core
    pub async fn new(config: CryptoInsightConfig) -> Result<Self> {
        info!("🧠 Initializing CryptoInsight AI Core v5.4 'INSIGHT CORE'");
        
        // Initialize components
        let jito_streamer = Arc::new(RwLock::new(
            JitoAwareStreamer::new(config.jito_config.clone()).await?
        ));
        
        let cognitive_cortex = Arc::new(RwLock::new(
            CognitiveCortex::new(config.ai_config.clone()).await?
        ));
        
        let warden_spex = Arc::new(RwLock::new(
            WardenSPEX::new(config.warden_config.clone()).await?
        ));
        
        let feature_store = Arc::new(RwLock::new(
            HybridFeatureStore::new(config.feature_store_config.clone()).await?
        ));
        
        let anti_mev_shield = Arc::new(RwLock::new(
            MemecoinShield::new(config.anti_mev_config.clone()).await?
        ));
        
        let pump_fun_monitor = Arc::new(RwLock::new(
            PumpFunMonitor::new().await?
        ));
        
        info!("✅ CryptoInsight AI Core initialized successfully");
        
        Ok(Self {
            config,
            jito_streamer,
            cognitive_cortex,
            warden_spex,
            feature_store,
            anti_mev_shield,
            pump_fun_monitor,
        })
    }
    
    /// Start the CryptoInsight AI pipeline
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting CryptoInsight AI pipeline...");
        
        // Start all components
        self.jito_streamer.read().await.start().await?;
        self.cognitive_cortex.read().await.start().await?;
        self.warden_spex.read().await.start().await?;
        self.feature_store.read().await.start().await?;
        self.anti_mev_shield.read().await.start().await?;
        self.pump_fun_monitor.read().await.start().await?;
        
        info!("✅ CryptoInsight AI pipeline started successfully");
        Ok(())
    }
    
    /// Stop the CryptoInsight AI pipeline
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping CryptoInsight AI pipeline...");
        
        // Stop all components
        self.pump_fun_monitor.read().await.stop().await?;
        self.anti_mev_shield.read().await.stop().await?;
        self.feature_store.read().await.stop().await?;
        self.warden_spex.read().await.stop().await?;
        self.cognitive_cortex.read().await.stop().await?;
        self.jito_streamer.read().await.stop().await?;
        
        info!("✅ CryptoInsight AI pipeline stopped successfully");
        Ok(())
    }
    
    /// Get system health status
    pub async fn health_check(&self) -> Result<CryptoInsightHealth> {
        let jito_health = self.jito_streamer.read().await.health_check().await?;
        let ai_health = self.cognitive_cortex.read().await.health_check().await?;
        let warden_health = self.warden_spex.read().await.health_check().await?;
        let feature_health = self.feature_store.read().await.health_check().await?;
        let mev_health = self.anti_mev_shield.read().await.health_check().await?;
        let pump_health = self.pump_fun_monitor.read().await.health_check().await?;
        
        Ok(CryptoInsightHealth {
            jito_streamer: jito_health,
            cognitive_cortex: ai_health,
            warden_spex: warden_health,
            feature_store: feature_health,
            anti_mev_shield: mev_health,
            pump_fun_monitor: pump_health,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoInsightHealth {
    pub jito_streamer: ComponentHealth,
    pub cognitive_cortex: ComponentHealth,
    pub warden_spex: ComponentHealth,
    pub feature_store: ComponentHealth,
    pub anti_mev_shield: ComponentHealth,
    pub pump_fun_monitor: ComponentHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub error_rate: f64,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
