//! FAZA 11 Swarm Genesis Integration Tests
//! 
//! Comprehensive testing suite for evolutionary cycle tests,
//! mutation validation, performance benchmarks, and security stress tests

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use overmind_protocol::overmind::{
    swarm::{SystemCandidate, AgentMetrics, CandidateStatus},
    leaderboard::{SwarmLeaderboard, PercentileAnalysis},
    evolution::{EvolutionEngine, ConfigMutationPlan, ConfigMutation, MutationType, RiskLevel},
    genetic_modifier::{GeneticConfigModifier, MutationResult},
    mutation_guard::{MutationGuard, ValidationResult, MarketHistory},
};

/// Test configuration template
const TEST_CONFIG_TEMPLATE: &str = r#"
[risk_thresholds]
max_drawdown = 0.10
position_limit = 0.25

[hft_params]
aggression = 1.0
latency_target = 500

[trading_strategy]
strategy_type = "momentum"
confidence_threshold = 0.75
"#;

#[tokio::test]
async fn test_system_candidate_lifecycle() -> Result<()> {
    println!("🧬 Testing SystemCandidate lifecycle");
    
    // Create SystemCandidate
    let mut candidate = SystemCandidate::spawn(TEST_CONFIG_TEMPLATE, "conservative").await?;
    assert_eq!(candidate.status, CandidateStatus::Booting);
    
    // Update status
    candidate.set_status(CandidateStatus::Active);
    assert_eq!(candidate.status, CandidateStatus::Active);
    
    // Update metrics
    candidate.performance_metrics.roi = 0.15;
    candidate.performance_metrics.win_rate = 0.75;
    candidate.performance_metrics.latency_p90 = Duration::from_millis(450);
    candidate.performance_metrics.capital_efficiency = 0.85;
    
    // Calculate Hotz score
    candidate.calculate_hotz_score();
    assert!(candidate.performance_metrics.hotz_score > 50.0);
    
    // Terminate candidate
    candidate.terminate().await?;
    assert_eq!(candidate.status, CandidateStatus::Terminated);
    
    println!("✅ SystemCandidate lifecycle test passed");
    Ok(())
}

#[tokio::test]
async fn test_swarm_leaderboard_operations() -> Result<()> {
    println!("📊 Testing SwarmLeaderboard operations");
    
    let leaderboard = SwarmLeaderboard::new();
    
    // Create test candidates
    let mut candidates = Vec::new();
    for i in 0..5 {
        let mut candidate = SystemCandidate::spawn(TEST_CONFIG_TEMPLATE, "test").await?;
        candidate.performance_metrics.hotz_score = 50.0 + (i as f64 * 10.0);
        candidate.performance_metrics.roi = 0.05 + (i as f64 * 0.02);
        candidate.performance_metrics.win_rate = 0.60 + (i as f64 * 0.05);
        
        leaderboard.add_candidate(candidate.clone()).await?;
        candidates.push(candidate);
    }
    
    // Test snapshot creation
    let snapshot = leaderboard.current_snapshot().await?;
    assert_eq!(snapshot.candidates.len(), 5);
    
    // Verify ranking (should be sorted by Hotz score)
    for i in 0..4 {
        assert!(snapshot.candidates[i].hotz_score >= snapshot.candidates[i + 1].hotz_score);
    }
    
    // Test percentile analysis
    let analysis = leaderboard.percentile_analysis().await?;
    assert!(!analysis.elite_candidates.is_empty());
    assert!(!analysis.underperformers.is_empty());
    assert!(analysis.p90_hotz_score >= analysis.median_hotz_score);
    assert!(analysis.median_hotz_score >= analysis.p10_hotz_score);
    
    println!("✅ SwarmLeaderboard operations test passed");
    Ok(())
}

#[tokio::test]
async fn test_evolution_engine_plan_generation() -> Result<()> {
    println!("🧬 Testing EvolutionEngine plan generation");
    
    let evolution_engine = EvolutionEngine::new().await?;
    let leaderboard = SwarmLeaderboard::new();
    
    // Add test candidates to leaderboard
    for i in 0..3 {
        let mut candidate = SystemCandidate::spawn(TEST_CONFIG_TEMPLATE, "test").await?;
        candidate.performance_metrics.hotz_score = 30.0 + (i as f64 * 20.0);
        leaderboard.add_candidate(candidate).await?;
    }
    
    // Generate evolution plan
    let plan = evolution_engine.generate_swarm_evolution_plan(&leaderboard).await?;
    
    // Validate plan structure
    assert!(!plan.mutations.is_empty());
    assert!(plan.expected_improvement > 0.0);
    assert!(plan.risk_assessment.safety_score > 0.0);
    
    // Test mutation recording
    evolution_engine.record_mutation_result(&plan, true, 0.15).await?;
    
    println!("✅ EvolutionEngine plan generation test passed");
    Ok(())
}

