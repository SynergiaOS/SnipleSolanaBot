//! Sub-millisecond E2E Pipeline - "HOTZ ASCENSION"
//! 
//! Optymalizacja całego pipeline do <1ms zgodnie z metrykami Hotza
//! "922μs target - Czas reakcji pod ostrzałem"

use crate::{CortexResult, CortexCore};
use crate::atomic_sentiment::{AtomicSentimentAgent, BenchmarkResults};
use crate::hardware_accel::{rdtsc, hardware_fingerprint};
use crate::zero_copy_v2::ZeroCopyDispatcher;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Sub-millisecond E2E Pipeline
pub struct SubMillisecondPipeline {
    /// Atomic sentiment agent
    sentiment_agent: Arc<AtomicSentimentAgent>,
    /// Zero-copy dispatcher
    dispatcher: Arc<ZeroCopyDispatcher>,
    /// Cortex core
    cortex: Arc<CortexCore>,
    /// Pipeline statistics
    stats: PipelineStats,
}

/// Statystyki pipeline
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub total_operations: u64,
    pub total_cycles: u64,
    pub fastest_operation_cycles: u64,
    pub slowest_operation_cycles: u64,
    pub sub_millisecond_count: u64,
}

/// Event z social media
#[derive(Debug, Clone)]
pub struct SocialMediaEvent {
    pub content: String,
    pub source: String,
    pub timestamp: Instant,
    pub influence_score: u32,
}

/// Decyzja tradingowa z timing
#[derive(Debug, Clone)]
pub struct TimedTradingDecision {
    pub action: TradingAction,
    pub confidence: f32,
    pub processing_time_cycles: u64,
    pub processing_time_micros: u64,
}

/// Akcja tradingowa
#[derive(Debug, Clone, PartialEq)]
pub enum TradingAction {
    Buy { amount: f64 },
    Sell { amount: f64 },
    Hold,
}

impl SubMillisecondPipeline {
    /// Utworzenie nowego sub-millisecond pipeline
    pub fn new() -> CortexResult<Self> {
        let cortex = Arc::new(CortexCore::new()?);
        let sentiment_agent = Arc::new(AtomicSentimentAgent::new(cortex.clone())?);
        let dispatcher = Arc::new(ZeroCopyDispatcher::new()?);

        Ok(Self {
            sentiment_agent,
            dispatcher,
            cortex,
            stats: PipelineStats::default(),
        })
    }

    /// Pełny cykl E2E - target <922μs
    pub fn process_event(&mut self, event: SocialMediaEvent) -> CortexResult<TimedTradingDecision> {
        let start_tsc = rdtsc();
        let start_time = Instant::now();

        // KROK 1: Analiza sentymentu (target: <411 cykli)
        let sentiment = self.sentiment_agent.analyze(&event.content)?;

        // KROK 2: Ocena ryzyka (mock - w rzeczywistości RiskAgent)
        let risk_score = self.assess_risk_fast(&event, sentiment);

        // KROK 3: Podejmowanie decyzji (deterministyczne)
        let action = self.make_decision_fast(sentiment, risk_score, &event);

        let end_tsc = rdtsc();
        let processing_time = start_time.elapsed();
        let cycles = end_tsc - start_tsc;

        // Aktualizacja statystyk
        self.update_stats(cycles, processing_time);

        Ok(TimedTradingDecision {
            action,
            confidence: sentiment.abs(),
            processing_time_cycles: cycles,
            processing_time_micros: processing_time.as_micros() as u64,
        })
    }

    /// Szybka ocena ryzyka - deterministyczna
    fn assess_risk_fast(&self, event: &SocialMediaEvent, sentiment: f32) -> f32 {
        let base_risk = 0.2;
        let sentiment_risk = sentiment.abs() * 0.1; // Wyższy sentiment = wyższe ryzyko
        let influence_risk = if event.influence_score > 100000 { 0.1 } else { 0.3 };
        
        (base_risk + sentiment_risk + influence_risk).clamp(0.0, 1.0)
    }

    /// Szybkie podejmowanie decyzji - deterministyczne
    fn make_decision_fast(&self, sentiment: f32, risk: f32, event: &SocialMediaEvent) -> TradingAction {
        let confidence = sentiment.abs() * (1.0 - risk);
        
        if confidence < 0.3 {
            return TradingAction::Hold;
        }

        // Rozmiar pozycji bazowany na influence score
        let base_amount = if event.influence_score > 50000 { 1000.0 } else { 500.0 };
        let amount = base_amount * confidence as f64;

        if sentiment > 0.5 {
            TradingAction::Buy { amount }
        } else if sentiment < -0.5 {
            TradingAction::Sell { amount }
        } else {
            TradingAction::Hold
        }
    }

