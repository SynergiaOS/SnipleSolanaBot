// Neural Execution Engine Demo - Ultra-Low Latency Trading System
// Demonstrates <200μs execution with hardware-aware routing and ML optimization

use anyhow::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use overmind_protocol::neural_execution::{
    NeuralExecutionEngine, NeuralExecutionConfig, 
    ExecutionRequest, ExecutionRequestType, ExecutionPriority,
    ExecutionConstraints, HardwareRequirement, CPURequirement,
    CacheRequirement
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🧠 Starting Neural Execution Engine Demo");
    info!("🎯 Target: <200μs execution latency with ML optimization");

    // Create Neural Execution Engine configuration
    let config = NeuralExecutionConfig::default();
    
    info!("⚙️ Configuration:");
    info!("   • Neural Router: Hardware topology discovery enabled");
    info!("   • Atomic Executor: Zero-copy with SIMD optimizations");
    info!("   • Neural Predictor: ML-driven timing predictions");
    info!("   • Hardware Accelerator: FPGA/ASIC support");
    info!("   • Execution Monitor: Microsecond precision analytics");

    // Initialize Neural Execution Engine
    info!("🚀 Initializing Neural Execution Engine...");
    let engine = NeuralExecutionEngine::new(config).await?;
    
    // Start the engine
    engine.start().await?;
    info!("✅ Neural Execution Engine started successfully");

    // Run demo scenarios
    run_demo_scenarios(&engine).await?;

    // Display performance metrics
    display_performance_metrics(&engine).await?;

    // Health check
    perform_health_check(&engine).await?;

    // Stop the engine
    engine.stop().await?;
    info!("🛑 Neural Execution Engine stopped");

    Ok(())
}

async fn run_demo_scenarios(engine: &NeuralExecutionEngine) -> Result<()> {
    info!("🎭 Running Neural Execution Demo Scenarios");

    // Scenario 1: High-Priority MEV Bundle
    info!("\n📦 Scenario 1: Ultra-High Priority MEV Bundle");
    let mev_request = create_mev_bundle_request().await?;
    let start_time = std::time::Instant::now();
    
    let result = engine.execute(mev_request).await?;
    let execution_time = start_time.elapsed().as_micros();
    
    info!("✅ MEV Bundle executed in {}μs", execution_time);
    info!("   • Status: {:?}", result.status);
    info!("   • Execution Time: {:.2}μs", result.metrics.execution_time_us);
    info!("   • CPU Utilization: {:.1}%", result.metrics.hardware_utilization.cpu_utilization_percent);
    info!("   • Memory Usage: {} bytes", result.metrics.memory_usage_bytes);

    // Scenario 2: Arbitrage Execution
    info!("\n⚡ Scenario 2: High-Speed Arbitrage Execution");
    let arbitrage_request = create_arbitrage_request().await?;
    let start_time = std::time::Instant::now();
    
    let result = engine.execute(arbitrage_request).await?;
    let execution_time = start_time.elapsed().as_micros();
    
    info!("✅ Arbitrage executed in {}μs", execution_time);
    info!("   • Status: {:?}", result.status);
    info!("   • Execution Time: {:.2}μs", result.metrics.execution_time_us);
    info!("   • Hardware Efficiency: {:.1}%", 
          result.metrics.hardware_utilization.cpu_utilization_percent);

    // Scenario 3: Batch Processing
    info!("\n🔄 Scenario 3: Parallel Batch Processing");
    let batch_requests = create_batch_requests().await?;
    let start_time = std::time::Instant::now();
    
    let mut handles = Vec::new();
    for request in batch_requests {
        let handle = engine.execute(request);
        handles.push(handle);
    }
    
    let mut successful = 0;
    let mut total_time = 0.0;
    
    for handle in handles {
        match handle.await {
            Ok(result) => {
                successful += 1;
                total_time += result.metrics.execution_time_us;
            }
            Err(e) => {
                error!("Batch execution failed: {}", e);
            }
        }
    }
    
    let batch_time = start_time.elapsed().as_micros();
    let avg_time = total_time / successful as f64;
    
    info!("✅ Batch processing completed in {}μs", batch_time);
    info!("   • Successful executions: {}/5", successful);
    info!("   • Average execution time: {:.2}μs", avg_time);
    info!("   • Parallel efficiency: {:.1}%", (avg_time * 5.0 / batch_time as f64) * 100.0);

    // Scenario 4: Stress Test
    info!("\n🔥 Scenario 4: High-Frequency Stress Test");
    let stress_start = std::time::Instant::now();
    let mut stress_successful = 0;
    let mut stress_total_time = 0.0;
    
    for i in 0..100 {
        let request = create_stress_test_request(i).await?;
        match engine.execute(request).await {
            Ok(result) => {
                stress_successful += 1;
                stress_total_time += result.metrics.execution_time_us;
            }
            Err(e) => {
                warn!("Stress test iteration {} failed: {}", i, e);
            }
        }
        
        // Small delay to prevent overwhelming
        if i % 10 == 0 {
            sleep(Duration::from_micros(100)).await;
        }
    }
    
    let stress_duration = stress_start.elapsed();
    let stress_avg_time = stress_total_time / stress_successful as f64;
    let throughput = stress_successful as f64 / stress_duration.as_secs_f64();
    
    info!("✅ Stress test completed in {:.2}s", stress_duration.as_secs_f64());
    info!("   • Successful executions: {}/100", stress_successful);
    info!("   • Average execution time: {:.2}μs", stress_avg_time);
    info!("   • Throughput: {:.0} ops/s", throughput);
    info!("   • Success rate: {:.1}%", (stress_successful as f64 / 100.0) * 100.0);

    Ok(())
}