#[tokio::test]
async fn test_genetic_configuration_modifier() -> Result<()> {
    println!("⚙️ Testing GeneticConfigModifier");
    
    let modifier = GeneticConfigModifier::new();
    
    // Create test configuration file
    let test_config_path = std::path::PathBuf::from("/tmp/test_config.toml");
    std::fs::write(&test_config_path, TEST_CONFIG_TEMPLATE)?;
    
    // Create test mutation plan
    let plan = ConfigMutationPlan {
        target_candidate: Uuid::new_v4(),
        mutations: vec![
            ConfigMutation {
                target: "risk_thresholds.max_drawdown".to_string(),
                delta: -0.02, // Reduce by 2%
                mutation_type: MutationType::Decrease,
                justification: "Conservative risk reduction".to_string(),
            },
            ConfigMutation {
                target: "hft_params.aggression".to_string(),
                delta: 0.05, // Increase by 5%
                mutation_type: MutationType::Increase,
                justification: "Performance optimization".to_string(),
            },
        ],
        expected_improvement: 10.0,
        risk_assessment: overmind_protocol::overmind::evolution::RiskAssessment {
            risk_level: RiskLevel::Low,
            max_drawdown_impact: 0.02,
            hotz_compliance: true,
            safety_score: 0.85,
        },
        validation_required: true,
    };
    
    // Apply mutations
    let result = modifier.apply_evolution(&test_config_path, &plan).await?;
    
    // Validate results
    assert!(result.success);
    assert_eq!(result.changes_applied.len(), 2);
    assert!(result.validation_errors.is_empty());
    assert!(result.backup_created);
    assert_ne!(result.config_hash_before, result.config_hash_after);
    
    // Verify configuration was actually modified
    let modified_config = std::fs::read_to_string(&test_config_path)?;
    assert!(modified_config.contains("max_drawdown"));
    assert!(modified_config.contains("aggression"));
    
    // Test rollback
    modifier.rollback_configuration(plan.target_candidate).await?;
    
    // Cleanup
    std::fs::remove_file(&test_config_path).ok();
    
    println!("✅ GeneticConfigModifier test passed");
    Ok(())
}

#[tokio::test]
async fn test_mutation_guard_validation() -> Result<()> {
    println!("🛡️ Testing MutationGuard validation");
    
    let mut guard = MutationGuard::new();
    
    // Create test market history
    let market_history = MarketHistory {
        price_history: Vec::new(),
        volatility_events: Vec::new(),
        black_swan_events: Vec::new(),
    };
    
    // Test safe mutation plan
    let safe_plan = ConfigMutationPlan {
        target_candidate: Uuid::new_v4(),
        mutations: vec![
            ConfigMutation {
                target: "risk_thresholds.max_drawdown".to_string(),
                delta: -0.01, // Small safe change
                mutation_type: MutationType::Decrease,
                justification: "Conservative adjustment".to_string(),
            },
        ],
        expected_improvement: 5.0,
        risk_assessment: overmind_protocol::overmind::evolution::RiskAssessment {
            risk_level: RiskLevel::Low,
            max_drawdown_impact: 0.01,
            hotz_compliance: true,
            safety_score: 0.90,
        },
        validation_required: true,
    };
    
    // Validate safe plan
    let safe_result = guard.validate_plan(&safe_plan, &market_history).await?;
    assert!(safe_result.passed);
    assert!(safe_result.warnings.is_empty());
    assert!(safe_result.hotz_compliance);
    
    // Test risky mutation plan
    let risky_plan = ConfigMutationPlan {
        target_candidate: Uuid::new_v4(),
        mutations: vec![
            ConfigMutation {
                target: "risk_thresholds.max_drawdown".to_string(),
                delta: 0.40, // Dangerous large change
                mutation_type: MutationType::Increase,
                justification: "Aggressive risk increase".to_string(),
            },
        ],
        expected_improvement: 50.0,
        risk_assessment: overmind_protocol::overmind::evolution::RiskAssessment {
            risk_level: RiskLevel::Critical,
            max_drawdown_impact: 0.40,
            hotz_compliance: false,
            safety_score: 0.20,
        },
        validation_required: true,
    };
    
    // Validate risky plan
    let risky_result = guard.validate_plan(&risky_plan, &market_history).await?;
    assert!(!risky_result.passed);
    assert!(!risky_result.warnings.is_empty());
    assert!(!risky_result.hotz_compliance);
    
    // Check validation statistics
    let stats = guard.get_stats();
    assert_eq!(stats.total_validations, 2);
    assert_eq!(stats.passed_validations, 1);
    assert_eq!(stats.failed_validations, 1);
    
    println!("✅ MutationGuard validation test passed");
    Ok(())
}

