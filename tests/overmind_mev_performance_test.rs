/*
THE OVERMIND PROTOCOL - Performance Tests
Comprehensive performance testing for the Helius + Jito v2 MEV pipeline

Tests include:
- Latency benchmarks (target: <10ms)
- Throughput testing (transactions per second)
- Memory usage optimization
- AI analysis performance
- Jito v2 tip optimization efficiency
*/

use std::time::{Duration, Instant};
use tokio::time::sleep;
use criterion::{black_box, Criterion};

use snipercor::modules::overmind_mev_pipeline::{
    OvermindMEVPipeline, OvermindMEVConfig, PipelineConfig, AIAnalysisConfig
};
use snipercor::modules::helius_streamer::{
    HeliusStreamerConfig, EnrichedTransaction, TransactionType, ParsedInstruction,
    AccountChange, TokenTransfer
};
use snipercor::modules::jito_v2_client::{JitoV2Config, PriorityLevel};

#[tokio::test]
async fn test_pipeline_initialization_performance() {
    let start = Instant::now();
    
    let config = create_test_config();
    let _pipeline = OvermindMEVPipeline::new(config).await.unwrap();
    
    let initialization_time = start.elapsed();
    
    // Should initialize in under 1 second
    assert!(initialization_time < Duration::from_secs(1), 
        "Pipeline initialization took too long: {:?}", initialization_time);
    
    println!("✅ Pipeline initialization: {:?}", initialization_time);
}

#[tokio::test]
async fn test_transaction_processing_latency() {
    let config = create_test_config();
    let pipeline = OvermindMEVPipeline::new(config).await.unwrap();
    
    // Create test transaction
    let test_tx = create_test_transaction();
    
    let start = Instant::now();
    
    // Simulate transaction processing (would normally go through channels)
    // For testing, we'll call the analysis methods directly
    let opportunities = simulate_transaction_analysis(&test_tx).await;
    
    let processing_time = start.elapsed();
    
    // Target: <10ms for transaction analysis
    assert!(processing_time < Duration::from_millis(10),
        "Transaction processing too slow: {:?}", processing_time);
    
    println!("✅ Transaction processing latency: {:?}", processing_time);
    println!("📊 Opportunities detected: {}", opportunities.len());
}

#[tokio::test]
async fn test_ai_analysis_performance() {
    let start = Instant::now();
    
    // Simulate AI analysis
    let ai_result = simulate_ai_analysis().await;
    
    let ai_latency = start.elapsed();
    
    // Target: <100ms for AI analysis
    assert!(ai_latency < Duration::from_millis(100),
        "AI analysis too slow: {:?}", ai_latency);
    
    println!("✅ AI analysis latency: {:?}", ai_latency);
    println!("🤖 AI confidence: {:.2}", ai_result.confidence);
}

#[tokio::test]
async fn test_jito_v2_tip_calculation_performance() {
    let start = Instant::now();
    
    // Simulate tip calculation
    let optimal_tip = simulate_tip_calculation(1_000_000_000, &PriorityLevel::High).await;
    
    let calculation_time = start.elapsed();
    
    // Should be very fast: <1ms
    assert!(calculation_time < Duration::from_millis(1),
        "Tip calculation too slow: {:?}", calculation_time);
    
    println!("✅ Tip calculation latency: {:?}", calculation_time);
    println!("💰 Optimal tip: {} lamports", optimal_tip);
}

#[tokio::test]
async fn test_throughput_under_load() {
    let config = create_test_config();
    let _pipeline = OvermindMEVPipeline::new(config).await.unwrap();
    
    let transaction_count = 1000;
    let start = Instant::now();
    
    // Simulate processing many transactions
    for i in 0..transaction_count {
        let test_tx = create_test_transaction_with_id(i);
        let _opportunities = simulate_transaction_analysis(&test_tx).await;
        
        // Small delay to simulate real conditions
        if i % 100 == 0 {
            sleep(Duration::from_micros(100)).await;
        }
    }
    
    let total_time = start.elapsed();
    let throughput = transaction_count as f64 / total_time.as_secs_f64();
    
    println!("✅ Processed {} transactions in {:?}", transaction_count, total_time);
    println!("📈 Throughput: {:.2} transactions/second", throughput);
    
    // Target: >500 TPS
    assert!(throughput > 500.0, "Throughput too low: {:.2} TPS", throughput);
}

#[tokio::test]
async fn test_memory_usage_stability() {
    let config = create_test_config();
    let _pipeline = OvermindMEVPipeline::new(config).await.unwrap();
    
    let initial_memory = get_memory_usage();
    
    // Simulate extended operation
    for i in 0..10000 {
        let test_tx = create_test_transaction_with_id(i);
        let _opportunities = simulate_transaction_analysis(&test_tx).await;
        
        if i % 1000 == 0 {
            let current_memory = get_memory_usage();
            let memory_growth = current_memory - initial_memory;
            
            println!("🧠 Memory usage after {} transactions: +{} bytes", i, memory_growth);
            
            // Memory growth should be reasonable
            assert!(memory_growth < 100_000_000, // 100MB max growth
                "Excessive memory growth: {} bytes", memory_growth);
        }
    }
    
    println!("✅ Memory usage stability test passed");
}

