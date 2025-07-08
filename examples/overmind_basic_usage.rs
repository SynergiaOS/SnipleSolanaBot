//! THE OVERMIND PROTOCOL v4.1 "MONOLITH" - Basic Usage Example
//! 
//! This example demonstrates how to initialize and use THE OVERMIND PROTOCOL
//! for autonomous AI trading on Solana blockchain.

use anyhow::Result;
use serde_json::json;
use tokio::time::{sleep, Duration};
use tracing::{info, error};

use overmind_protocol::{
    OvermindProtocol,
    Cortex,
    AgentCandidate,
    KnowledgeGraph,
    DataFlywheel,
    EvolutionEngine,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 THE OVERMIND PROTOCOL v4.1 'MONOLITH' - Basic Usage Example");
    info!("================================================================");

    // Example 1: Initialize THE OVERMIND PROTOCOL
    example_1_initialization().await?;

    // Example 2: Work with individual components
    example_2_components().await?;

    // Example 3: Process trading signals
    example_3_signal_processing().await?;

    // Example 4: Monitor system performance
    example_4_monitoring().await?;

    info!("✅ All examples completed successfully!");
    Ok(())
}

/// Example 1: Basic OVERMIND initialization
async fn example_1_initialization() -> Result<()> {
    info!("📋 Example 1: Basic OVERMIND Initialization");
    info!("-------------------------------------------");

    // Initialize THE OVERMIND PROTOCOL
    let mut overmind = OvermindProtocol::new().await?;
    info!("✅ THE OVERMIND PROTOCOL initialized successfully");

    // The protocol is now ready to process trading signals
    info!("🧠 OVERMIND is ready for autonomous trading");

    Ok(())
}

/// Example 2: Working with individual components
async fn example_2_components() -> Result<()> {
    info!("📋 Example 2: Individual Components");
    info!("----------------------------------");

    // Initialize Cortex (AI Brain)
    let cortex = Cortex::new().await?;
    info!("🧠 Cortex initialized with 5 agent candidates");

    // Initialize Knowledge Graph
    let knowledge_graph = KnowledgeGraph::new().await?;
    info!("📊 Knowledge Graph initialized (in-memory mode)");

    // Initialize Data Flywheel
    let data_flywheel = DataFlywheel::new().await?;
    info!("🔄 Data Flywheel initialized for optimization");

    // Initialize Evolution Engine
    let evolution_engine = EvolutionEngine::new().await?;
    info!("🧬 Evolution Engine initialized for agent improvement");

    // Get statistics from each component
    let kg_stats = knowledge_graph.get_statistics().await?;
    info!("📈 Knowledge Graph stats: {}", kg_stats);

    let df_stats = data_flywheel.get_statistics().await?;
    info!("📈 Data Flywheel stats: {}", df_stats);

    let ee_stats = evolution_engine.get_statistics().await?;
    info!("📈 Evolution Engine stats: {}", ee_stats);

    Ok(())
}

/// Example 3: Processing trading signals
async fn example_3_signal_processing() -> Result<()> {
    info!("📋 Example 3: Signal Processing");
    info!("------------------------------");

    // Initialize Cortex
    let cortex = Cortex::new().await?;

    // Simulate market signals
    let signals = vec![
        json!({
            "type": "new_token",
            "token_address": "So11111111111111111111111111111111111111112",
            "price": 150.50,
            "volume_24h": 1000000,
            "market_cap": 50000000,
            "sentiment": "bullish",
            "timestamp": chrono::Utc::now()
        }),
        json!({
            "type": "price_movement",
            "token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "price_change": 5.2,
            "volume_spike": true,
            "technical_indicators": {
                "rsi": 65,
                "macd": "bullish",
                "moving_average": "above"
            },
            "timestamp": chrono::Utc::now()
        }),
        json!({
            "type": "arbitrage_opportunity",
            "token_address": "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
            "dex_a_price": 100.0,
            "dex_b_price": 102.5,
            "profit_potential": 2.5,
            "liquidity_available": 50000,
            "timestamp": chrono::Utc::now()
        })
    ];

    info!("🔄 Processing {} market signals...", signals.len());

    for (i, signal) in signals.iter().enumerate() {
        info!("📡 Processing signal {}: {}", i + 1, signal.get("type").unwrap());
        
        // Process signal through the cortex
        match cortex.process_signal(signal.clone()).await? {
            Some(decision) => {
                info!("🎯 Decision made: {}", decision);
                
                // Extract decision details
                if let Some(action) = decision.get("action") {
                    if let Some(confidence) = decision.get("confidence") {
                        info!("   Action: {}, Confidence: {}", action, confidence);
                    }
                }
            }
            None => {
                info!("⏸️  No action taken for this signal");
            }
        }

        // Small delay between signals
        sleep(Duration::from_millis(100)).await;
    }

    info!("✅ All signals processed successfully");
    Ok(())
}

