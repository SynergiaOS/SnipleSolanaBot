//! THE OVERMIND PROTOCOL v4.4 "GEOHOT CORE" - Cortex Core
//! 
//! Implementacja filozofii Hotza: "Żadnych Czarnych Skrzynek"
//! Każdy cykl zegara ma znaczenie. Minimalizm jako broń.

pub mod core;
pub mod swarm;
pub mod dispatcher;
pub mod agents;
pub mod e2e_tests;
pub mod hardware_accel;
pub mod zero_copy_v2;
pub mod atomic_sentiment;
pub mod sub_millisecond_e2e;

#[cfg(feature = "geo_hot")]
pub mod amd_kernel;

pub use core::CortexCore;
pub use swarm::SwarmTopology;
pub use dispatcher::{AiTaskType, CortexDispatcher};

/// Główne błędy systemu Cortex
#[derive(Debug)]
pub enum CortexError {
    /// Błąd komunikacji z AI Gateway
    AiGatewayError(String),
    /// Błąd swarm topology
    SwarmError(String),
    /// Błąd AMD kernel (tylko z feature geo_hot)
    #[cfg(feature = "geo_hot")]
    AmdKernelError(String),
    /// Błąd parsowania odpowiedzi
    ParseError(String),
    /// Błąd timeout
    TimeoutError,
}

impl std::fmt::Display for CortexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CortexError::AiGatewayError(msg) => write!(f, "AI Gateway Error: {}", msg),
            CortexError::SwarmError(msg) => write!(f, "Swarm Error: {}", msg),
            #[cfg(feature = "geo_hot")]
            CortexError::AmdKernelError(msg) => write!(f, "AMD Kernel Error: {}", msg),
            CortexError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CortexError::TimeoutError => write!(f, "Timeout Error"),
        }
    }
}

impl std::error::Error for CortexError {}

/// Wynik operacji Cortex
pub type CortexResult<T> = Result<T, CortexError>;

/// Metryki wydajności zgodnie z filozofią Hotza
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Najgorszy przypadek (nanosekundy)
    pub worst_case_ns: u64,
    /// Średni przypadek (nanosekundy)
    pub avg_case_ns: u64,
    /// Liczba cykli CPU
    pub cycle_count: u64,
    /// Timestamp ostatniego pomiaru
    pub last_measurement: std::time::Instant,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            worst_case_ns: 0,
            avg_case_ns: 0,
            cycle_count: 0,
            last_measurement: std::time::Instant::now(),
        }
    }

    /// Aktualizacja metryk z nowym pomiarem
    pub fn update(&mut self, duration_ns: u64) {
        self.worst_case_ns = self.worst_case_ns.max(duration_ns);
        self.avg_case_ns = (self.avg_case_ns + duration_ns) / 2;
        self.cycle_count += 1;
        self.last_measurement = std::time::Instant::now();
    }
}

/// Makro do profilowania zgodnie z filozofią Hotza
#[macro_export]
macro_rules! profile_hotz {
    ($name:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed().as_nanos() as u64;
        
        // Log tylko jeśli przekracza próg wydajności
        if duration > 1_000_000 { // 1ms
            eprintln!("HOTZ_PROFILE: {} took {}ns (CRITICAL)", $name, duration);
        }
        
        result
    }};
}

/// Inicjalizacja systemu Cortex
pub fn init_cortex() -> CortexResult<CortexCore> {
    profile_hotz!("cortex_init", {
        CortexCore::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();
        metrics.update(1000);
        metrics.update(2000);

        assert_eq!(metrics.worst_case_ns, 2000);
        // Średnia: (0 + 1000) / 2 = 500, potem (500 + 2000) / 2 = 1250
        assert_eq!(metrics.avg_case_ns, 1250);
        assert_eq!(metrics.cycle_count, 2);
    }

    #[test]
    fn test_cortex_error_display() {
        let error = CortexError::AiGatewayError("test".to_string());
        assert_eq!(format!("{}", error), "AI Gateway Error: test");
    }
}
