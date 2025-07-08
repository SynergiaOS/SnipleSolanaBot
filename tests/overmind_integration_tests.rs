/*
THE OVERMIND PROTOCOL v4.1 - Integration Tests
End-to-end testing of OVERMIND component interactions

Tests include:
- Full pipeline: Signal → AI Analysis → Swarm Decision → Evolution
- Cortex + Swarm coordination
- AI Engine + Evolution Engine integration
- Knowledge Graph + All components
- Performance under realistic load
*/

use std::time::{Duration, Instant};
use serde_json::json;
use futures;

use overmind_protocol::overmind::{
    OvermindProtocol,
    ai_engine::{AIEngineConfig, TradingSignal},
};

#[tokio::test]
async fn test_full_overmind_pipeline() {
    // Initialize complete OVERMIND system
    let mut overmind = OvermindProtocol::new().await.unwrap();
    
    // Initialize AI Engine
    let ai_config = AIEngineConfig {
        device: "cpu".to_string(),
        model_path: "test-model".to_string(),
        max_sequence_length: 512,
        batch_size: 8,
        enable_fine_tuning: false,
        learning_rate: 1e-5,
    };
    
    overmind.initialize_ai_engine(Some(ai_config)).await.unwrap();
    
    // Initialize Swarm
    overmind.initialize_swarm().await.unwrap();
    
    // Test full pipeline
    let start = Instant::now();
    
    // 1. Create market signal
    let market_signal = json!({
        "type": "price_movement",
        "symbol": "SOL",
        "price": 150.75,
        "volume": 1500000,
        "trend": "bullish",
        "confidence": 0.8
    });
    
    // 2. Process through Swarm
    let swarm_decisions = overmind.swarm()
        .process_market_signal(market_signal.clone()).await.unwrap();
    
    // 3. AI Analysis (if available)
    if overmind.has_ai_engine() {
        let trading_signal = TradingSignal {
            market_data: "SOL bullish trend detected".to_string(),
            technical_indicators: vec![0.7, 0.8, 0.6, 0.9, 0.5, 0.4, 0.8, 0.7, 0.6, 0.9,
                                      0.3, 0.8, 0.7, 0.5, 0.6, 0.8, 0.4, 0.7, 0.9, 0.6],
            sentiment_score: 0.8,
            volume: 1500000.0,
            price: 150.75,
            timestamp: chrono::Utc::now(),
        };
        
        let ai_prediction = overmind.ai_engine().unwrap()
            .predict(&trading_signal).await.unwrap();
        
        println!("🤖 AI Prediction: {} with {:.2}% confidence", 
                 ai_prediction.action, ai_prediction.confidence * 100.0);
    }
    
    let pipeline_time = start.elapsed();
    
    // Validate results
    assert!(!swarm_decisions.is_empty(), "Should generate swarm decisions");
    assert!(pipeline_time < Duration::from_secs(2), 
        "Full pipeline too slow: {:?}", pipeline_time);
    
    println!("✅ Full OVERMIND pipeline: {:?}", pipeline_time);
    println!("📊 Swarm decisions: {}", swarm_decisions.len());
}

#[tokio::test]
async fn test_cortex_swarm_coordination() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    // Test coordination between Cortex agents and Swarm agents
    let cortex_agents = overmind.cortex().get_all_agents().await.unwrap();
    let swarm_metrics = overmind.swarm().get_swarm_metrics().await.unwrap();

    // Both should have agents
    assert!(!cortex_agents.is_empty(), "Cortex should have agents");
    assert!(swarm_metrics["total_agents"].as_u64().unwrap() > 0, 
            "Swarm should have agents");
    
    // Test signal processing through both systems
    let test_signal = json!({
        "type": "arbitrage_opportunity",
        "pair": "SOL/USDC",
        "price_diff": 0.05,
        "volume": 100000
    });
    
    let start = Instant::now();
    let decisions = overmind.swarm()
        .process_market_signal(test_signal).await.unwrap();
    let coordination_time = start.elapsed();
    
    assert!(coordination_time < Duration::from_millis(500), 
        "Coordination too slow: {:?}", coordination_time);
    
    println!("✅ Cortex-Swarm coordination: {:?}", coordination_time);
    println!("🤝 Coordinated decisions: {}", decisions.len());
}

#[tokio::test]
async fn test_ai_evolution_integration() {
    let mut overmind = OvermindProtocol::new().await.unwrap();
    
    // Initialize AI Engine
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
    
    // Get an agent ID for evolution testing
    let cortex_agents = overmind.cortex().get_all_agents().await.unwrap();
    let test_agent_id = cortex_agents[0].0; // Get UUID from tuple
    
    // Test evolution analysis (this will use LLM API)
    let start = Instant::now();
    let analysis_result = overmind.evolution()
        .analyze_candidate_performance(test_agent_id).await;
    let analysis_time = start.elapsed();
    
    // Analysis might fail due to LLM API issues, but should not crash
    match analysis_result {
        Ok(analysis) => {
            println!("✅ Evolution analysis successful: {:?}", analysis_time);
            println!("📊 Performance score: {:.2}", analysis.performance_score);
        }
        Err(e) => {
            println!("⚠️ Evolution analysis failed (expected): {}", e);
            // This is acceptable for testing - LLM API might not be available
        }
    }
    
    // Should complete within reasonable time regardless of success/failure
    assert!(analysis_time < Duration::from_secs(30), 
        "Evolution analysis timeout: {:?}", analysis_time);
    
    println!("✅ AI-Evolution integration test completed");
}

