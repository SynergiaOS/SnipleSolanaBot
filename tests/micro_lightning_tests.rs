//! MICRO-LIGHTNING SYSTEM TESTS
//! 
//! Comprehensive test suite for micro-lightning trading operations
//! Tests all components: wallet, entry conditions, mining engine, emergency protocols, etc.

use anyhow::Result;
use std::time::{SystemTime, Duration};
use tokio;

use snipercor::modules::micro_lightning::{
    MicroWallet, WalletType, EntryConditions, TokenData, MiningEngine,
    EmergencyProtocol, EmergencyTrigger, TimeProtocol, ExitSystem,
    OperationControl, OperationError, MetricsCollector, OperationRecord,
    MicroLightningStrategy, MicroLightningOrchestrator,
    check_entry, panic_exit, get_time_based_recommendation
};

/// Test micro wallet functionality
#[tokio::test]
async fn test_micro_wallet_operations() -> Result<()> {
    let mut wallet = MicroWallet::new();
    
    // Test initial allocation
    assert_eq!(wallet.total_capital, 20.0);
    assert_eq!(wallet.lightning, 4.0);
    assert_eq!(wallet.emergency_gas, 3.5);
    assert_eq!(wallet.reentry, 4.5);
    assert_eq!(wallet.psychology, 4.0);
    assert_eq!(wallet.tactical_exit, 4.0);
    
    // Test fund allocation
    let allocated = wallet.allocate_funds(&WalletType::Lightning, 2.0)?;
    assert_eq!(allocated, 2.0);
    assert_eq!(wallet.lightning, 2.0);
    
    // Test psychology tax
    let after_tax = wallet.apply_psychology_tax(10.0);
    assert_eq!(after_tax, 9.0);
    assert_eq!(wallet.psychology, 5.0); // 4.0 + 1.0 tax
    
    // Test position sizing
    let position_size = wallet.get_lightning_position_size(0.8);
    assert_eq!(position_size, 1.6); // 80% of remaining 2.0
    
    // Test wallet integrity
    wallet.validate_integrity()?;
    
    println!("✅ Micro wallet tests passed");
    Ok(())
}

/// Test entry conditions validation
#[tokio::test]
async fn test_entry_conditions() -> Result<()> {
    let mut token = TokenData::new(
        "test_token_123".to_string(),
        "TEST".to_string(),
        "Test Token".to_string(),
    );
    
    // Set up valid token data
    token.age_minutes = 10;
    token.liquidity = 5000.0;
    token.holders = 100;
    token.creator_txn_count = 1;
    token.is_honeypot = false;
    token.social_score = 0.5;
    token.entry_price = 0.001;
    token.calculate_risk_score();
    
    // Test valid entry
    assert!(check_entry(&token, 50));
    
    // Test invalid entry - too old
    token.age_minutes = 20;
    assert!(!check_entry(&token, 50));
    
    // Test invalid entry - honeypot
    token.age_minutes = 10;
    token.is_honeypot = true;
    assert!(!check_entry(&token, 50));
    
    // Test invalid entry - insufficient mentions
    token.is_honeypot = false;
    assert!(!check_entry(&token, 20)); // Only 20 mentions, need 30
    
    // Test battlefield validation
    assert!(token.is_in_battlefield_range());
    token.liquidity = 15000.0;
    assert!(!token.is_in_battlefield_range());
    
    println!("✅ Entry conditions tests passed");
    Ok(())
}

/// Test mining engine operations
#[tokio::test]
async fn test_mining_engine() -> Result<()> {
    let mut engine = MiningEngine::new();
    
    let token = TokenData::new(
        "mining_test_token".to_string(),
        "MINE".to_string(),
        "Mining Test Token".to_string(),
    );
    
    // Execute mining operation
    let execution = engine.execute(&token);
    
    // Verify execution parameters
    assert_eq!(execution.initial_entry.token, token.address);
    assert!(execution.initial_entry.amount > 0.0);
    assert!(execution.reentry_conditions.enabled);
    assert!(execution.dlmm_position.enabled);
    assert_eq!(execution.exit_strategy.take_profit_levels.len(), 4);
    
    // Test reentry conditions
    let should_reenter = engine.should_reenter(&token.address, 1.20, 1.00); // 20% increase
    assert!(should_reenter);
    
    let should_not_reenter = engine.should_reenter(&token.address, 1.10, 1.00); // 10% increase
    assert!(!should_not_reenter);
    
    // Test performance tracking
    engine.update_metrics(5.0, 25.0, true); // Successful trade
    engine.update_metrics(-2.0, 15.0, false); // Failed trade
    
    let summary = engine.get_performance_summary();
    assert_eq!(summary.total_operations, 2);
    assert_eq!(summary.win_rate, 0.5);
    
    println!("✅ Mining engine tests passed");
    Ok(())
}