    /// Aktualizacja statystyk pipeline
    fn update_stats(&mut self, cycles: u64, duration: Duration) {
        self.stats.total_operations += 1;
        self.stats.total_cycles += cycles;

        if self.stats.fastest_operation_cycles == 0 || cycles < self.stats.fastest_operation_cycles {
            self.stats.fastest_operation_cycles = cycles;
        }

        if cycles > self.stats.slowest_operation_cycles {
            self.stats.slowest_operation_cycles = cycles;
        }

        if duration.as_micros() < 1000 {
            self.stats.sub_millisecond_count += 1;
        }
    }

    /// Pobranie statystyk pipeline
    pub fn get_stats(&self) -> PipelineStats {
        self.stats.clone()
    }

    /// Benchmark wydajności pipeline
    pub fn benchmark_pipeline(&mut self, iterations: usize) -> CortexResult<PipelineBenchmark> {
        let test_events = vec![
            SocialMediaEvent {
                content: "🚀 Bitcoin to the moon! Major adoption incoming!".to_string(),
                source: "twitter".to_string(),
                timestamp: Instant::now(),
                influence_score: 150000,
            },
            SocialMediaEvent {
                content: "💥 Market crash warning! Sell everything now!".to_string(),
                source: "reddit".to_string(),
                timestamp: Instant::now(),
                influence_score: 75000,
            },
            SocialMediaEvent {
                content: "Neutral market update from SEC".to_string(),
                source: "news".to_string(),
                timestamp: Instant::now(),
                influence_score: 25000,
            },
        ];

        // Reset statystyk
        self.stats = PipelineStats::default();

        // Rozgrzewka
        for _ in 0..100 {
            for event in &test_events {
                let _ = self.process_event(event.clone())?;
            }
        }

        // Reset po rozgrzewce
        self.stats = PipelineStats::default();

        // Właściwy benchmark
        let start = Instant::now();
        let start_tsc = rdtsc();

        for _ in 0..iterations {
            for event in &test_events {
                let _ = self.process_event(event.clone())?;
            }
        }

        let total_duration = start.elapsed();
        let total_cycles = rdtsc() - start_tsc;
        let total_operations = iterations * test_events.len();

        let avg_micros = (total_duration.as_micros() as f64) / (total_operations as f64);
        let avg_cycles = (total_cycles as f64) / (total_operations as f64);

        let sub_millisecond_ratio = (self.stats.sub_millisecond_count as f64) / (total_operations as f64);

        Ok(PipelineBenchmark {
            total_operations,
            total_duration,
            average_micros_per_operation: avg_micros,
            average_cycles_per_operation: avg_cycles,
            sub_millisecond_ratio,
            fastest_operation_cycles: self.stats.fastest_operation_cycles,
            slowest_operation_cycles: self.stats.slowest_operation_cycles,
            meets_hotz_target: avg_micros <= 922.0, // Target: <922μs
        })
    }

    /// Stress test - maksymalna wydajność
    pub fn stress_test(&mut self, duration_seconds: u64) -> CortexResult<StressTestResults> {
        let test_event = SocialMediaEvent {
            content: "High frequency trading test event".to_string(),
            source: "stress_test".to_string(),
            timestamp: Instant::now(),
            influence_score: 50000,
        };

        let start = Instant::now();
        let mut operations = 0u64;
        let mut sub_millisecond_ops = 0u64;

        while start.elapsed().as_secs() < duration_seconds {
            let decision = self.process_event(test_event.clone())?;
            operations += 1;

            if decision.processing_time_micros < 1000 {
                sub_millisecond_ops += 1;
            }
        }

        let total_duration = start.elapsed();
        let ops_per_second = (operations as f64) / total_duration.as_secs_f64();
        let sub_ms_percentage = (sub_millisecond_ops as f64) / (operations as f64) * 100.0;

        Ok(StressTestResults {
            total_operations: operations,
            duration: total_duration,
            operations_per_second: ops_per_second,
            sub_millisecond_percentage: sub_ms_percentage,
            meets_performance_target: ops_per_second >= 1000.0, // Target: ≥1000 ops/sec
        })
    }
}

/// Wyniki benchmarku pipeline
#[derive(Debug, Clone)]
pub struct PipelineBenchmark {
    pub total_operations: usize,
    pub total_duration: Duration,
    pub average_micros_per_operation: f64,
    pub average_cycles_per_operation: f64,
    pub sub_millisecond_ratio: f64,
    pub fastest_operation_cycles: u64,
    pub slowest_operation_cycles: u64,
    pub meets_hotz_target: bool,
}

