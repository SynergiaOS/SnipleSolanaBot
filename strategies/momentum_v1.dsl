// OPERACJA "FORGE" - Momentum Strategy v1
// 
// AI-Generated momentum trading strategy
// Compiled by TensorZero to native Rust code

strategy MomentumStrategyV1:
  metadata:
    name: "Momentum Strategy V1"
    version: "1.0.0"
    author: "THE OVERMIND PROTOCOL"
    description: "High-frequency momentum trading with AI signal generation"
    risk_level: 3
    expected_return: 0.18  // 18% annual return
    max_drawdown: 0.08     // 8% maximum drawdown
    
  risk_model:
    max_drawdown: 8%
    daily_loss_limit: 2%
    position_size: 12%
    stop_loss: 2.5%
    take_profit: 5%
    max_positions: 3
    correlation_limit: 0.7
    
  market_conditions:
    preferred_volatility: [0.01, 0.05]  // 1-5% volatility
    min_volume: 1000000                 // Minimum 1M volume
    min_liquidity_score: 0.6
    max_spread: 0.002                   // 0.2% max spread
    
  entry_logic:
    // Primary momentum signal
    - trigger: "momentum_signal > 0.75 AND volume > avg_volume_20 * 1.5"
      action: market_buy(size=position_size)
      priority: 1
      enabled: true
      confidence_threshold: 0.8
      
    // Breakout confirmation
    - trigger: "price_breakout_confirmed AND rsi < 70 AND macd_bullish"
      action: limit_buy(size=position_size*0.8, offset=0.1%)
      priority: 2
      enabled: true
      confidence_threshold: 0.7
      
    // Volume spike entry
    - trigger: "volume_spike > 2.0 AND momentum_acceleration > 0.5"
      action: market_buy(size=position_size*0.6)
      priority: 3
      enabled: true
      confidence_threshold: 0.6
      
  exit_logic:
    // Profit taking
    - trigger: "profit > 5% OR momentum_signal < 0.2"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      
    // Stop loss
    - trigger: "loss > 2.5% OR support_broken"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      
    // Momentum weakening
    - trigger: "momentum_signal < 0.3 AND holding_time > 300s"
      action: market_sell(size=50%)
      priority: 2
      enabled: true
      
    // Volume drying up
    - trigger: "volume < avg_volume_20 * 0.5 AND profit > 1%"
      action: limit_sell(size=75%, offset=0.2%)
      priority: 3
      enabled: true
      
  ai_models:
    // Primary momentum detector
    - name: MomentumNet
      version: 2.1
      purpose: "Real-time momentum signal generation"
      input_features: ["price", "volume", "volatility", "order_book_imbalance"]
      output: "momentum_signal"
      parameters:
        lookback_window: 60
        signal_threshold: 0.7
        confidence_filter: 0.8
        update_frequency: 100  // milliseconds
        
    // Volume pattern analyzer
    - name: VolumeAnalyzer
      version: 1.5
      purpose: "Volume pattern recognition and spike detection"
      input_features: ["volume", "price", "time_of_day"]
      output: "volume_spike"
      parameters:
        volume_ma_period: 20
        spike_threshold: 2.0
        pattern_memory: 1440  // minutes (24h)
        
    // Risk assessment model
    - name: RiskAssessor
      version: 3.0
      purpose: "Dynamic risk assessment and position sizing"
      input_features: ["volatility", "correlation", "portfolio_exposure"]
      output: "risk_score"
      parameters:
        volatility_window: 30
        correlation_threshold: 0.6
        max_exposure: 0.15
        rebalance_frequency: 300  // seconds
        
    // Market microstructure analyzer
    - name: MicrostructureAI
      version: 1.8
      purpose: "Order book analysis and execution timing"
      input_features: ["bid_ask_spread", "order_book_depth", "trade_flow"]
      output: "execution_quality"
      parameters:
        depth_levels: 10
        flow_prediction_window: 30
        execution_threshold: 0.7
        
  technical_indicators:
    // Moving averages
    - name: "ema_fast"
      type: "exponential_moving_average"
      period: 12
      
    - name: "ema_slow"
      type: "exponential_moving_average"
      period: 26
      
    // Momentum indicators
    - name: "rsi"
      type: "relative_strength_index"
      period: 14
      
    - name: "macd"
      type: "moving_average_convergence_divergence"
      fast_period: 12
      slow_period: 26
      signal_period: 9
      
    // Volume indicators
    - name: "volume_ma"
      type: "simple_moving_average"
      period: 20
      source: "volume"
      
    - name: "vwap"
      type: "volume_weighted_average_price"
      period: 1440  // daily VWAP
      
  execution_parameters:
    // Timing constraints
    max_execution_time_ms: 50
    max_slippage: 0.001  // 0.1%
    min_fill_ratio: 0.95
    
    // Order management
    order_timeout_seconds: 30
    partial_fill_threshold: 0.8
    cancel_on_adverse_move: 0.005  // 0.5%
    
    // Market making parameters
    bid_ask_improvement: 0.0001  // 1 basis point
    inventory_target: 0.0
    max_inventory_deviation: 0.1
    
  performance_targets:
    // Return targets
    daily_return_target: 0.0005  // 0.05% daily
    monthly_return_target: 0.015  // 1.5% monthly
    annual_return_target: 0.18   // 18% annual
    
    // Risk targets
    max_daily_drawdown: 0.02     // 2%
    max_monthly_drawdown: 0.05   // 5%
    target_sharpe_ratio: 2.0
    target_win_rate: 0.65        // 65%
    
    // Efficiency targets
    avg_trade_duration: 180      // 3 minutes
    max_trade_duration: 600      // 10 minutes
    min_profit_per_trade: 0.001  // 0.1%
    
  monitoring:
    // Performance monitoring
    track_pnl: true
    track_drawdown: true
    track_win_rate: true
    track_execution_quality: true
    
    // Risk monitoring
    monitor_correlation: true
    monitor_exposure: true
    monitor_volatility: true
    
    // System monitoring
    monitor_latency: true
    monitor_memory_usage: true
    monitor_cpu_usage: true
    
  alerts:
    // Performance alerts
    - condition: "daily_loss > 1.5%"
      action: "reduce_position_size"
      severity: "warning"
      
    - condition: "daily_loss > 2%"
      action: "halt_trading"
      severity: "critical"
      
    // Risk alerts
    - condition: "correlation > 0.8"
      action: "diversify_positions"
      severity: "warning"
      
    - condition: "volatility > 0.1"
      action: "reduce_exposure"
      severity: "warning"
      
    // System alerts
    - condition: "execution_latency > 100ms"
      action: "switch_to_backup_system"
      severity: "critical"
      
    - condition: "memory_usage > 80%"
      action: "restart_strategy"
      severity: "warning"