/// Test emergency protocols
#[tokio::test]
async fn test_emergency_protocols() -> Result<()> {
    let protocol = EmergencyProtocol::new();
    
    // Test different emergency triggers
    let triggers = vec![
        EmergencyTrigger::CreatorSellDetected {
            wallet_address: "creator_wallet".to_string(),
            sell_amount: 1000.0,
            percentage_of_supply: 0.1,
        },
        EmergencyTrigger::LiquidityDrop {
            previous_liquidity: 10000.0,
            current_liquidity: 6000.0,
            drop_percentage: 0.4,
        },
        EmergencyTrigger::TimeExceeded {
            max_time_minutes: 55,
            actual_time_minutes: 60,
        },
        EmergencyTrigger::HoneypotDetected {
            detection_method: "transaction_analysis".to_string(),
            confidence: 0.95,
        },
    ];
    
    for trigger in triggers {
        let position = super::Position {
            token: "test_token".to_string(),
            amount: 100.0,
            entry_price: 0.001,
            current_value: 95.0,
        };
        
        let emergency_exit = panic_exit(trigger.clone(), &position);
        
        // Verify emergency exit structure
        assert!(!emergency_exit.actions.is_empty());
        assert!(!emergency_exit.execution_order.is_empty());
        assert!(emergency_exit.max_execution_time_seconds > 0);
        
        // Check for required actions
        let has_cancel_orders = emergency_exit.actions.iter().any(|a| {
            matches!(a, super::Action::CancelAllOrders)
        });
        assert!(has_cancel_orders);
        
        let has_market_sell = emergency_exit.actions.iter().any(|a| {
            matches!(a, super::Action::MarketSell { .. })
        });
        assert!(has_market_sell);
    }
    
    println!("✅ Emergency protocols tests passed");
    Ok(())
}

/// Test time protocols
#[tokio::test]
async fn test_time_protocols() -> Result<()> {
    let mut protocol = TimeProtocol::new();
    
    // Test initial state (golden window)
    let exit_percentage = protocol.exit_strategy();
    assert_eq!(exit_percentage.as_decimal(), 0.0); // No exit in golden window
    
    // Test time calculations
    assert!(protocol.elapsed_minutes() < 1.0); // Should be very recent
    assert!(protocol.remaining_minutes() > 54.0); // Should be close to 55 minutes
    
    // Test window detection
    let window = protocol.get_current_window();
    assert!(matches!(window, super::TimeWindow::Golden { .. }));
    
    // Test emergency buffer
    assert!(!protocol.is_emergency_buffer_reached());
    assert!(!protocol.should_force_close());
    
    // Test time-based recommendation
    let recommendation = get_time_based_recommendation(&mut protocol);
    assert_eq!(recommendation.urgency, super::ExitUrgency::None);
    
    println!("✅ Time protocols tests passed");
    Ok(())
}

/// Test exit system
#[tokio::test]
async fn test_exit_system() -> Result<()> {
    let mut exit_system = ExitSystem::new();
    
    // Test take-profit levels
    assert_eq!(exit_system.take_profit_radar.levels.len(), 4);
    assert!(!exit_system.take_profit_radar.levels[0].triggered);
    
    // Test exit decision with profitable context
    let mut context = super::TradeContext {
        profit: 0.20, // 20% profit
        volatility_5min: 0.15,
        red_candle_count: 1,
        social_mentions: vec![],
        position: super::Position {
            token: "test_token".to_string(),
            amount: 100.0,
            entry_price: 0.001,
            current_value: 120.0,
        },
    };
    
    let exit_command = exit_system.should_exit(&context);
    assert!(matches!(exit_command, Some(super::ExitCommand::PartialExit(_))));
    
    // Test volatility circuit breaker
    context.volatility_5min = 0.30; // High volatility
    context.red_candle_count = 3;   // Multiple red candles
    
    let exit_command = exit_system.should_exit(&context);
    assert!(matches!(exit_command, Some(super::ExitCommand::FullExit)));
    
    // Test emergency exit on massive loss
    context.profit = -0.30; // 30% loss
    let exit_command = exit_system.should_exit(&context);
    assert!(matches!(exit_command, Some(super::ExitCommand::EmergencyExit)));
    
    println!("✅ Exit system tests passed");
    Ok(())
}