#[tokio::test]
async fn test_end_to_end_evolutionary_cycle() -> Result<()> {
    println!("🔄 Testing end-to-end evolutionary cycle");
    
    // Initialize all components
    let leaderboard = SwarmLeaderboard::new();
    let evolution_engine = EvolutionEngine::new().await?;
    let modifier = GeneticConfigModifier::new();
    let mut guard = MutationGuard::new();
    
    // Create test candidates with varying performance
    let mut candidates = Vec::new();
    for i in 0..12 {
        let mut candidate = SystemCandidate::spawn(TEST_CONFIG_TEMPLATE, "test").await?;
        
        // Simulate performance distribution
        candidate.performance_metrics.hotz_score = 30.0 + (i as f64 * 5.0) + (rand::random::<f64>() * 10.0);
        candidate.performance_metrics.roi = 0.02 + (i as f64 * 0.01);
        candidate.performance_metrics.win_rate = 0.50 + (i as f64 * 0.03);
        
        leaderboard.add_candidate(candidate.clone()).await?;
        candidates.push(candidate);
    }
    
    println!("📊 Created {} candidates", candidates.len());
    
    // Get initial snapshot
    let initial_snapshot = leaderboard.current_snapshot().await?;
    let initial_avg_score = initial_snapshot.summary_stats.avg_hotz_score;
    println!("📈 Initial average Hotz score: {:.2}", initial_avg_score);
    
    // Perform evolutionary cycle
    println!("🧬 Generating evolution plan...");
    let plan = evolution_engine.generate_swarm_evolution_plan(&leaderboard).await?;
    
    println!("🛡️ Validating mutation plan...");
    let market_history = MarketHistory {
        price_history: Vec::new(),
        volatility_events: Vec::new(),
        black_swan_events: Vec::new(),
    };
    let validation_result = guard.validate_plan(&plan, &market_history).await?;
    
    if validation_result.passed {
        println!("✅ Mutation plan validated successfully");
        
        // Apply mutations (simulate with temporary file)
        let temp_config = std::path::PathBuf::from("/tmp/evolution_test_config.toml");
        std::fs::write(&temp_config, TEST_CONFIG_TEMPLATE)?;
        
        let mutation_result = modifier.apply_evolution(&temp_config, &plan).await?;
        assert!(mutation_result.success);
        
        println!("⚙️ Mutations applied: {} changes", mutation_result.changes_applied.len());
        
        // Record successful mutation
        evolution_engine.record_mutation_result(&plan, true, 0.22).await?;
        
        // Cleanup
        std::fs::remove_file(&temp_config).ok();
        
        println!("✅ End-to-end evolutionary cycle completed successfully");
    } else {
        println!("⚠️ Mutation plan failed validation: {:?}", validation_result.warnings);
        
        // Record failed mutation
        evolution_engine.record_mutation_result(&plan, false, -0.05).await?;
        
        println!("✅ End-to-end evolutionary cycle completed with validation rejection");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_performance_benchmarks() -> Result<()> {
    println!("⚡ Testing performance benchmarks");
    
    let start_time = std::time::Instant::now();
    
    // Benchmark SystemCandidate creation
    let candidate_start = std::time::Instant::now();
    let _candidate = SystemCandidate::spawn(TEST_CONFIG_TEMPLATE, "benchmark").await?;
    let candidate_time = candidate_start.elapsed();
    
    // Benchmark leaderboard operations
    let leaderboard = SwarmLeaderboard::new();
    let leaderboard_start = std::time::Instant::now();
    
    for i in 0..100 {
        let mut candidate = SystemCandidate::spawn(TEST_CONFIG_TEMPLATE, "bench").await?;
        candidate.performance_metrics.hotz_score = 50.0 + (i as f64);
        leaderboard.add_candidate(candidate).await?;
    }
    
    let _snapshot = leaderboard.current_snapshot().await?;
    let leaderboard_time = leaderboard_start.elapsed();
    
    // Benchmark evolution plan generation
    let evolution_engine = EvolutionEngine::new().await?;
    let evolution_start = std::time::Instant::now();
    let _plan = evolution_engine.generate_swarm_evolution_plan(&leaderboard).await?;
    let evolution_time = evolution_start.elapsed();
    
    let total_time = start_time.elapsed();
    
    // Performance assertions (Hotz philosophy: sub-millisecond where possible)
    assert!(candidate_time < Duration::from_millis(100), "SystemCandidate creation too slow: {:?}", candidate_time);
    assert!(leaderboard_time < Duration::from_secs(5), "Leaderboard operations too slow: {:?}", leaderboard_time);
    assert!(evolution_time < Duration::from_secs(2), "Evolution plan generation too slow: {:?}", evolution_time);
    
    println!("⚡ Performance benchmarks:");
    println!("  - SystemCandidate creation: {:?}", candidate_time);
    println!("  - Leaderboard (100 candidates): {:?}", leaderboard_time);
    println!("  - Evolution plan generation: {:?}", evolution_time);
    println!("  - Total benchmark time: {:?}", total_time);
    println!("✅ All performance benchmarks passed");
    
    Ok(())
}
