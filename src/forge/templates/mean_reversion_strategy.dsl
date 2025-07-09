strategy MeanReversionStrategy:
  risk_model:
    max_drawdown: 3%
    daily_loss_limit: 1.5%
    position_size: 8%
    stop_loss: 1.5%
    take_profit: 3%
    max_positions: 5
    
  entry_logic:
    - trigger: "rsi < 30 AND bollinger_lower_touch AND volume > avg_volume*1.2"
      action: limit_buy(size=position_size, offset=0.05%)
      priority: 1
      enabled: true
      
    - trigger: "price_deviation > 2_std AND support_level_near"
      action: market_buy(size=position_size*0.6)
      priority: 2
      enabled: true
      
  exit_logic:
    - trigger: "rsi > 70 OR profit > 3%"
      action: limit_sell(size=100%, offset=0.1%)
      priority: 1
      enabled: true
      
    - trigger: "bollinger_upper_touch AND profit > 1%"
      action: market_sell(size=75%)
      priority: 2
      enabled: true
      
    - trigger: "loss > 1.5% OR holding_time > 600s"
      action: market_sell(size=100%)
      priority: 3
      enabled: true
      
  ai_models:
    - name: MeanReversionNet
      version: 1.8
      purpose: "Mean reversion signal detection"
      parameters:
        lookback_period: 120
        deviation_threshold: 2.0
        reversion_confidence: 0.75
        
    - name: SupportResistanceAI
      version: 2.2
      purpose: "Dynamic support/resistance levels"
      parameters:
        level_strength_min: 0.6
        touch_tolerance: 0.1
        
    - name: VolatilityPredictor
      version: 1.3
      purpose: "Short-term volatility forecasting"
      parameters:
        prediction_horizon: 300
        confidence_threshold: 0.7
