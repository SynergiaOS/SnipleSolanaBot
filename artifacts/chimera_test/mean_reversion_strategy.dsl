strategy ChimeraMeanReversionV1:
  metadata:
    name: "Chimera Mean Reversion Strategy V1"
    version: "1.0.0"
    author: "THE OVERMIND PROTOCOL"
    description: "Statistical arbitrage with AI-enhanced mean reversion signals"
    risk_level: 2
    expected_return: 0.15
    max_drawdown: 0.04
    
  risk_model:
    max_drawdown: 4%
    daily_loss_limit: 1%
    position_size: 8%
    stop_loss: 1.5%
    take_profit: 3%
    max_positions: 6
    correlation_limit: 0.4
    
  entry_logic:
    - trigger: "reversion_signal > 0.7 AND oversold_condition AND support_level_near"
      action: limit_buy(size=position_size, offset=0.1%)
      priority: 1
      enabled: true
      
  ai_models:
    - name: ReversionNet
      version: 1.8
      purpose: "Mean reversion signal generation"
      input_features: ["price_deviation", "volume", "volatility"]
      output: "reversion_signal"
