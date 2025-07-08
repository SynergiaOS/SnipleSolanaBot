/*
THE OVERMIND PROTOCOL v4.1 - Benchmark Suite
Comprehensive performance benchmarking using Criterion

Benchmarks include:
- AI Engine inference performance
- Swarm coordination latency
- Knowledge Graph operations
- Memory allocation patterns
- Concurrent processing throughput
*/

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tokio::runtime::Runtime;
use serde_json::json;
use futures;

use overmind_protocol::overmind::{
    OvermindProtocol,
    ai_engine::{AIEngineConfig, TradingSignal},
    swarm::SwarmOrchestrator,
};

fn bench_overmind_initialization(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("overmind_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let _overmind = black_box(OvermindProtocol::new().await.unwrap());
        });
    });
}

fn bench_ai_engine_inference(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup AI Engine
    let mut overmind = rt.block_on(async {
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
        overmind
    });
    
    let signal = TradingSignal {
        market_data: "SOL price analysis".to_string(),
        technical_indicators: vec![0.7, 0.8, 0.6, 0.9, 0.5, 0.4, 0.8, 0.7, 0.6, 0.9, 
                                  0.3, 0.8, 0.7, 0.5, 0.6, 0.8, 0.4, 0.7, 0.9, 0.6],
        sentiment_score: 0.8,
        volume: 1500000.0,
        price: 150.75,
        timestamp: chrono::Utc::now(),
    };
    
    c.bench_function("ai_engine_inference", |b| {
        b.to_async(&rt).iter(|| async {
            if overmind.has_ai_engine() {
                let _prediction = black_box(
                    overmind.ai_engine().unwrap().predict(&signal).await
                );
            }
        });
    });
}

fn bench_swarm_signal_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let overmind = rt.block_on(async {
        let overmind = OvermindProtocol::new().await.unwrap();
        overmind.initialize_swarm().await.unwrap();
        overmind
    });
    
    let signal = json!({
        "type": "market_update",
        "symbol": "SOL",
        "price": 150.75,
        "volume": 1500000,
        "trend": "bullish"
    });
    
    c.bench_function("swarm_signal_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let _decisions = black_box(
                overmind.swarm().process_market_signal(signal.clone()).await.unwrap()
            );
        });
    });
}

fn bench_swarm_agent_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("swarm_add_agent", |b| {
        b.to_async(&rt).iter(|| async {
            let swarm = SwarmOrchestrator::new().await.unwrap();
            let _agent_id = black_box(swarm.add_agent("conservative").await.unwrap());
        });
    });
}

fn bench_concurrent_signal_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let overmind = rt.block_on(async {
        let overmind = OvermindProtocol::new().await.unwrap();
        overmind.initialize_swarm().await.unwrap();
        overmind
    });
    
    let mut group = c.benchmark_group("concurrent_signal_processing");
    
    for signal_count in [1, 5, 10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("signals", signal_count),
            signal_count,
            |b, &signal_count| {
                b.to_async(&rt).iter(|| async {
                    let mut tasks = Vec::new();
                    
                    for i in 0..signal_count {
                        let swarm = overmind.swarm();
                        let signal = json!({
                            "type": "concurrent_test",
                            "id": i,
                            "symbol": "SOL",
                            "price": 150.0 + i as f64,
                        });
                        
                        let task = async move {
                            swarm.process_market_signal(signal).await
                        };
                        
                        tasks.push(task);
                    }
                    
                    let _results = black_box(futures::future::join_all(tasks).await);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_knowledge_graph_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let overmind = rt.block_on(async {
        OvermindProtocol::new().await.unwrap()
    });
    
    let test_data = json!({
        "type": "trading_pattern",
        "pattern": "bullish_breakout",
        "success_rate": 0.75,
        "conditions": ["high_volume", "price_above_ma", "rsi_oversold"]
    });
    
    c.bench_function("knowledge_graph_store", |b| {
        b.to_async(&rt).iter(|| async {
            let _result = black_box(
                overmind.cortex().knowledge_graph()
                    .store_knowledge("benchmark_pattern", test_data.clone()).await
            );
        });
    });
}

fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("memory_allocation");
    
    for data_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("signal_processing", data_size),
            data_size,
            |b, &data_size| {
                b.to_async(&rt).iter(|| async {
                    let overmind = OvermindProtocol::new().await.unwrap();
                    overmind.initialize_swarm().await.unwrap();
                    
                    for i in 0..*data_size {
                        let signal = json!({
                            "type": "memory_test",
                            "id": i,
                            "data": vec![i; 100],
                        });
                        
                        let _result = black_box(
                            overmind.swarm().process_market_signal(signal).await
                        );
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_evolution_engine_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let overmind = rt.block_on(async {
        OvermindProtocol::new().await.unwrap()
    });
    
    // Get a test agent ID
    let agent_id = rt.block_on(async {
        let agents = overmind.cortex().get_all_agents().await.unwrap();
        agents[0].0
    });
    
    c.bench_function("evolution_engine_analysis", |b| {
        b.to_async(&rt).iter(|| async {
            // This might fail due to LLM API, but we benchmark the attempt
            let _result = black_box(
                overmind.evolution().analyze_candidate_performance(agent_id).await
            );
        });
    });
}

fn bench_full_pipeline_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut overmind = rt.block_on(async {
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
        overmind.initialize_swarm().await.unwrap();
        overmind
    });
    
    c.bench_function("full_pipeline", |b| {
        b.to_async(&rt).iter(|| async {
            // 1. Market signal
            let market_signal = json!({
                "type": "pipeline_benchmark",
                "symbol": "SOL",
                "price": 150.75,
                "volume": 1500000,
                "trend": "bullish"
            });
            
            // 2. Swarm processing
            let _swarm_decisions = black_box(
                overmind.swarm().process_market_signal(market_signal).await.unwrap()
            );
            
            // 3. AI analysis (if available)
            if overmind.has_ai_engine() {
                let ai_signal = TradingSignal {
                    market_data: "Pipeline benchmark".to_string(),
                    technical_indicators: vec![0.5; 20],
                    sentiment_score: 0.7,
                    volume: 1500000.0,
                    price: 150.75,
                    timestamp: chrono::Utc::now(),
                };
                
                let _ai_prediction = black_box(
                    overmind.ai_engine().unwrap().predict(&ai_signal).await
                );
            }
        });
    });
}

criterion_group!(
    benches,
    bench_overmind_initialization,
    bench_ai_engine_inference,
    bench_swarm_signal_processing,
    bench_swarm_agent_management,
    bench_concurrent_signal_processing,
    bench_knowledge_graph_operations,
    bench_memory_allocation_patterns,
    bench_evolution_engine_analysis,
    bench_full_pipeline_performance
);

criterion_main!(benches);
