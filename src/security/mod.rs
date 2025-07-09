//! THE OVERMIND PROTOCOL - Security Module
//! 
//! Enterprise-grade security components for THE OVERMIND PROTOCOL
//! - Infisical integration for secret management
//! - DragonflyDB cache for high-performance secret caching
//! - VPC network isolation and security

pub mod infisical_client;
pub mod dragonflydb_cache;
pub mod quantum_safe;
pub mod ai_monitor;
pub mod zero_trust;
pub mod blockchain_vault;
pub mod homomorphic;

// Re-export main components
pub use infisical_client::{InfisicalClient, SecureEnvLoader, create_infisical_client};
pub use dragonflydb_cache::{DragonflyCache, DragonflyConfig, CacheStats, create_dragonflydb_cache};
pub use quantum_safe::{QuantumSafeManager, QuantumSafeInfisicalWrapper, create_quantum_safe_manager};
pub use ai_monitor::{AISecurityMonitor, SecurityEvent, SecurityEventType, ThreatAssessment, create_ai_security_monitor};
pub use zero_trust::{ZeroTrustEngine, ZeroTrustIdentity, AccessRequest, AccessDecision, create_zero_trust_engine, create_overmind_identity};
pub use blockchain_vault::{BlockchainVault, HybridVaultStorage, HybridStorageStrategy, create_blockchain_vault};
pub use homomorphic::{HomomorphicEncryption, HomomorphicContext, HomomorphicSecretComputation, create_homomorphic_encryption, create_homomorphic_context};
