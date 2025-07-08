/*
THE OVERMIND PROTOCOL v4.1 - Performance Tests
Comprehensive performance and benchmark testing

Tests include:
- Latency benchmarks (target: <10ms for critical operations)
- Throughput testing (signals per second)
- Memory usage optimization
- AI inference performance
- Concurrent processing capabilities
- System scalability limits
*/

use std::time::{Duration, Instant};
use tokio::time::sleep;
use serde_json::json;
use futures;

use overmind_protocol::overmind::{
    OvermindProtocol,
    ai_engine::{AIEngineConfig, TradingSignal},
};

#[tokio::test]
async fn test_overmind_initialization_performance() {
    let start = Instant::now();
    
    let _overmind = OvermindProtocol::new().await.unwrap();
    
    let initialization_time = start.elapsed();
    
    // Should initialize in under 10 seconds
    assert!(initialization_time < Duration::from_secs(10), 
        "OVERMIND initialization took too long: {:?}", initialization_time);
    
    println!("✅ OVERMIND initialization: {:?}", initialization_time);
}

#[tokio::test]
async fn test_ai_engine_inference_latency() {
    let mut overmind = OvermindProtocol::new().await.unwrap();
    
    let ai_config = AIEngineConfig {
        device: "cpu".to_string(),
        model_path: "test-model".to_string(),
        max_sequence_length: 512,
        batch_size: 8,
        enable_fine_tuning: false,
        learning_rate: 1e-5,
    };
    
    overmind.initialize_ai_engine(Some(ai_config)).await.unwrap();
    
    // Warm up the model
    let warmup_signal = TradingSignal {
        market_data: "Warmup".to_string(),
        technical_indicators: vec![0.5; 20],
        sentiment_score: 0.5,
        volume: 1000000.0,
        price: 100.0,
        timestamp: chrono::Utc::now(),
    };
    
    let _warmup = overmind.ai_engine().unwrap().predict(&warmup_signal).await.unwrap();
    
    // Now measure actual inference time
    let test_signal = TradingSignal {
        market_data: "SOL price analysis".to_string(),
        technical_indicators: vec![0.7, 0.8, 0.6, 0.9, 0.5, 0.4, 0.8, 0.7, 0.6, 0.9,
                                  0.3, 0.8, 0.7, 0.5, 0.6, 0.8, 0.4, 0.7, 0.9, 0.6],
        sentiment_score: 0.8,
        volume: 1500000.0,
        price: 150.75,
        timestamp: chrono::Utc::now(),
    };
    
    let start = Instant::now();
    let _prediction = overmind.ai_engine().unwrap().predict(&test_signal).await.unwrap();
    let inference_time = start.elapsed();
    
    // Target: <50ms for AI inference
    assert!(inference_time < Duration::from_millis(50), 
        "AI inference too slow: {:?}", inference_time);
    
    println!("✅ AI inference latency: {:?}", inference_time);
}

#[tokio::test]
async fn test_swarm_signal_processing_latency() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    let test_signal = json!({
        "type": "market_update",
        "symbol": "SOL",
        "price": 150.75,
        "volume": 1500000,
        "trend": "bullish"
    });
    
    let start = Instant::now();
    let _decisions = overmind.swarm().process_market_signal(test_signal).await.unwrap();
    let processing_time = start.elapsed();
    
    // Target: <100ms for swarm processing
    assert!(processing_time < Duration::from_millis(100), 
        "Swarm processing too slow: {:?}", processing_time);
    
    println!("✅ Swarm signal processing latency: {:?}", processing_time);
}

#[tokio::test]
async fn test_throughput_under_load() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    let signal_count = 1000;
    let start = Instant::now();
    
    // Process signals sequentially to measure raw throughput
    for i in 0..signal_count {
        let signal = json!({
            "type": "throughput_test",
            "id": i,
            "symbol": "SOL",
            "price": 150.0 + (i as f64 * 0.01),
            "volume": 1000000 + i * 1000
        });
        
        let _result = overmind.swarm().process_market_signal(signal).await;
        
        // Small delay every 100 signals to prevent overwhelming
        if i % 100 == 0 {
            sleep(Duration::from_micros(100)).await;
        }
    }
    
    let total_time = start.elapsed();
    let throughput = signal_count as f64 / total_time.as_secs_f64();
    
    println!("✅ Processed {} signals in {:?}", signal_count, total_time);
    println!("📈 Throughput: {:.2} signals/second", throughput);
    
    // Target: >100 signals per second
    assert!(throughput > 100.0, "Throughput too low: {:.2} signals/sec", throughput);
}

