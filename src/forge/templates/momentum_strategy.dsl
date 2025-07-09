strategy MomentumStrategy:
  risk_model:
    max_drawdown: 5%
    daily_loss_limit: 2%
    position_size: 10%
    stop_loss: 2%
    take_profit: 4%
    max_positions: 3
    
  entry_logic:
    - trigger: "momentum_signal > 0.7 AND volume > 1M AND volatility < 3%"
      action: market_buy(size=position_size)
      priority: 1
      enabled: true
      
    - trigger: "price_breakout AND rsi < 70 AND macd_bullish"
      action: limit_buy(size=position_size*0.8, offset=0.1%)
      priority: 2
      enabled: true
      
  exit_logic:
    - trigger: "profit > 4% OR loss > 2%"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      
    - trigger: "momentum_signal < 0.3 AND holding_time > 300s"
      action: market_sell(size=50%)
      priority: 2
      enabled: true
      
    - trigger: "volume_spike AND profit > 1%"
      action: limit_sell(size=100%, offset=0.2%)
      priority: 3
      enabled: true
      
  ai_models:
    - name: MomentumNet
      version: 2.1
      purpose: "Real-time momentum signal generation"
      parameters:
        lookback_window: 60
        signal_threshold: 0.7
        confidence_filter: 0.8
        
    - name: VolumeAnalyzer
      version: 1.5
      purpose: "Volume pattern recognition"
      parameters:
        volume_ma_period: 20
        spike_threshold: 2.0
        
    - name: RiskAssessor
      version: 3.0
      purpose: "Dynamic risk assessment"
      parameters:
        volatility_window: 30
        correlation_threshold: 0.6
        max_exposure: 0.15
