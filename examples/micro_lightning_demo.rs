//! MICRO-LIGHTNING TRADING SYSTEM DEMO
//! 
//! Comprehensive demonstration of the micro-lightning trading system
//! Shows complete workflow from token discovery to exit execution

use anyhow::Result;
use std::time::{SystemTime, Duration};
use tokio::time::sleep;
use tracing::{info, warn, error};

use snipercor::modules::micro_lightning::{
    MicroLightningOrchestrator, MicroLightningStrategy, MicroWallet, WalletType,
    EntryConditions, TokenData, MiningEngine, EmergencyProtocol, TimeProtocol,
    ExitSystem, OperationControl, MetricsCollector, OperationRecord,
    check_entry, get_time_based_recommendation
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting MICRO-LIGHTNING TRADING SYSTEM DEMO");
    info!("💰 Capital: $20 | Target: $20/60min operations");
    
    // Demo 1: Basic Wallet Operations
    demo_wallet_operations().await?;
    
    // Demo 2: Token Entry Evaluation
    demo_token_evaluation().await?;
    
    // Demo 3: Mining Engine Operations
    demo_mining_operations().await?;
    
    // Demo 4: Time Protocol Management
    demo_time_protocols().await?;
    
    // Demo 5: Emergency Response System
    demo_emergency_protocols().await?;
    
    // Demo 6: Complete Trading Workflow
    demo_complete_workflow().await?;
    
    // Demo 7: Performance Metrics
    demo_performance_metrics().await?;
    
    info!("✅ MICRO-LIGHTNING DEMO COMPLETED SUCCESSFULLY");
    Ok(())
}

/// Demo 1: Basic wallet operations and fund management
async fn demo_wallet_operations() -> Result<()> {
    info!("\n🏦 === DEMO 1: WALLET OPERATIONS ===");
    
    let mut wallet = MicroWallet::new();
    
    info!("💰 Initial wallet allocation:");
    info!("  Lightning: ${:.2}", wallet.lightning);
    info!("  Emergency Gas: ${:.2}", wallet.emergency_gas);
    info!("  Reentry: ${:.2}", wallet.reentry);
    info!("  Psychology: ${:.2}", wallet.psychology);
    info!("  Tactical Exit: ${:.2}", wallet.tactical_exit);
    
    // Demonstrate fund allocation
    let position_size = wallet.get_lightning_position_size(0.8);
    info!("⚡ Lightning position size (80%): ${:.2}", position_size);
    
    let allocated = wallet.allocate_funds(&WalletType::Lightning, position_size)?;
    info!("💸 Allocated ${:.2} from lightning wallet", allocated);
    
    // Demonstrate psychology tax
    let profit = 10.0;
    let after_tax = wallet.apply_psychology_tax(profit);
    info!("🧠 Applied psychology tax: ${:.2} profit → ${:.2} after tax", profit, after_tax);
    
    // Show utilization summary
    let utilization = wallet.get_utilization_summary();
    info!("📊 Wallet utilization: {:.1}%", utilization.utilization_rate * 100.0);
    
    Ok(())
}

/// Demo 2: Token entry evaluation and filtering
async fn demo_token_evaluation() -> Result<()> {
    info!("\n🔍 === DEMO 2: TOKEN EVALUATION ===");
    
    // Create sample tokens with different characteristics
    let tokens = vec![
        create_sample_token("GOOD", 8, 5000.0, 150, false, 0.6),   // Good token
        create_sample_token("OLD", 25, 4000.0, 100, false, 0.5),   // Too old
        create_sample_token("SCAM", 5, 3000.0, 80, true, 0.3),     // Honeypot
        create_sample_token("LOWLIQ", 10, 1500.0, 120, false, 0.4), // Low liquidity
        create_sample_token("PERFECT", 12, 7500.0, 200, false, 0.8), // Perfect token
    ];
    
    let conditions = EntryConditions::default();
    
    for mut token in tokens {
        token.calculate_risk_score();
        let social_mentions = if token.symbol == "PERFECT" { 60 } else { 35 };
        
        let passed = check_entry(&token, social_mentions);
        let quality_score = token.get_quality_score();
        
        info!("🪙 Token: {} | Age: {}min | Liquidity: ${:.0} | Quality: {:.2} | Risk: {:.2} | Result: {}",
              token.symbol, token.age_minutes, token.liquidity, 
              quality_score, token.risk_score,
              if passed { "✅ PASS" } else { "❌ FAIL" });
    }
    
    Ok(())
}