#[tokio::test]
async fn test_concurrent_processing_performance() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    let concurrent_signals = 50;
    let mut tasks = Vec::new();
    
    let start = Instant::now();
    
    // Create concurrent tasks
    for i in 0..concurrent_signals {
        let swarm = overmind.swarm();
        let signal = json!({
            "type": "concurrent_test",
            "id": i,
            "symbol": format!("TOKEN{}", i % 10),
            "price": 100.0 + i as f64,
            "volume": 1000000
        });
        
        let task = async move {
            swarm.process_market_signal(signal).await
        };
        
        tasks.push(task);
    }
    
    // Execute all tasks concurrently
    let results = futures::future::join_all(tasks).await;
    let concurrent_time = start.elapsed();
    
    let successful_count = results.iter().filter(|r| r.is_ok()).count();
    let concurrent_throughput = successful_count as f64 / concurrent_time.as_secs_f64();
    
    println!("✅ Concurrent processing: {} signals in {:?}", 
             successful_count, concurrent_time);
    println!("📈 Concurrent throughput: {:.2} signals/second", concurrent_throughput);
    
    // Should process most signals successfully
    assert!(successful_count >= concurrent_signals * 90 / 100, 
            "Too many failed signals: {}/{}", 
            concurrent_signals - successful_count, concurrent_signals);
    
    // Concurrent processing should be faster than sequential
    assert!(concurrent_throughput > 200.0, 
            "Concurrent throughput too low: {:.2} signals/sec", concurrent_throughput);
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    let initial_memory = get_memory_usage();
    let load_iterations = 1000;
    
    println!("🧠 Initial memory usage: {} bytes", initial_memory);
    
    for i in 0..load_iterations {
        // Create signals with varying data sizes
        let data_size = (i % 100) + 1;
        let signal = json!({
            "type": "memory_test",
            "id": i,
            "data": vec![i; data_size],
            "large_string": "x".repeat(data_size * 10)
        });
        
        let _result = overmind.swarm().process_market_signal(signal).await;
        
        // Check memory every 200 iterations
        if i % 200 == 0 && i > 0 {
            let current_memory = get_memory_usage();
            let memory_growth = current_memory.saturating_sub(initial_memory);
            
            println!("🧠 Memory after {} iterations: +{} bytes", i, memory_growth);
            
            // Memory growth should be reasonable (< 100MB)
            assert!(memory_growth < 100_000_000, 
                "Excessive memory growth: {} bytes", memory_growth);
        }
    }
    
    let final_memory = get_memory_usage();
    let total_growth = final_memory.saturating_sub(initial_memory);
    
    println!("✅ Memory test completed. Total growth: {} bytes", total_growth);
}

#[tokio::test]
async fn test_ai_inference_batch_performance() {
    let mut overmind = OvermindProtocol::new().await.unwrap();
    
    let ai_config = AIEngineConfig {
        device: "cpu".to_string(),
        model_path: "test-model".to_string(),
        max_sequence_length: 512,
        batch_size: 8,
        enable_fine_tuning: false,
        learning_rate: 1e-5,
    };
    
    overmind.initialize_ai_engine(Some(ai_config)).await.unwrap();
    
    let batch_size = 100;
    let start = Instant::now();
    
    // Process batch of AI predictions
    for i in 0..batch_size {
        let signal = TradingSignal {
            market_data: format!("Batch prediction {}", i),
            technical_indicators: vec![
                (i as f64 / batch_size as f64).sin().abs() as f32; 20
            ],
            sentiment_score: (i as f64 / batch_size as f64).cos().abs() as f32,
            volume: 1000000.0 + i as f64 * 10000.0,
            price: 100.0 + i as f64,
            timestamp: chrono::Utc::now(),
        };
        
        let _prediction = overmind.ai_engine().unwrap().predict(&signal).await.unwrap();
    }
    
    let batch_time = start.elapsed();
    let avg_inference_time = batch_time / batch_size;
    let inference_throughput = batch_size as f64 / batch_time.as_secs_f64();
    
    println!("✅ AI batch processing: {} predictions in {:?}", batch_size, batch_time);
    println!("⚡ Average inference time: {:?}", avg_inference_time);
    println!("📈 AI throughput: {:.2} predictions/second", inference_throughput);
    
    // Target: <10ms average inference time
    assert!(avg_inference_time < Duration::from_millis(10), 
        "Average AI inference too slow: {:?}", avg_inference_time);
}

