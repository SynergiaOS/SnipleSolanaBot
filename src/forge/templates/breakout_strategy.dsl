strategy BreakoutStrategy:
  risk_model:
    max_drawdown: 6%
    daily_loss_limit: 3%
    position_size: 12%
    stop_loss: 2.5%
    take_profit: 5%
    max_positions: 4
    
  entry_logic:
    - trigger: "price_breakout_confirmed AND volume > 2*avg_volume"
      action: market_buy(size=position_size)
      priority: 1
      enabled: true
      
    - trigger: "resistance_break AND momentum_acceleration"
      action: limit_buy(size=position_size*0.9, offset=0.2%)
      priority: 2
      enabled: true
      
  exit_logic:
    - trigger: "profit > 5% OR false_breakout_detected"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      
    - trigger: "momentum_weakening AND profit > 2%"
      action: limit_sell(size=75%, offset=0.1%)
      priority: 2
      enabled: true
      
    - trigger: "loss > 2.5% OR support_broken"
      action: market_sell(size=100%)
      priority: 3
      enabled: true
      
  ai_models:
    - name: BreakoutDetector
      version: 2.8
      purpose: "Pattern-based breakout detection"
      parameters:
        pattern_confidence: 0.8
        volume_confirmation: 1.5
        
    - name: FalseBreakoutFilter
      version: 1.4
      purpose: "False breakout prevention"
      parameters:
        confirmation_time: 60
        volume_sustainability: 0.7
