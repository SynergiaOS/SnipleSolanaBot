strategy ChimeraSentimentV1:
  metadata:
    name: "Chimera Sentiment Strategy V1"
    version: "1.0.0"
    author: "THE OVERMIND PROTOCOL"
    description: "Multi-source sentiment analysis with AI aggregation"
    risk_level: 2
    expected_return: 0.18
    max_drawdown: 0.05
    
  ai_models:
    - name: SentimentNet
      version: 3.2
      purpose: "Multi-source sentiment analysis"
      input_features: ["news_sentiment", "social_sentiment", "technical_sentiment"]
      output: "sentiment_score"