/// Example 4: System monitoring
async fn example_4_monitoring() -> Result<()> {
    info!("📋 Example 4: System Monitoring");
    info!("------------------------------");

    // Initialize components
    let cortex = Cortex::new().await?;
    let knowledge_graph = KnowledgeGraph::new().await?;
    let data_flywheel = DataFlywheel::new().await?;

    // Simulate some activity
    info!("🔄 Simulating system activity...");

    // Add some entities to knowledge graph
    use overmind_protocol::overmind::knowledge_graph::{Entity, Relation};
    use std::collections::HashMap;
    use uuid::Uuid;

    let token_entity = Entity {
        id: Uuid::new_v4(),
        name: "SOL".to_string(),
        entity_type: "token".to_string(),
        properties: {
            let mut props = HashMap::new();
            props.insert("symbol".to_string(), json!("SOL"));
            props.insert("price".to_string(), json!(150.50));
            props
        },
        observations: vec!["High trading volume".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        version: 1,
    };

    knowledge_graph.upsert_entity(token_entity).await?;

    // Record some metrics
    use overmind_protocol::overmind::optimization::{ModelMetrics, TrainingDataPoint};

    let metrics = ModelMetrics {
        model_id: "conservative_agent".to_string(),
        accuracy: 0.85,
        latency_ms: 45.0,
        throughput_rps: 25.0,
        cost_per_request: 0.001,
        memory_usage_mb: 128.0,
        timestamp: chrono::Utc::now(),
    };

    data_flywheel.record_metrics(metrics).await?;

    let training_data = TrainingDataPoint {
        input: "Market signal: SOL price increase".to_string(),
        expected_output: "BUY".to_string(),
        actual_output: Some("BUY".to_string()),
        quality_score: Some(0.9),
        timestamp: chrono::Utc::now(),
    };

    data_flywheel.record_training_data(training_data).await?;

    // Get updated statistics
    info!("📊 Collecting system statistics...");

    let kg_stats = knowledge_graph.get_statistics().await?;
    info!("📈 Knowledge Graph: {} entities, {} relations", 
          kg_stats["entities_count"], kg_stats["relations_count"]);

    let df_stats = data_flywheel.get_statistics().await?;
    info!("📈 Data Flywheel: {} training points, {} metrics", 
          df_stats["training_data_points"], df_stats["metrics_history_points"]);

    // Check best performing agent
    if let Some(best_agent) = cortex.get_best_candidate().await {
        info!("🏆 Best performing agent: {}", best_agent);
    } else {
        info!("📊 No performance data available yet");
    }

    info!("✅ Monitoring example completed");
    Ok(())
}

/// Example 5: Advanced usage with evolution
#[allow(dead_code)]
async fn example_5_evolution() -> Result<()> {
    info!("📋 Example 5: Agent Evolution");
    info!("----------------------------");

    let evolution_engine = EvolutionEngine::new().await?;
    let agent_id = uuid::Uuid::new_v4();

    // Simulate evolution cycle
    info!("🧬 Starting evolution cycle for agent: {}", agent_id);

    match evolution_engine.evolve_candidate(agent_id).await {
        Ok(result) => {
            info!("✅ Evolution completed successfully");
            info!("   Strategy used: {:?}", result.strategy_used);
            info!("   Changes made: {:?}", result.changes_made);
            info!("   Expected improvement: {:.2}%", result.expected_improvement);
            info!("   Confidence: {:.2}", result.confidence);
        }
        Err(e) => {
            error!("❌ Evolution failed: {}", e);
        }
    }

    Ok(())
}

/// Helper function to demonstrate error handling
#[allow(dead_code)]
async fn example_error_handling() -> Result<()> {
    info!("📋 Example: Error Handling");
    info!("-------------------------");

    // Demonstrate graceful error handling
    match OvermindProtocol::new().await {
        Ok(_) => info!("✅ OVERMIND initialized successfully"),
        Err(e) => {
            error!("❌ Failed to initialize OVERMIND: {}", e);
            info!("🔄 Falling back to safe mode...");
            // Implement fallback logic here
        }
    }

    Ok(())
}