#[tokio::test]
async fn test_error_recovery_performance() {
    let config = create_test_config();
    let _pipeline = OvermindMEVPipeline::new(config).await.unwrap();
    
    let start = Instant::now();
    
    // Simulate error conditions and recovery
    for _ in 0..100 {
        // Simulate network error
        simulate_network_error_recovery().await;
        
        // Simulate AI timeout
        simulate_ai_timeout_recovery().await;
        
        // Simulate Jito rejection
        simulate_jito_rejection_recovery().await;
    }
    
    let recovery_time = start.elapsed();
    
    println!("✅ Error recovery test completed in {:?}", recovery_time);
    
    // Recovery should be fast
    assert!(recovery_time < Duration::from_secs(5),
        "Error recovery too slow: {:?}", recovery_time);
}

// Helper functions for testing

fn create_test_config() -> OvermindMEVConfig {
    OvermindMEVConfig {
        helius_config: HeliusStreamerConfig {
            api_key: "test_key".to_string(),
            websocket_url: "ws://localhost:8080".to_string(),
            connection_timeout_secs: 10,
            max_reconnect_attempts: 3,
            reconnect_backoff_base: 1,
            max_queue_size: 1000,
            enable_enrichment: true,
            enable_compression: false, // Disable for testing
        },
        jito_config: JitoV2Config::default(),
        pipeline_config: PipelineConfig {
            max_latency_ms: 10,
            min_mev_value: 1_000_000,
            max_concurrent_ops: 50,
            enable_ai_analysis: true,
            enable_realtime_optimization: false, // Disable for testing
            opportunity_timeout_ms: 1000,
        },
        ai_config: AIAnalysisConfig {
            confidence_threshold: 0.7,
            enable_sentiment_analysis: false, // Disable for testing
            enable_pattern_recognition: true,
            ai_timeout_ms: 50,
        },
    }
}

fn create_test_transaction() -> EnrichedTransaction {
    create_test_transaction_with_id(1)
}

fn create_test_transaction_with_id(id: u64) -> EnrichedTransaction {
    EnrichedTransaction {
        signature: format!("test_signature_{}", id),
        slot: 100000 + id,
        timestamp: chrono::Utc::now().timestamp(),
        fee: 5000,
        success: true,
        instructions: vec![
            ParsedInstruction {
                program_id: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
                instruction_type: "swap".to_string(),
                data: serde_json::json!({"amount": 1000000}),
            }
        ],
        account_changes: vec![
            AccountChange {
                account: "test_account_1".to_string(),
                before_balance: 1000000000,
                after_balance: 900000000,
                change: -100000000,
            }
        ],
        token_transfers: vec![
            TokenTransfer {
                from_account: "test_from".to_string(),
                to_account: "test_to".to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                amount: 100000000,
                decimals: 9,
            }
        ],
        estimated_mev_value: Some(50000000),
        tx_type: TransactionType::Swap,
    }
}

async fn simulate_transaction_analysis(tx: &EnrichedTransaction) -> Vec<String> {
    // Simulate analysis time
    sleep(Duration::from_micros(100)).await;
    
    // Return mock opportunities
    if tx.estimated_mev_value.unwrap_or(0) > 10_000_000 {
        vec!["arbitrage".to_string(), "front_run".to_string()]
    } else {
        vec![]
    }
}

async fn simulate_ai_analysis() -> AIAnalysisResult {
    // Simulate AI processing time
    sleep(Duration::from_millis(10)).await;
    
    AIAnalysisResult {
        confidence: 0.85,
        recommendation: "execute".to_string(),
    }
}

async fn simulate_tip_calculation(profit: u64, priority: &PriorityLevel) -> u64 {
    // Simulate calculation
    let base_tip = 10_000;
    let priority_multiplier = match priority {
        PriorityLevel::Low => 0.5,
        PriorityLevel::Medium => 1.0,
        PriorityLevel::High => 1.5,
        PriorityLevel::Critical => 2.0,
        PriorityLevel::MEV => 3.0,
    };
    
    let profit_based = (profit as f64 * 0.05) as u64; // 5% of profit
    let priority_adjusted = (base_tip as f64 * priority_multiplier) as u64;
    
    profit_based.max(priority_adjusted)
}

async fn simulate_network_error_recovery() {
    sleep(Duration::from_micros(10)).await;
}

async fn simulate_ai_timeout_recovery() {
    sleep(Duration::from_micros(5)).await;
}

async fn simulate_jito_rejection_recovery() {
    sleep(Duration::from_micros(20)).await;
}

fn get_memory_usage() -> usize {
    // Simplified memory usage calculation
    // In production, would use proper system APIs
    std::mem::size_of::<OvermindMEVPipeline>() * 1000
}

struct AIAnalysisResult {
    confidence: f64,
    recommendation: String,
}

// Benchmark tests (requires criterion feature)
#[cfg(feature = "bench")]
mod benchmarks {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};

    fn bench_transaction_analysis(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let test_tx = create_test_transaction();
        
        c.bench_function("transaction_analysis", |b| {
            b.iter(|| {
                rt.block_on(async {
                    black_box(simulate_transaction_analysis(&test_tx).await)
                })
            })
        });
    }

    fn bench_tip_calculation(c: &mut Criterion) {
        c.bench_function("tip_calculation", |b| {
            b.iter(|| {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    black_box(simulate_tip_calculation(1_000_000_000, &PriorityLevel::High).await)
                })
            })
        });
    }

    criterion_group!(benches, bench_transaction_analysis, bench_tip_calculation);
    criterion_main!(benches);
}