/// Test operation control (5 Commandments)
#[tokio::test]
async fn test_operation_control() -> Result<()> {
    let mut control = OperationControl::new();
    
    // Test initial conditions
    assert!(control.check_conditions().is_ok());
    
    // Test wallet rotation requirement
    control.operations_this_wallet = 3;
    let result = control.check_conditions();
    assert!(matches!(result, Err(OperationError::WalletRotationRequired)));
    
    // Reset for next test
    control.operations_this_wallet = 0;
    
    // Test battlefield validation
    assert!(control.validate_battlefield(5000.0, 100).is_ok());
    assert!(control.validate_battlefield(1000.0, 100).is_err()); // Too low liquidity
    assert!(control.validate_battlefield(5000.0, 10).is_err());  // Too few holders
    
    // Test operation lifecycle
    control.start_operation()?;
    assert_eq!(control.operations_this_wallet, 1);
    assert_eq!(control.total_operations, 1);
    
    control.complete_operation(5.0, true); // Successful operation
    assert_eq!(control.loss_streak, 0);
    
    // Test loss streak
    control.complete_operation(-2.0, false);
    control.complete_operation(-1.5, false);
    control.complete_operation(-3.0, false);
    assert_eq!(control.loss_streak, 3);
    
    // Should trigger cooldown
    let result = control.check_conditions();
    assert!(matches!(result, Err(OperationError::CoolDownPeriod)));
    
    println!("✅ Operation control tests passed");
    Ok(())
}

/// Test metrics collection
#[tokio::test]
async fn test_metrics_collection() -> Result<()> {
    let mut collector = MetricsCollector::new();
    
    // Test initial state
    let stats = collector.get_stats();
    assert_eq!(stats.total_operations, 0);
    assert_eq!(stats.win_rate, 0.0);
    
    // Record some operations
    let operations = vec![
        OperationRecord {
            operation_id: 1,
            timestamp: SystemTime::now(),
            token_symbol: "TEST1".to_string(),
            entry_price: 0.001,
            exit_price: 0.0012,
            profit_loss: 5.0,
            profit_percentage: 0.20,
            hold_time_minutes: 25.0,
            success: true,
            exit_reason: "take_profit".to_string(),
        },
        OperationRecord {
            operation_id: 2,
            timestamp: SystemTime::now(),
            token_symbol: "TEST2".to_string(),
            entry_price: 0.002,
            exit_price: 0.0018,
            profit_loss: -2.0,
            profit_percentage: -0.10,
            hold_time_minutes: 15.0,
            success: false,
            exit_reason: "stop_loss".to_string(),
        },
    ];
    
    for record in operations {
        collector.record_operation(record);
    }
    
    // Verify updated statistics
    let stats = collector.get_stats();
    assert_eq!(stats.total_operations, 2);
    assert_eq!(stats.successful_operations, 1);
    assert_eq!(stats.win_rate, 0.5);
    assert_eq!(stats.avg_hold_time_minutes, 20.0);
    
    // Test performance windows
    let window_24h = collector.get_performance_window("24h");
    assert!(window_24h.is_some());
    
    let recent_ops = collector.get_recent_operations(5);
    assert_eq!(recent_ops.len(), 2);
    
    println!("✅ Metrics collection tests passed");
    Ok(())
}

/// Integration test for complete micro-lightning workflow
#[tokio::test]
async fn test_micro_lightning_integration() -> Result<()> {
    let mut orchestrator = MicroLightningOrchestrator::new();
    
    // Test initialization
    orchestrator.start().await?;
    let status = orchestrator.get_status();
    assert!(status.module_active);
    
    // Test strategy creation
    let strategy = MicroLightningStrategy::new(20.0);
    assert_eq!(strategy.get_capital_allocation(), 20.0);
    assert!(!strategy.is_active()); // Should start inactive
    
    orchestrator.stop().await?;
    let status = orchestrator.get_status();
    assert!(!status.module_active);
    
    println!("✅ Micro-lightning integration tests passed");
    Ok(())
}

/// Performance benchmark test
#[tokio::test]
async fn test_performance_benchmarks() -> Result<()> {
    let start_time = SystemTime::now();
    
    // Test entry condition evaluation speed
    let mut token = TokenData::new(
        "perf_test".to_string(),
        "PERF".to_string(),
        "Performance Test".to_string(),
    );
    token.age_minutes = 10;
    token.liquidity = 5000.0;
    token.holders = 100;
    token.creator_txn_count = 1;
    token.is_honeypot = false;
    token.social_score = 0.5;
    token.entry_price = 0.001;
    
    // Benchmark entry condition checks
    for _ in 0..1000 {
        check_entry(&token, 50);
    }
    
    let entry_check_time = start_time.elapsed().unwrap();
    println!("⚡ 1000 entry checks completed in: {:?}", entry_check_time);
    
    // Should be well under 120ms for micro-lightning requirements
    assert!(entry_check_time < Duration::from_millis(120));
    
    println!("✅ Performance benchmarks passed");
    Ok(())
}