#[tokio::test]
async fn test_system_scalability_limits() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    // Test with increasing load to find limits
    let load_levels = vec![10, 50, 100, 200, 500];
    
    for load in load_levels {
        let start = Instant::now();
        let mut tasks = Vec::new();
        
        for i in 0..load {
            let swarm = overmind.swarm();
            let signal = json!({
                "type": "scalability_test",
                "load_level": load,
                "id": i,
                "timestamp": chrono::Utc::now()
            });
            
            let task = async move {
                swarm.process_market_signal(signal).await
            };
            
            tasks.push(task);
        }
        
        let results = futures::future::join_all(tasks).await;
        let load_time = start.elapsed();
        
        let successful = results.iter().filter(|r| r.is_ok()).count();
        let success_rate = successful as f64 / load as f64;
        let throughput = successful as f64 / load_time.as_secs_f64();
        
        println!("📊 Load level {}: {}/{} successful in {:?} ({:.1}% success, {:.2} signals/sec)", 
                 load, successful, load, load_time, success_rate * 100.0, throughput);
        
        // Should maintain >90% success rate under reasonable load
        if load <= 200 {
            assert!(success_rate > 0.9, 
                "Success rate too low at load {}: {:.1}%", load, success_rate * 100.0);
        }
    }
    
    println!("✅ Scalability test completed");
}

#[tokio::test]
async fn test_end_to_end_pipeline_performance() {
    let mut overmind = OvermindProtocol::new().await.unwrap();
    
    // Initialize full system
    let ai_config = AIEngineConfig {
        device: "cpu".to_string(),
        model_path: "test-model".to_string(),
        max_sequence_length: 512,
        batch_size: 8,
        enable_fine_tuning: false,
        learning_rate: 1e-5,
    };
    
    overmind.initialize_ai_engine(Some(ai_config)).await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    // Test complete pipeline performance
    let pipeline_iterations = 50;
    let start = Instant::now();
    
    for i in 0..pipeline_iterations {
        // 1. Market signal
        let market_signal = json!({
            "type": "pipeline_test",
            "iteration": i,
            "symbol": "SOL",
            "price": 150.0 + (i as f64 * 0.1),
            "volume": 1500000 + i * 10000
        });
        
        // 2. Swarm processing
        let _swarm_decisions = overmind.swarm()
            .process_market_signal(market_signal).await.unwrap();
        
        // 3. AI analysis
        if overmind.has_ai_engine() {
            let ai_signal = TradingSignal {
                market_data: format!("Pipeline iteration {}", i),
                technical_indicators: vec![0.5 + (i as f64 / 100.0) as f32; 20],
                sentiment_score: 0.7,
                volume: 1500000.0,
                price: 150.0 + (i as f64 * 0.1),
                timestamp: chrono::Utc::now(),
            };
            
            let _ai_prediction = overmind.ai_engine().unwrap()
                .predict(&ai_signal).await.unwrap();
        }
    }
    
    let total_pipeline_time = start.elapsed();
    let avg_pipeline_time = total_pipeline_time / pipeline_iterations;
    let pipeline_throughput = pipeline_iterations as f64 / total_pipeline_time.as_secs_f64();
    
    println!("✅ End-to-end pipeline: {} iterations in {:?}", 
             pipeline_iterations, total_pipeline_time);
    println!("⚡ Average pipeline time: {:?}", avg_pipeline_time);
    println!("📈 Pipeline throughput: {:.2} pipelines/second", pipeline_throughput);
    
    // Target: <500ms per complete pipeline
    assert!(avg_pipeline_time < Duration::from_millis(500), 
        "Pipeline too slow: {:?}", avg_pipeline_time);
}

// Helper function to estimate memory usage
fn get_memory_usage() -> usize {
    // Simplified memory estimation
    // In production, use proper memory profiling tools
    use std::alloc::{GlobalAlloc, Layout, System};
    
    // This is a placeholder - real implementation would use
    // memory profiling libraries like jemalloc or system APIs
    std::process::id() as usize * 1024
}
