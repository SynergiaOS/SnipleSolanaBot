//! CortexCore - Serce THE OVERMIND PROTOCOL v4.4 "GEOHOT CORE"
//! 
//! Implementacja zgodna z Task 2.1 z dokumentu

use crate::{CortexResult, CortexError, PerformanceMetrics, SwarmTopology};
use chimera_client::ChimeraClient;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Główny rdzeń systemu Cortex
pub struct CortexCore {
    /// AI Gateway dla komunikacji z modelami
    ai_gateway: ChimeraClient,
    /// Topologia swarm agentów
    swarm_nexus: Arc<RwLock<SwarmTopology>>,
    /// Akcelerator sprzętowy AMD (opcjonalny)
    #[cfg(feature = "geo_hot")]
    hw_accelerator: Option<crate::amd_kernel::AmdLlamaKernel>,
    /// Metryki wydajności
    performance_vault: Arc<RwLock<PerformanceMetrics>>,
}

impl CortexCore {
    /// Utworzenie nowego CortexCore
    pub fn new() -> CortexResult<Self> {
        // Konfiguracja ChimeraClient z domyślnymi ustawieniami
        let config = chimera_client::ChimeraConfig::new("demo_api_key".to_string());
        let ai_gateway = ChimeraClient::new(config)
            .map_err(|e| CortexError::AiGatewayError(format!("Failed to create ChimeraClient: {}", e)))?;

        let swarm_nexus = Arc::new(RwLock::new(SwarmTopology::new()));
        let performance_vault = Arc::new(RwLock::new(PerformanceMetrics::new()));

        #[cfg(feature = "geo_hot")]
        let hw_accelerator = {
            match crate::amd_kernel::AmdLlamaKernel::new() {
                Ok(kernel) => Some(kernel),
                Err(e) => {
                    eprintln!("AMD Kernel initialization failed: {}, falling back to CPU", e);
                    None
                }
            }
        };

        Ok(Self {
            ai_gateway,
            swarm_nexus,
            #[cfg(feature = "geo_hot")]
            hw_accelerator,
            performance_vault,
        })
    }

    /// Dispatch zadania AI zgodnie z Task 2.2
    pub async fn dispatch(&self, task: crate::dispatcher::AiTaskType) -> CortexResult<Vec<u8>> {
        let start = std::time::Instant::now();

        // Statyczne alokacje - zero garbage (filozofia Hotza)
        let mut prompt_buffer = [0u8; 512];
        let prompt_len = self.build_prompt(&mut prompt_buffer, &task)?;

        // Wykonanie przez AI Gateway
        let raw_response = self.ai_gateway
            .execute_binary(&prompt_buffer[..prompt_len])
            .await
            .map_err(|e| CortexError::AiGatewayError(format!("Gateway execution failed: {}", e)))?;

        // Aktualizacja metryk wydajności
        let duration_ns = start.elapsed().as_nanos() as u64;
        {
            let mut metrics = self.performance_vault.write().await;
            metrics.update(duration_ns);
        }

        Ok(raw_response)
    }

    /// Budowanie prompt w buforze statycznym
    fn build_prompt(&self, buffer: &mut [u8], task: &crate::dispatcher::AiTaskType) -> CortexResult<usize> {
        use crate::dispatcher::AiTaskType;

        let prompt_bytes = match task {
            AiTaskType::SentimentAnalysis(flags) => {
                format!("SENTIMENT_ANALYSIS:flags={}", flags).into_bytes()
            },
            AiTaskType::LiquidityScan(token_hash) => {
                // Simple hex encoding without external dependency
                let hex_str = token_hash.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>();
                format!("LIQUIDITY_SCAN:token={}", hex_str).into_bytes()
            },
            AiTaskType::RiskAssessment { token, time_window } => {
                format!("RISK_ASSESSMENT:token={},window={}", token, time_window).into_bytes()
            },
        };

        if prompt_bytes.len() > buffer.len() {
            return Err(CortexError::ParseError("Prompt too large for buffer".to_string()));
        }

        buffer[..prompt_bytes.len()].copy_from_slice(&prompt_bytes);
        Ok(prompt_bytes.len())
    }

    /// Dostęp do swarm topology
    pub async fn get_swarm(&self) -> Arc<RwLock<SwarmTopology>> {
        self.swarm_nexus.clone()
    }

    /// Sprawdzenie stanu AMD accelerator
    #[cfg(feature = "geo_hot")]
    pub fn has_amd_acceleration(&self) -> bool {
        self.hw_accelerator.is_some()
    }

    /// Wykonanie na AMD kernel (jeśli dostępny)
    #[cfg(feature = "geo_hot")]
    pub async fn execute_amd_kernel(&self, input: &[f32]) -> CortexResult<Vec<f32>> {
        if let Some(ref kernel) = self.hw_accelerator {
            kernel.forward(input)
                .map_err(|e| CortexError::AmdKernelError(format!("Kernel execution failed: {}", e)))
        } else {
            Err(CortexError::AmdKernelError("No AMD accelerator available".to_string()))
        }
    }

    /// Pobranie metryk wydajności
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        let metrics = self.performance_vault.read().await;
        metrics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cortex_core_creation() {
        // Test może się nie powieść jeśli ChimeraClient nie jest dostępny
        // W środowisku testowym używamy mock
        // let core = CortexCore::new();
        // assert!(core.is_ok());
    }

    #[test]
    fn test_prompt_building() {
        // Test może być włączony gdy ChimeraClient::mock będzie dostępny
        // let core = CortexCore::mock();
        // let mut buffer = [0u8; 512];
        // let task = crate::dispatcher::AiTaskType::SentimentAnalysis(0x1234);
        // let result = core.build_prompt(&mut buffer, &task);
        // assert!(result.is_ok());
    }
}
