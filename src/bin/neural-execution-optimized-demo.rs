// Neural Execution Engine Optimized Demo - Software-Only Ultra-Low Latency
// Target: <200μs execution with CPU/RAM optimization only (no FPGA/ASIC)

use anyhow::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{info, warn, error};
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

    info!("🧠 Starting Neural Execution Engine OPTIMIZED Demo");
    info!("🎯 Target: <200μs execution latency (SOFTWARE ONLY)");
    info!("💰 Budget-friendly: CPU/RAM optimization without FPGA/ASIC");

    // Create optimized configuration
    let config = NeuralExecutionConfig::default();
    
    info!("⚙️ Optimized Configuration:");
    info!("   • Neural Router: 5μs optimization interval");
    info!("   • Atomic Executor: AVX512 SIMD, 4-stage pipeline");
    info!("   • Neural Predictor: 32-feature vectors, training disabled");
    info!("   • Hardware Accelerator: Software-only mode");
    info!("   • Execution Monitor: Reduced overhead monitoring");

    // Initialize Neural Execution Engine
    info!("🚀 Initializing Optimized Neural Execution Engine...");
    let engine = NeuralExecutionEngine::new(config).await?;
    
    // Start the engine
    engine.start().await?;
    info!("✅ Optimized Neural Execution Engine started");

    // Run optimized demo scenarios
    run_optimized_scenarios(&engine).await?;

    // Display performance metrics
    display_optimized_metrics(&engine).await?;

    // Health check
    perform_optimized_health_check(&engine).await?;

    // Stop the engine
    engine.stop().await?;
    info!("🛑 Optimized Neural Execution Engine stopped");

    Ok(())
}

async fn run_optimized_scenarios(engine: &NeuralExecutionEngine) -> Result<()> {
    info!("🎭 Running Optimized Demo Scenarios (Software-Only)");

    // Scenario 1: Micro MEV Bundle (tiny payload)
    info!("\n📦 Scenario 1: Micro MEV Bundle (64 bytes)");
    let micro_mev = create_micro_mev_request().await?;
    let start_time = std::time::Instant::now();
    
    let result = engine.execute(micro_mev).await?;
    let execution_time = start_time.elapsed().as_micros();
    
    info!("✅ Micro MEV executed in {}μs", execution_time);
    info!("   • Status: {:?}", result.status);
    info!("   • Execution Time: {:.2}μs", result.metrics.execution_time_us);
    info!("   • CPU Utilization: {:.1}%", result.metrics.hardware_utilization.cpu_utilization_percent);

    // Scenario 2: Lightning Arbitrage (minimal payload)
    info!("\n⚡ Scenario 2: Lightning Arbitrage (32 bytes)");
    let lightning_arb = create_lightning_arbitrage().await?;
    let start_time = std::time::Instant::now();
    
    let result = engine.execute(lightning_arb).await?;
    let execution_time = start_time.elapsed().as_micros();
    
    info!("✅ Lightning Arbitrage executed in {}μs", execution_time);
    info!("   • Status: {:?}", result.status);
    info!("   • Execution Time: {:.2}μs", result.metrics.execution_time_us);

    // Scenario 3: Burst Processing (10 tiny requests)
    info!("\n🔄 Scenario 3: Burst Processing (10x16 bytes)");
    let burst_start = std::time::Instant::now();
    let mut burst_successful = 0;
    let mut burst_total_time = 0.0;
    
    for i in 0..10 {
        let request = create_burst_request(i).await?;
        match engine.execute(request).await {
            Ok(result) => {
                burst_successful += 1;
                burst_total_time += result.metrics.execution_time_us;
            }
            Err(e) => {
                error!("Burst request {} failed: {}", i, e);
            }
        }
    }
    
    let burst_duration = burst_start.elapsed().as_micros();
    let burst_avg = burst_total_time / burst_successful as f64;
    
    info!("✅ Burst processing completed in {}μs", burst_duration);
    info!("   • Successful: {}/10", burst_successful);
    info!("   • Average execution time: {:.2}μs", burst_avg);
    info!("   • Total throughput: {:.0} ops/s", 10.0 / (burst_duration as f64 / 1_000_000.0));

    // Scenario 4: Sustained Load Test (50 requests)
    info!("\n🔥 Scenario 4: Sustained Load Test (50x8 bytes)");
    let load_start = std::time::Instant::now();
    let mut load_successful = 0;
    let mut load_total_time = 0.0;
    let mut min_time = f64::MAX;
    let mut max_time: f64 = 0.0;
    
    for i in 0..50 {
        let request = create_load_test_request(i).await?;
        match engine.execute(request).await {
            Ok(result) => {
                load_successful += 1;
                let exec_time = result.metrics.execution_time_us;
                load_total_time += exec_time;
                min_time = min_time.min(exec_time);
                max_time = max_time.max(exec_time);
            }
            Err(e) => {
                warn!("Load test {} failed: {}", i, e);
            }
        }
        
        // Tiny delay to prevent overwhelming
        if i % 5 == 0 {
            sleep(Duration::from_micros(10)).await;
        }
    }
    
    let load_duration = load_start.elapsed();
    let load_avg = load_total_time / load_successful as f64;
    let throughput = load_successful as f64 / load_duration.as_secs_f64();
    
    info!("✅ Sustained load test completed in {:.2}s", load_duration.as_secs_f64());
    info!("   • Successful: {}/50", load_successful);
    info!("   • Average execution time: {:.2}μs", load_avg);
    info!("   • Min/Max execution time: {:.2}μs / {:.2}μs", min_time, max_time);
    info!("   • Sustained throughput: {:.0} ops/s", throughput);
    info!("   • Success rate: {:.1}%", (load_successful as f64 / 50.0) * 100.0);

    Ok(())
}

