strategy ChimeraMomentumV1:
  metadata:
    name: "Chimera Momentum Strategy V1"
    version: "1.0.0"
    author: "THE OVERMIND PROTOCOL"
    description: "AI-enhanced momentum strategy with multi-timeframe analysis"
    risk_level: 3
    expected_return: 0.22
    max_drawdown: 0.06
    
  risk_model:
    max_drawdown: 6%
    daily_loss_limit: 1.5%
    position_size: 12%
    stop_loss: 2%
    take_profit: 6%
    max_positions: 4
    correlation_limit: 0.5
    
  market_conditions:
    preferred_volatility: [0.01, 0.05]
    min_volume: 1000000
    min_liquidity_score: 0.7
    max_spread: 0.002
    
  entry_logic:
    - trigger: "momentum_signal > 0.8 AND volume_confirmation AND trend_strength > 0.7"
      action: market_buy(size=position_size)
      priority: 1
      enabled: true
      confidence_threshold: 0.8
      
  exit_logic:
    - trigger: "profit > 6% OR loss > 2% OR momentum_signal < 0.3"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      
  ai_models:
    - name: MomentumNet
      version: 2.1
      purpose: "Multi-timeframe momentum detection"
      input_features: ["price", "volume", "volatility"]
      output: "momentum_signal"
