strategy MarketMakingStrategy:
  risk_model:
    max_drawdown: 2%
    daily_loss_limit: 1%
    position_size: 5%
    stop_loss: 1%
    take_profit: 2%
    max_positions: 20
    
  entry_logic:
    - trigger: "bid_ask_spread > 0.1% AND order_book_depth > 10M"
      action: place_bid_ask_orders(spread=0.05%)
      priority: 1
      enabled: true
      
    - trigger: "inventory_imbalance < 20% AND volatility < 2%"
      action: market_making_orders(size=position_size)
      priority: 2
      enabled: true
      
  exit_logic:
    - trigger: "inventory_risk_high OR volatility > 5%"
      action: reduce_inventory(percentage=50%)
      priority: 1
      enabled: true
      
    - trigger: "spread_compression OR profit_target_reached"
      action: close_market_making_positions()
      priority: 2
      enabled: true
      
  ai_models:
    - name: OrderBookAnalyzer
      version: 2.5
      purpose: "Order book depth and flow analysis"
      parameters:
        depth_levels: 10
        flow_prediction_window: 30
        
    - name: InventoryManager
      version: 1.7
      purpose: "Optimal inventory management"
      parameters:
        target_inventory: 0
        max_inventory_deviation: 0.2