async fn create_mev_bundle_request() -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("mev_{}", Uuid::new_v4()),
        request_type: ExecutionRequestType::MEVBundle,
        priority: ExecutionPriority::UltraHigh,
        payload: vec![0xAB; 1024], // 1KB MEV bundle data
        constraints: ExecutionConstraints {
            max_latency_us: 100, // 100μs max latency
            required_hardware: vec![
                HardwareRequirement::CPU(CPURequirement {
                    min_cores: 2,
                    instruction_sets: vec!["AVX512".to_string()],
                    cache_requirements: CacheRequirement {
                        l1_cache_kb: 32,
                        l2_cache_kb: 256,
                        l3_cache_kb: 8192,
                    },
                })
            ],
            memory_requirements_bytes: 2 * 1024 * 1024, // 2MB
            cpu_requirements_cores: 2,
            gpu_requirements: None,
            atomic_execution: true,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 500_000, // 500μs deadline
    })
}

async fn create_arbitrage_request() -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("arb_{}", Uuid::new_v4()),
        request_type: ExecutionRequestType::ArbitrageExecution,
        priority: ExecutionPriority::High,
        payload: vec![0xCD; 512], // 512B arbitrage data
        constraints: ExecutionConstraints {
            max_latency_us: 200, // 200μs max latency
            required_hardware: vec![],
            memory_requirements_bytes: 1024 * 1024, // 1MB
            cpu_requirements_cores: 1,
            gpu_requirements: None,
            atomic_execution: true,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 1_000_000, // 1ms deadline
    })
}

async fn create_batch_requests() -> Result<Vec<ExecutionRequest>> {
    let mut requests = Vec::new();
    
    for i in 0..5 {
        requests.push(ExecutionRequest {
            id: format!("batch_{}_{}", i, Uuid::new_v4()),
            request_type: ExecutionRequestType::SolanaTransaction,
            priority: ExecutionPriority::Normal,
            payload: vec![0xEF; 256], // 256B transaction data
            constraints: ExecutionConstraints {
                max_latency_us: 500, // 500μs max latency
                required_hardware: vec![],
                memory_requirements_bytes: 512 * 1024, // 512KB
                cpu_requirements_cores: 1,
                gpu_requirements: None,
                atomic_execution: false,
            },
            timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
            deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 2_000_000, // 2ms deadline
        });
    }
    
    Ok(requests)
}