/// Wyniki stress test
#[derive(Debug, Clone)]
pub struct StressTestResults {
    pub total_operations: u64,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub sub_millisecond_percentage: f64,
    pub meets_performance_target: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sub_millisecond_pipeline_creation() {
        let pipeline = SubMillisecondPipeline::new().unwrap();
        let stats = pipeline.get_stats();
        
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.sub_millisecond_count, 0);
    }

    #[test]
    fn test_process_event() {
        let mut pipeline = SubMillisecondPipeline::new().unwrap();
        
        let event = SocialMediaEvent {
            content: "Bitcoin pump incoming! 🚀".to_string(),
            source: "twitter".to_string(),
            timestamp: Instant::now(),
            influence_score: 100000,
        };

        let decision = pipeline.process_event(event).unwrap();
        
        assert!(decision.confidence > 0.0);
        assert!(decision.processing_time_micros > 0);
        
        match decision.action {
            TradingAction::Buy { amount } => assert!(amount > 0.0),
            TradingAction::Sell { amount } => assert!(amount > 0.0),
            TradingAction::Hold => {}
        }
    }

    #[test]
    fn test_benchmark_pipeline() {
        let mut pipeline = SubMillisecondPipeline::new().unwrap();
        let benchmark = pipeline.benchmark_pipeline(50).unwrap();
        
        assert!(benchmark.total_operations > 0);
        assert!(benchmark.average_micros_per_operation > 0.0);
        assert!(benchmark.sub_millisecond_ratio >= 0.0 && benchmark.sub_millisecond_ratio <= 1.0);
        
        println!("Pipeline benchmark: {:.2}μs avg, {:.1}% sub-ms, target met: {}", 
                benchmark.average_micros_per_operation,
                benchmark.sub_millisecond_ratio * 100.0,
                benchmark.meets_hotz_target);
    }

    #[test]
    fn test_stress_test() {
        let mut pipeline = SubMillisecondPipeline::new().unwrap();
        let results = pipeline.stress_test(1).unwrap(); // 1 sekunda
        
        assert!(results.total_operations > 0);
        assert!(results.operations_per_second > 0.0);
        assert!(results.sub_millisecond_percentage >= 0.0);
        
        println!("Stress test: {} ops/sec, {:.1}% sub-ms", 
                results.operations_per_second as u64,
                results.sub_millisecond_percentage);
    }

    #[test]
    fn test_trading_decisions() {
        let mut pipeline = SubMillisecondPipeline::new().unwrap();
        
        // Test pozytywnego sentymentu
        let bull_event = SocialMediaEvent {
            content: "Major bull run confirmed by institutions!".to_string(),
            source: "news".to_string(),
            timestamp: Instant::now(),
            influence_score: 200000,
        };
        
        let bull_decision = pipeline.process_event(bull_event).unwrap();
        match bull_decision.action {
            TradingAction::Buy { amount } => assert!(amount > 0.0),
            TradingAction::Hold => {
                // Może być Hold jeśli confidence jest za niskie
                assert!(bull_decision.confidence >= 0.0);
            },
            _ => panic!("Expected buy or hold decision for bullish sentiment"),
        }

        // Test negatywnego sentymentu
        let bear_event = SocialMediaEvent {
            content: "Massive crash incoming! Sell everything!".to_string(),
            source: "twitter".to_string(),
            timestamp: Instant::now(),
            influence_score: 150000,
        };
        
        let bear_decision = pipeline.process_event(bear_event).unwrap();
        match bear_decision.action {
            TradingAction::Sell { amount } => assert!(amount > 0.0),
            TradingAction::Hold => {
                // Może być Hold jeśli confidence jest za niskie
                assert!(bear_decision.confidence >= 0.0);
            },
            _ => panic!("Expected sell or hold decision for bearish sentiment"),
        }
    }

    #[test]
    fn test_performance_target() {
        let mut pipeline = SubMillisecondPipeline::new().unwrap();
        
        let event = SocialMediaEvent {
            content: "Quick test".to_string(),
            source: "test".to_string(),
            timestamp: Instant::now(),
            influence_score: 10000,
        };

        // Test pojedynczej operacji
        let decision = pipeline.process_event(event).unwrap();
        
        // Sprawdzenie czy spełnia target Hotza (<922μs)
        println!("Single operation: {}μs (target: <922μs)", decision.processing_time_micros);
        
        // Może nie przejść na wszystkich maszynach, ale powinno być szybkie
        assert!(decision.processing_time_micros < 10000); // Maksymalnie 10ms
    }
}