async fn create_micro_mev_request() -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("micro_mev_{}", Uuid::new_v4()),
        request_type: ExecutionRequestType::MEVBundle,
        priority: ExecutionPriority::UltraHigh,
        payload: vec![0xAB; 64], // Tiny 64-byte payload
        constraints: ExecutionConstraints {
            max_latency_us: 50, // 50μs max latency
            required_hardware: vec![
                HardwareRequirement::CPU(CPURequirement {
                    min_cores: 1,
                    instruction_sets: vec!["AVX512".to_string()],
                    cache_requirements: CacheRequirement {
                        l1_cache_kb: 32,
                        l2_cache_kb: 256,
                        l3_cache_kb: 8192,
                    },
                })
            ],
            memory_requirements_bytes: 128 * 1024, // 128KB
            cpu_requirements_cores: 1,
            gpu_requirements: None,
            atomic_execution: true,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 100_000, // 100μs deadline
    })
}

async fn create_lightning_arbitrage() -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("lightning_arb_{}", Uuid::new_v4()),
        request_type: ExecutionRequestType::ArbitrageExecution,
        priority: ExecutionPriority::High,
        payload: vec![0xCD; 32], // Tiny 32-byte payload
        constraints: ExecutionConstraints {
            max_latency_us: 100, // 100μs max latency
            required_hardware: vec![],
            memory_requirements_bytes: 64 * 1024, // 64KB
            cpu_requirements_cores: 1,
            gpu_requirements: None,
            atomic_execution: true,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 200_000, // 200μs deadline
    })
}

async fn create_burst_request(index: usize) -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("burst_{}_{}", index, Uuid::new_v4()),
        request_type: ExecutionRequestType::SolanaTransaction,
        priority: ExecutionPriority::Normal,
        payload: vec![0xEF; 16], // Tiny 16-byte payload
        constraints: ExecutionConstraints {
            max_latency_us: 150, // 150μs max latency
            required_hardware: vec![],
            memory_requirements_bytes: 32 * 1024, // 32KB
            cpu_requirements_cores: 1,
            gpu_requirements: None,
            atomic_execution: false,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 300_000, // 300μs deadline
    })
}