async fn create_stress_test_request(index: usize) -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("stress_{}_{}", index, Uuid::new_v4()),
        request_type: ExecutionRequestType::MarketMaking,
        priority: ExecutionPriority::Normal,
        payload: vec![0x42; 128], // 128B market making data
        constraints: ExecutionConstraints {
            max_latency_us: 300, // 300μs max latency
            required_hardware: vec![],
            memory_requirements_bytes: 256 * 1024, // 256KB
            cpu_requirements_cores: 1,
            gpu_requirements: None,
            atomic_execution: false,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 1_000_000, // 1ms deadline
    })
}

async fn display_performance_metrics(engine: &NeuralExecutionEngine) -> Result<()> {
    info!("\n📊 Neural Execution Engine Performance Metrics");
    
    let metrics = engine.get_metrics().await;
    
    info!("🎯 Execution Metrics:");
    info!("   • Total Executions: {}", metrics.total_executions);
    info!("   • Successful Executions: {}", metrics.successful_executions);
    info!("   • Average Execution Time: {:.2}μs", metrics.avg_execution_time_us);
    info!("   • 99th Percentile Latency: {:.2}μs", metrics.p99_latency_us);
    info!("   • Throughput: {:.0} ops/s", metrics.throughput_ops_s);
    info!("   • Error Rate: {:.3}%", metrics.error_rate_percent);
    
    info!("🔧 Hardware Efficiency:");
    info!("   • Hardware Efficiency: {:.1}%", metrics.hardware_efficiency_percent);
    
    // Performance targets validation
    info!("\n🎯 Performance Target Validation:");
    let target_met = metrics.avg_execution_time_us < 200.0;
    let status = if target_met { "✅ MET" } else { "❌ NOT MET" };
    info!("   • Target Latency (<200μs): {} ({:.2}μs)", status, metrics.avg_execution_time_us);
    
    let throughput_met = metrics.throughput_ops_s > 1000.0;
    let throughput_status = if throughput_met { "✅ MET" } else { "❌ NOT MET" };
    info!("   • Target Throughput (>1K ops/s): {} ({:.0} ops/s)", throughput_status, metrics.throughput_ops_s);
    
    let error_met = metrics.error_rate_percent < 0.1;
    let error_status = if error_met { "✅ MET" } else { "❌ NOT MET" };
    info!("   • Target Error Rate (<0.1%): {} ({:.3}%)", error_status, metrics.error_rate_percent);

    Ok(())
}

async fn perform_health_check(engine: &NeuralExecutionEngine) -> Result<()> {
    info!("\n🏥 Neural Execution Engine Health Check");
    
    let health = engine.health_check().await?;
    
    info!("🔍 Component Health Status:");
    info!("   • Neural Router: {:?} ({:.2}μs latency)", 
          health.neural_router.status, health.neural_router.latency_us);
    info!("   • Atomic Executor: {:?} ({:.2}μs latency)", 
          health.atomic_executor.status, health.atomic_executor.latency_us);
    info!("   • Neural Predictor: {:?} ({:.2}μs latency)", 
          health.neural_predictor.status, health.neural_predictor.latency_us);
    info!("   • Hardware Accelerator: {:?} ({:.2}μs latency)", 
          health.hardware_accelerator.status, health.hardware_accelerator.latency_us);
    info!("   • Execution Monitor: {:?} ({:.2}μs latency)", 
          health.execution_monitor.status, health.execution_monitor.latency_us);
    
    // Overall health assessment
    let all_healthy = [
        &health.neural_router,
        &health.atomic_executor,
        &health.neural_predictor,
        &health.hardware_accelerator,
        &health.execution_monitor,
    ].iter().all(|component| matches!(component.status, overmind_protocol::neural_execution::HealthStatus::Healthy));
    
    if all_healthy {
        info!("🟢 Overall System Health: EXCELLENT");
    } else {
        warn!("🟡 Overall System Health: DEGRADED - Some components need attention");
    }

    Ok(())
}