/// Demo 3: Mining engine operations and strategy execution
async fn demo_mining_operations() -> Result<()> {
    info!("\n⛏️ === DEMO 3: MINING OPERATIONS ===");
    
    let mut engine = MiningEngine::new();
    let token = create_sample_token("MINE", 10, 6000.0, 180, false, 0.7);
    
    // Execute mining operation
    let execution = engine.execute(&token);
    
    info!("🎯 Mining execution for {}:", token.symbol);
    info!("  Initial entry: ${:.2} on {:?}", execution.initial_entry.amount, execution.initial_entry.dex);
    info!("  Reentry enabled: {} (threshold: {:.1}%)", 
          execution.reentry_conditions.enabled,
          (execution.reentry_conditions.price_threshold - 1.0) * 100.0);
    info!("  DLMM allocation: ${:.2}", execution.dlmm_position.allocation);
    info!("  Take-profit levels: {}", execution.exit_strategy.take_profit_levels.len());
    
    // Simulate price movements and reentry checks
    let price_scenarios = vec![1.05, 1.12, 1.18, 1.25]; // 5%, 12%, 18%, 25% gains
    
    for (i, price_multiplier) in price_scenarios.iter().enumerate() {
        let current_price = token.entry_price * price_multiplier;
        let should_reenter = engine.should_reenter(&token.address, current_price, token.entry_price);
        
        info!("📈 Price scenario {}: {:.1}% gain → Reentry: {}", 
              i + 1, (price_multiplier - 1.0) * 100.0,
              if should_reenter { "✅ YES" } else { "❌ NO" });
    }
    
    // Update performance metrics
    engine.update_metrics(8.5, 22.0, true);  // Successful trade
    engine.update_metrics(-1.2, 18.0, false); // Failed trade
    
    let summary = engine.get_performance_summary();
    info!("📊 Performance: {} ops, {:.1}% win rate, ${:.2} net profit",
          summary.total_operations, summary.win_rate * 100.0, summary.net_profit);
    
    Ok(())
}

/// Demo 4: Time protocol management and exit timing
async fn demo_time_protocols() -> Result<()> {
    info!("\n⏰ === DEMO 4: TIME PROTOCOLS ===");
    
    let mut protocol = TimeProtocol::new();
    
    // Simulate different time windows
    let time_scenarios = vec![
        (5.0, "Early Golden Window"),
        (12.0, "Late Golden Window"),
        (20.0, "Early Decay Window"),
        (35.0, "Mid Decay Window"),
        (48.0, "Late Decay Window"),
        (52.0, "Emergency Buffer"),
        (56.0, "Hard Expiry"),
    ];
    
    for (minutes, description) in time_scenarios {
        // Simulate elapsed time (in real implementation, this would be actual time)
        let window = protocol.get_current_window();
        let recommendation = get_time_based_recommendation(&mut protocol);
        
        info!("🕐 {} ({:.0}min): Window: {:?} | Urgency: {:?}",
              description, minutes, window, recommendation.urgency);
    }
    
    // Show timing summary
    let summary = protocol.get_timing_summary();
    info!("📋 Timing Summary:");
    info!("  Elapsed: {:.1}min | Remaining: {:.1}min", 
          summary.elapsed_minutes, summary.remaining_minutes);
    info!("  Emergency buffer: {} | Force close: {}", 
          summary.is_emergency_buffer, summary.should_force_close);
    
    Ok(())
}

/// Demo 5: Emergency response system
async fn demo_emergency_protocols() -> Result<()> {
    info!("\n🚨 === DEMO 5: EMERGENCY PROTOCOLS ===");
    
    let protocol = EmergencyProtocol::new();
    
    // Simulate different emergency scenarios
    let emergencies = vec![
        ("Creator Sell", super::EmergencyTrigger::CreatorSellDetected {
            wallet_address: "creator123".to_string(),
            sell_amount: 5000.0,
            percentage_of_supply: 0.08,
        }),
        ("Liquidity Drop", super::EmergencyTrigger::LiquidityDrop {
            previous_liquidity: 10000.0,
            current_liquidity: 6500.0,
            drop_percentage: 0.35,
        }),
        ("Honeypot Detected", super::EmergencyTrigger::HoneypotDetected {
            detection_method: "transaction_analysis".to_string(),
            confidence: 0.92,
        }),
        ("Time Exceeded", super::EmergencyTrigger::TimeExceeded {
            max_time_minutes: 55,
            actual_time_minutes: 58,
        }),
    ];
    
    for (name, trigger) in emergencies {
        let position = super::Position {
            token: "EMERGENCY_TOKEN".to_string(),
            amount: 100.0,
            entry_price: 0.001,
            current_value: 85.0,
        };
        
        let emergency_exit = super::panic_exit(trigger, &position);
        
        info!("🚨 Emergency: {} → {} actions, max {}s execution",
              name, emergency_exit.actions.len(), emergency_exit.max_execution_time_seconds);
        
        // Show action types
        for (i, action) in emergency_exit.actions.iter().enumerate() {
            let action_type = match action {
                super::Action::CancelAllOrders => "Cancel Orders",
                super::Action::MarketSell { slippage, .. } => &format!("Market Sell ({:.1}% slippage)", slippage),
                super::Action::Transfer { .. } => "Transfer Funds",
                super::Action::FlagToken { .. } => "Flag Token",
                super::Action::NotifyOperator { .. } => "Notify Operator",
                super::Action::ActivateCircuitBreaker { .. } => "Circuit Breaker",
                super::Action::EmergencyWithdraw { .. } => "Emergency Withdraw",
            };
            info!("  {}. {}", i + 1, action_type);
        }
    }
    
    Ok(())
}