async fn create_load_test_request(index: usize) -> Result<ExecutionRequest> {
    Ok(ExecutionRequest {
        id: format!("load_{}_{}", index, Uuid::new_v4()),
        request_type: ExecutionRequestType::MarketMaking,
        priority: ExecutionPriority::Normal,
        payload: vec![0x42; 8], // Ultra-tiny 8-byte payload
        constraints: ExecutionConstraints {
            max_latency_us: 200, // 200μs max latency
            required_hardware: vec![],
            memory_requirements_bytes: 16 * 1024, // 16KB
            cpu_requirements_cores: 1,
            gpu_requirements: None,
            atomic_execution: false,
        },
        timestamp_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64,
        deadline_ns: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64 + 400_000, // 400μs deadline
    })
}

async fn display_optimized_metrics(engine: &NeuralExecutionEngine) -> Result<()> {
    info!("\n📊 Optimized Neural Execution Engine Metrics");
    
    let metrics = engine.get_metrics().await;
    
    info!("🎯 Execution Metrics:");
    info!("   • Total Executions: {}", metrics.total_executions);
    info!("   • Successful Executions: {}", metrics.successful_executions);
    info!("   • Average Execution Time: {:.2}μs", metrics.avg_execution_time_us);
    info!("   • 99th Percentile Latency: {:.2}μs", metrics.p99_latency_us);
    info!("   • Throughput: {:.0} ops/s", metrics.throughput_ops_s);
    info!("   • Error Rate: {:.3}%", metrics.error_rate_percent);
    
    info!("🔧 Software Efficiency:");
    info!("   • Hardware Efficiency: {:.1}%", metrics.hardware_efficiency_percent);
    
    // Performance targets validation
    info!("\n🎯 Software-Only Performance Validation:");
    let target_met = metrics.avg_execution_time_us < 200.0;
    let status = if target_met { "✅ MET" } else { "🔄 IMPROVING" };
    info!("   • Target Latency (<200μs): {} ({:.2}μs)", status, metrics.avg_execution_time_us);
    
    let throughput_met = metrics.throughput_ops_s > 1000.0;
    let throughput_status = if throughput_met { "✅ MET" } else { "🔄 IMPROVING" };
    info!("   • Target Throughput (>1K ops/s): {} ({:.0} ops/s)", throughput_status, metrics.throughput_ops_s);
    
    let error_met = metrics.error_rate_percent < 1.0;
    let error_status = if error_met { "✅ MET" } else { "🔄 IMPROVING" };
    info!("   • Target Error Rate (<1%): {} ({:.3}%)", error_status, metrics.error_rate_percent);

    // Show improvement suggestions
    if !target_met || !throughput_met {
        info!("\n💡 Software Optimization Suggestions:");
        if !target_met {
            info!("   • Reduce payload sizes further");
            info!("   • Enable CPU affinity pinning");
            info!("   • Use memory pools for zero-copy");
        }
        if !throughput_met {
            info!("   • Increase parallel processing");
            info!("   • Optimize SIMD operations");
            info!("   • Reduce monitoring overhead");
        }
    }

    Ok(())
}

async fn perform_optimized_health_check(engine: &NeuralExecutionEngine) -> Result<()> {
    info!("\n🏥 Optimized System Health Check");
    
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
    let healthy_count = [
        &health.neural_router,
        &health.atomic_executor,
        &health.neural_predictor,
        &health.hardware_accelerator,
        &health.execution_monitor,
    ].iter().filter(|component| matches!(component.status, overmind_protocol::neural_execution::HealthStatus::Healthy)).count();
    
    match healthy_count {
        5 => info!("🟢 Overall System Health: EXCELLENT (All components healthy)"),
        3..=4 => info!("🟡 Overall System Health: GOOD ({}/5 components healthy)", healthy_count),
        1..=2 => warn!("🟠 Overall System Health: DEGRADED ({}/5 components healthy)", healthy_count),
        0 => error!("🔴 Overall System Health: CRITICAL (No healthy components)"),
        _ => unreachable!(),
    }

    Ok(())
}