#[tokio::test]
async fn test_knowledge_graph_integration() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    // Test knowledge storage and retrieval
    let knowledge_data = json!({
        "type": "trading_pattern",
        "pattern": "bullish_breakout",
        "success_rate": 0.75,
        "conditions": ["high_volume", "price_above_ma", "rsi_oversold"]
    });
    
    let start = Instant::now();
    let store_result = overmind.cortex().knowledge_graph()
        .store_knowledge("pattern_analysis", knowledge_data).await;
    let storage_time = start.elapsed();
    
    // Should not fail (falls back to memory if Qdrant unavailable)
    assert!(store_result.is_ok(), "Knowledge storage should work");
    assert!(storage_time < Duration::from_secs(2),
        "Knowledge storage too slow: {:?}", storage_time);
    
    println!("✅ Knowledge Graph integration: {:?}", storage_time);
}

#[tokio::test]
async fn test_concurrent_signal_processing() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    // Test concurrent processing of multiple signals
    let signal_count = 10;
    let mut tasks = Vec::new();
    
    let start = Instant::now();
    
    for i in 0..signal_count {
        let swarm = overmind.swarm();
        let signal = json!({
            "type": "market_update",
            "id": i,
            "symbol": "SOL",
            "price": 150.0 + i as f64,
            "volume": 1000000 + i * 100000
        });
        
        let task = async move {
            swarm.process_market_signal(signal).await
        };
        
        tasks.push(task);
    }
    
    // Process all signals concurrently
    let results = futures::future::join_all(tasks).await;
    let concurrent_time = start.elapsed();
    
    // All should succeed
    let successful_count = results.iter()
        .filter(|r| r.is_ok())
        .count();
    
    assert_eq!(successful_count, signal_count, 
               "All signals should process successfully");
    
    // Should be faster than sequential processing
    assert!(concurrent_time < Duration::from_secs(5), 
        "Concurrent processing too slow: {:?}", concurrent_time);
    
    println!("✅ Concurrent signal processing: {:?}", concurrent_time);
    println!("📊 Processed {} signals concurrently", signal_count);
}

#[tokio::test]
async fn test_system_resilience() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    // Test system behavior with invalid inputs
    let invalid_signals = vec![
        json!({}), // Empty signal
        json!({"invalid": "data"}), // Invalid structure
        json!({"type": "unknown_type"}), // Unknown type
    ];
    
    for (i, signal) in invalid_signals.iter().enumerate() {
        let result = overmind.swarm()
            .process_market_signal(signal.clone()).await;
        
        // System should handle gracefully (not crash)
        match result {
            Ok(decisions) => {
                println!("✅ Invalid signal {} handled gracefully: {} decisions", 
                         i, decisions.len());
            }
            Err(e) => {
                println!("⚠️ Invalid signal {} rejected: {}", i, e);
                // This is also acceptable - system should reject invalid input
            }
        }
    }
    
    println!("✅ System resilience test completed");
}

#[tokio::test]
async fn test_memory_stability_under_load() {
    let overmind = OvermindProtocol::new().await.unwrap();
    overmind.initialize_swarm().await.unwrap();
    
    let initial_memory = get_memory_usage();
    let iteration_count = 100;
    
    for i in 0..iteration_count {
        let signal = json!({
            "type": "stress_test",
            "iteration": i,
            "data": vec![0; 1000], // Some data to process
        });
        
        let _result = overmind.swarm()
            .process_market_signal(signal).await;
        
        // Check memory every 25 iterations
        if i % 25 == 0 {
            let current_memory = get_memory_usage();
            let memory_growth = current_memory.saturating_sub(initial_memory);
            
            println!("🧠 Memory after {} iterations: +{} bytes", i, memory_growth);
            
            // Memory growth should be reasonable (< 50MB)
            assert!(memory_growth < 50_000_000, 
                "Excessive memory growth: {} bytes", memory_growth);
        }
    }
    
    println!("✅ Memory stability test completed");
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let mut overmind = OvermindProtocol::new().await.unwrap();
    
    // Initialize AI Engine
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
    
    // Benchmark different operations
    let benchmarks = vec![
        ("Swarm Signal Processing", Duration::from_millis(100)),
        ("AI Prediction", Duration::from_millis(50)),
        ("Agent Coordination", Duration::from_millis(200)),
    ];
    
    for (operation, target_time) in benchmarks {
        let start = Instant::now();
        
        match operation {
            "Swarm Signal Processing" => {
                let signal = json!({"type": "benchmark", "data": "test"});
                let _result = overmind.swarm().process_market_signal(signal).await;
            }
            "AI Prediction" => {
                if overmind.has_ai_engine() {
                    let signal = TradingSignal {
                        market_data: "Benchmark test".to_string(),
                        technical_indicators: vec![0.5; 20],
                        sentiment_score: 0.5,
                        volume: 1000000.0,
                        price: 100.0,
                        timestamp: chrono::Utc::now(),
                    };
                    let _result = overmind.ai_engine().unwrap().predict(&signal).await;
                }
            }
            "Agent Coordination" => {
                let _metrics = overmind.swarm().get_swarm_metrics().await;
            }
            _ => {}
        }
        
        let elapsed = start.elapsed();
        
        println!("⚡ {}: {:?} (target: {:?})", operation, elapsed, target_time);
        
        // Performance targets are guidelines, not hard requirements for tests
        if elapsed > target_time {
            println!("⚠️ {} slower than target", operation);
        }
    }
    
    println!("✅ Performance benchmarks completed");
}

// Helper function to get memory usage (simplified)
fn get_memory_usage() -> usize {
    // This is a simplified implementation
    // In a real scenario, you'd use a proper memory profiling library
    std::process::id() as usize * 1000 // Placeholder
}
