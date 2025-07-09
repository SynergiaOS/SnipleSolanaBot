// OPERACJA "FORGE" - SentimentAgent DSL Conversion
// 
// Przepisanie SentimentAgent logic w TensorZero DSL
// Proof of Concept dla dynamicznego ładowania strategii

strategy SentimentAgentV1:
  metadata:
    name: "Sentiment Analysis Agent V1"
    version: "1.0.0"
    author: "THE OVERMIND PROTOCOL"
    description: "AI-powered sentiment analysis for trading decisions"
    risk_level: 2
    expected_return: 0.12  // 12% annual return
    max_drawdown: 0.06     // 6% maximum drawdown
    agent_type: "sentiment"
    
  risk_model:
    max_drawdown: 6%
    daily_loss_limit: 1.5%
    position_size: 8%
    stop_loss: 1.8%
    take_profit: 3.5%
    max_positions: 5
    correlation_limit: 0.6
    sentiment_confidence_threshold: 0.7
    
  market_conditions:
    preferred_volatility: [0.005, 0.03]  // 0.5-3% volatility
    min_volume: 500000                   // Minimum 500K volume
    min_liquidity_score: 0.5
    max_spread: 0.003                    // 0.3% max spread
    sentiment_data_freshness: 300        // 5 minutes max age
    
  entry_logic:
    // Primary sentiment signal
    - trigger: "sentiment_score > 0.8 AND sentiment_confidence > 0.7 AND volume > avg_volume_10 * 1.2"
      action: market_buy(size=position_size)
      priority: 1
      enabled: true
      confidence_threshold: 0.8
      description: "Strong positive sentiment with volume confirmation"
      
    // News-driven sentiment
    - trigger: "news_sentiment > 0.75 AND social_sentiment > 0.7 AND price_momentum > 0.3"
      action: limit_buy(size=position_size*0.7, offset=0.05%)
      priority: 2
      enabled: true
      confidence_threshold: 0.75
      description: "Positive news sentiment with social confirmation"
      
    // Contrarian sentiment play
    - trigger: "sentiment_score < -0.6 AND oversold_indicator AND support_level_near"
      action: limit_buy(size=position_size*0.5, offset=0.1%)
      priority: 3
      enabled: true
      confidence_threshold: 0.6
      description: "Contrarian play on oversold sentiment"
      
    // Sentiment momentum
    - trigger: "sentiment_momentum > 0.5 AND sentiment_acceleration > 0.3 AND rsi < 65"
      action: market_buy(size=position_size*0.6)
      priority: 4
      enabled: true
      confidence_threshold: 0.65
      description: "Sentiment momentum acceleration"
      
  exit_logic:
    // Sentiment reversal
    - trigger: "sentiment_score < 0.2 OR sentiment_confidence < 0.5"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      description: "Sentiment reversal or confidence drop"
      
    // Profit taking on sentiment peak
    - trigger: "profit > 3.5% AND sentiment_score > 0.9"
      action: limit_sell(size=75%, offset=0.1%)
      priority: 2
      enabled: true
      description: "Profit taking at sentiment peak"
      
    // Stop loss
    - trigger: "loss > 1.8% OR sentiment_score < -0.4"
      action: market_sell(size=100%)
      priority: 1
      enabled: true
      description: "Stop loss or negative sentiment"
      
    // Time-based exit
    - trigger: "holding_time > 3600s AND profit > 1%"
      action: market_sell(size=50%)
      priority: 3
      enabled: true
      description: "Time-based partial exit with profit"
      
    // Sentiment momentum weakening
    - trigger: "sentiment_momentum < 0.1 AND holding_time > 1800s"
      action: limit_sell(size=60%, offset=0.05%)
      priority: 4
      enabled: true
      description: "Sentiment momentum weakening"
      
  ai_models:
    // Primary sentiment analyzer
    - name: SentimentNet
      version: 3.2
      purpose: "Multi-source sentiment analysis and scoring"
      input_features: ["news_text", "social_media_posts", "price_action", "volume"]
      output: "sentiment_score"
      parameters:
        lookback_window: 1440  // 24 hours
        confidence_threshold: 0.7
        source_weights:
          news: 0.4
          social: 0.3
          technical: 0.3
        update_frequency: 60   // seconds
        
    // News sentiment processor
    - name: NewsAnalyzer
      version: 2.8
      purpose: "Real-time news sentiment extraction"
      input_features: ["news_headlines", "article_content", "source_credibility"]
      output: "news_sentiment"
      parameters:
        sources: ["reuters", "bloomberg", "coindesk", "twitter"]
        credibility_weights: [0.9, 0.9, 0.8, 0.6]
        sentiment_model: "finbert"
        batch_size: 32
        
    // Social sentiment tracker
    - name: SocialSentimentAI
      version: 1.9
      purpose: "Social media sentiment aggregation"
      input_features: ["tweets", "reddit_posts", "telegram_messages"]
      output: "social_sentiment"
      parameters:
        platforms: ["twitter", "reddit", "telegram"]
        influence_weighting: true
        spam_filter_threshold: 0.8
        sentiment_aggregation: "weighted_average"
        
    // Sentiment momentum calculator
    - name: SentimentMomentum
      version: 1.4
      purpose: "Sentiment trend and momentum analysis"
      input_features: ["sentiment_history", "volume_history", "price_history"]
      output: "sentiment_momentum"
      parameters:
        momentum_window: 180   // 3 hours
        acceleration_window: 60 // 1 hour
        trend_strength_threshold: 0.3
        
    // Market context analyzer
    - name: MarketContextAI
      version: 2.1
      purpose: "Market condition assessment for sentiment trading"
      input_features: ["market_cap", "trading_volume", "volatility", "time_of_day"]
      output: "market_context_score"
      parameters:
        market_hours_weight: 1.2
        weekend_discount: 0.8
        volatility_adjustment: true
        
  technical_indicators:
    // Sentiment-specific indicators
    - name: "sentiment_sma"
      type: "simple_moving_average"
      period: 20
      source: "sentiment_score"
      
    - name: "sentiment_ema"
      type: "exponential_moving_average"
      period: 12
      source: "sentiment_score"
      
    // Traditional technical indicators
    - name: "rsi"
      type: "relative_strength_index"
      period: 14
      
    - name: "macd"
      type: "moving_average_convergence_divergence"
      fast_period: 12
      slow_period: 26
      signal_period: 9
      
    - name: "bollinger_bands"
      type: "bollinger_bands"
      period: 20
      std_dev: 2.0
      
    - name: "volume_sma"
      type: "simple_moving_average"
      period: 10
      source: "volume"
      
  data_sources:
    // News sources
    news_feeds:
      - source: "reuters_crypto"
        weight: 0.3
        update_frequency: 300  // 5 minutes
        
      - source: "bloomberg_crypto"
        weight: 0.3
        update_frequency: 300
        
      - source: "coindesk"
        weight: 0.2
        update_frequency: 180  // 3 minutes
        
      - source: "crypto_twitter"
        weight: 0.2
        update_frequency: 60   // 1 minute
        
    // Social media sources
    social_feeds:
      - platform: "twitter"
        keywords: ["$SOL", "solana", "#solana"]
        influencer_list: "crypto_influencers.json"
        weight: 0.4
        
      - platform: "reddit"
        subreddits: ["solana", "cryptocurrency", "cryptomarkets"]
        weight: 0.3
        
      - platform: "telegram"
        channels: ["solana_official", "crypto_signals"]
        weight: 0.3
        
  execution_parameters:
    // Sentiment-specific execution
    sentiment_execution_delay: 30     // seconds to wait for sentiment confirmation
    max_execution_time_ms: 75
    max_slippage: 0.0008              // 0.08%
    min_fill_ratio: 0.92
    
    // Order management
    order_timeout_seconds: 45
    partial_fill_threshold: 0.85
    cancel_on_sentiment_change: true
    sentiment_change_threshold: 0.15   // 15% sentiment change cancels order
    
  performance_targets:
    // Sentiment-specific targets
    sentiment_accuracy_target: 0.72    // 72% sentiment prediction accuracy
    daily_return_target: 0.0003       // 0.03% daily
    monthly_return_target: 0.01       // 1% monthly
    annual_return_target: 0.12        // 12% annual
    
    // Risk targets
    max_daily_drawdown: 0.015          // 1.5%
    max_monthly_drawdown: 0.04         // 4%
    target_sharpe_ratio: 1.8
    target_win_rate: 0.68              // 68%
    
    // Efficiency targets
    avg_trade_duration: 2400           // 40 minutes
    max_trade_duration: 7200           // 2 hours
    min_profit_per_trade: 0.0008       // 0.08%
    
  monitoring:
    // Sentiment monitoring
    track_sentiment_accuracy: true
    track_sentiment_sources: true
    track_news_impact: true
    track_social_influence: true
    
    // Performance monitoring
    track_pnl: true
    track_drawdown: true
    track_win_rate: true
    track_execution_quality: true
    
    // Data quality monitoring
    monitor_data_freshness: true
    monitor_source_reliability: true
    monitor_sentiment_consistency: true
    
  alerts:
    // Sentiment alerts
    - condition: "sentiment_data_age > 600s"
      action: "pause_trading"
      severity: "warning"
      description: "Sentiment data too old"
      
    - condition: "sentiment_confidence < 0.5"
      action: "reduce_position_size"
      severity: "warning"
      description: "Low sentiment confidence"
      
    - condition: "news_sentiment_conflict > 0.5"
      action: "manual_review"
      severity: "warning"
      description: "Conflicting sentiment signals"
      
    // Performance alerts
    - condition: "daily_loss > 1%"
      action: "reduce_exposure"
      severity: "warning"
      
    - condition: "daily_loss > 1.5%"
      action: "halt_trading"
      severity: "critical"
      
    // Data quality alerts
    - condition: "sentiment_source_failure > 2"
      action: "switch_to_backup_sources"
      severity: "warning"
      
    - condition: "all_sentiment_sources_down"
      action: "emergency_exit_positions"
      severity: "critical"