/// Demo 6: Complete trading workflow simulation
async fn demo_complete_workflow() -> Result<()> {
    info!("\n🔄 === DEMO 6: COMPLETE WORKFLOW ===");
    
    let mut orchestrator = MicroLightningOrchestrator::new();
    
    // Start the system
    orchestrator.start().await?;
    info!("🚀 Micro-lightning system started");
    
    // Show initial status
    let status = orchestrator.get_status();
    info!("📊 Status: {} | Remaining ops: {} | MEV warning: {}",
          if status.module_active { "ACTIVE" } else { "INACTIVE" },
          status.remaining_ops, status.mev_warning);
    
    // Simulate token discovery and evaluation
    info!("🔍 Scanning for token candidates...");
    sleep(Duration::from_millis(100)).await;
    
    let candidate = create_sample_token("WORKFLOW", 8, 5500.0, 175, false, 0.75);
    info!("🎯 Found candidate: {} (Age: {}min, Liquidity: ${:.0})",
          candidate.symbol, candidate.age_minutes, candidate.liquidity);
    
    // Simulate entry decision
    if check_entry(&candidate, 45) {
        info!("✅ Entry conditions met - executing trade");
        
        // Simulate trading phases
        let phases = vec![
            ("Entry Execution", 2000),
            ("Position Monitoring", 5000),
            ("Reentry Check", 3000),
            ("Exit Evaluation", 2000),
            ("Position Close", 1500),
        ];
        
        for (phase, duration_ms) in phases {
            info!("⚡ {}", phase);
            sleep(Duration::from_millis(duration_ms)).await;
        }
        
        info!("✅ Trade completed successfully");
    } else {
        warn!("❌ Entry conditions not met - skipping token");
    }
    
    // Stop the system
    orchestrator.stop().await?;
    info!("🛑 Micro-lightning system stopped");
    
    Ok(())
}

/// Demo 7: Performance metrics and reporting
async fn demo_performance_metrics() -> Result<()> {
    info!("\n📊 === DEMO 7: PERFORMANCE METRICS ===");
    
    let mut collector = MetricsCollector::new();
    
    // Simulate a series of trading operations
    let operations = vec![
        ("PROFIT1", 0.001, 0.0012, 5.0, 0.20, 18.0, true, "take_profit"),
        ("LOSS1", 0.002, 0.0018, -2.0, -0.10, 12.0, false, "stop_loss"),
        ("PROFIT2", 0.0015, 0.0019, 8.0, 0.27, 25.0, true, "take_profit"),
        ("PROFIT3", 0.0008, 0.0011, 6.5, 0.38, 22.0, true, "time_exit"),
        ("LOSS2", 0.0025, 0.0021, -3.5, -0.16, 15.0, false, "volatility_exit"),
    ];
    
    for (i, (symbol, entry, exit, pnl, pct, time, success, reason)) in operations.iter().enumerate() {
        let record = OperationRecord {
            operation_id: (i + 1) as u32,
            timestamp: SystemTime::now(),
            token_symbol: symbol.to_string(),
            entry_price: *entry,
            exit_price: *exit,
            profit_loss: *pnl,
            profit_percentage: *pct,
            hold_time_minutes: *time,
            success: *success,
            exit_reason: reason.to_string(),
        };
        
        collector.record_operation(record);
        
        info!("📈 Op {}: {} | P&L: ${:.1} ({:.1}%) | Time: {:.0}min | Result: {}",
              i + 1, symbol, pnl, pct * 100.0, time,
              if *success { "✅ WIN" } else { "❌ LOSS" });
    }
    
    // Show final statistics
    let stats = collector.get_stats();
    info!("\n📊 FINAL STATISTICS:");
    info!("  Total Operations: {}", stats.total_operations);
    info!("  Win Rate: {:.1}%", stats.win_rate * 100.0);
    info!("  Average Hold Time: {:.1} minutes", stats.avg_hold_time_minutes);
    info!("  Net Profit: ${:.2}", stats.net_profit);
    info!("  Sharpe Ratio: {:.2}", stats.sharpe_ratio);
    
    // Show performance windows
    if let Some(window_24h) = collector.get_performance_window("24h") {
        info!("  24h Performance: {} ops, ${:.2} profit", 
              window_24h.operations.len(), window_24h.total_profit);
    }
    
    // Generate performance report
    let report = collector.generate_report();
    info!("📋 Performance report generated with {} total operations", 
          report.total_operations);
    
    Ok(())
}

/// Helper function to create sample tokens for testing
fn create_sample_token(symbol: &str, age: u8, liquidity: f64, holders: usize, honeypot: bool, social: f64) -> TokenData {
    let mut token = TokenData::new(
        format!("token_address_{}", symbol.to_lowercase()),
        symbol.to_string(),
        format!("{} Token", symbol),
    );
    
    token.age_minutes = age;
    token.liquidity = liquidity;
    token.holders = holders;
    token.creator_txn_count = 1;
    token.is_honeypot = honeypot;
    token.entry_price = 0.001;
    token.social_score = social;
    token.market_cap = liquidity * 10.0; // Rough estimate
    token.volume_24h = liquidity * 2.0;  // Rough estimate
    
    token
}
