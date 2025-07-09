//! CHIMERA Client integration example with THE OVERMIND PROTOCOL
//! 
//! This example demonstrates how to integrate CHIMERA Client with
//! THE OVERMIND PROTOCOL for autonomous AI-driven trading decisions.

use chimera_client::{
    ChimeraClient, ChimeraConfig, ChatCompletionRequest, ChatMessage,
    FallbackEngine, MarketCondition, TradingAction, ChimeraError,
};
use std::env;
use std::time::Duration;
use tracing::{info, warn, error};
use serde_json::Value;

/// Simulated market data structure
#[derive(Debug)]
struct MarketData {
    symbol: String,
    price: f64,
    price_change_24h: f64,
    volume: f64,
    rsi: Option<f64>,
    ma_short: Option<f64>,
    ma_long: Option<f64>,
    volatility: f64,
}

/// Trading decision from AI or fallback
#[derive(Debug)]
struct TradingDecision {
    action: TradingAction,
    confidence: f64,
    reasoning: String,
    source: DecisionSource,
}

#[derive(Debug)]
enum DecisionSource {
    AI,
    Fallback,
}

/// THE OVERMIND PROTOCOL AI Trading Engine
struct OvermindAIEngine {
    chimera_client: ChimeraClient,
    fallback_engine: FallbackEngine,
}

impl OvermindAIEngine {
    /// Initialize THE OVERMIND PROTOCOL AI Engine
    pub fn new(api_key: String) -> Result<Self, ChimeraError> {
        info!("🧠 Initializing THE OVERMIND PROTOCOL AI Engine");
        
        let config = ChimeraConfig::new(api_key)
            .with_timeout(Duration::from_secs(30))
            .with_max_retries(3)
            .with_endpoint("https://api.deepseek.com".to_string());
        
        let chimera_client = ChimeraClient::new(config)?;
        let fallback_engine = FallbackEngine::new(); // Conservative by default
        
        info!("✅ CHIMERA Client initialized successfully");
        
        Ok(Self {
            chimera_client,
            fallback_engine,
        })
    }
    
    /// Analyze market and make trading decision
    pub async fn analyze_and_decide(&self, market_data: &MarketData) -> TradingDecision {
        info!("📊 Analyzing market for {}", market_data.symbol);
        
        // Try AI analysis first
        match self.get_ai_decision(market_data).await {
            Ok(decision) => {
                info!("🤖 AI decision: {:?} (confidence: {:.2})", 
                     decision.action, decision.confidence);
                decision
            }
            Err(error) => {
                warn!("🔄 AI analysis failed: {}. Using fallback logic.", error);
                self.get_fallback_decision(market_data)
            }
        }
    }
    
    /// Get AI-powered trading decision
    async fn get_ai_decision(&self, market_data: &MarketData) -> Result<TradingDecision, ChimeraError> {
        let prompt = self.create_market_analysis_prompt(market_data);
        
        let messages = vec![
            ChatMessage::system(
                "You are an expert cryptocurrency trading analyst for THE OVERMIND PROTOCOL. \
                 Analyze market conditions and provide trading recommendations in JSON format. \
                 Always include: action (buy/sell/hold), confidence (0.0-1.0), and reasoning.".to_string()
            ),
            ChatMessage::user(prompt),
        ];
        
        let request = ChatCompletionRequest::new("deepseek-chat".to_string(), messages)
            .with_temperature(0.3)
            .with_max_tokens(500)
            .with_json_output();
        
        let response = self.chimera_client.execute_reasoning_task(request).await?;
        self.parse_ai_response(&response)
    }
    
    /// Create detailed market analysis prompt
    fn create_market_analysis_prompt(&self, market_data: &MarketData) -> String {
        format!(
            "Analyze the following market conditions for {}:\n\
             - Current Price: ${:.4}\n\
             - 24h Change: {:.2}%\n\
             - Volume: ${:.0}\n\
             - RSI: {}\n\
             - Short MA: {}\n\
             - Long MA: {}\n\
             - Volatility: {:.2}%\n\n\
             Provide your analysis in JSON format with:\n\
             {{\n\
               \"action\": \"buy|sell|hold\",\n\
               \"confidence\": 0.85,\n\
               \"reasoning\": \"Detailed explanation of the decision\",\n\
               \"risk_level\": 0.6,\n\
               \"position_size\": 0.1\n\
             }}",
            market_data.symbol,
            market_data.price,
            market_data.price_change_24h,
            market_data.volume,
            market_data.rsi.map_or("N/A".to_string(), |r| format!("{:.1}", r)),
            market_data.ma_short.map_or("N/A".to_string(), |ma| format!("${:.4}", ma)),
            market_data.ma_long.map_or("N/A".to_string(), |ma| format!("${:.4}", ma)),
            market_data.volatility * 100.0
        )
    }
    
    /// Parse AI response into trading decision
    fn parse_ai_response(&self, response: &str) -> Result<TradingDecision, ChimeraError> {
        let parsed: Value = serde_json::from_str(response)
            .map_err(|e| ChimeraError::Serialization(e))?;
        
        let action_str = parsed["action"].as_str()
            .ok_or_else(|| ChimeraError::Critical("Missing action in AI response".to_string()))?;
        
        let action = match action_str.to_lowercase().as_str() {
            "buy" => TradingAction::Buy,
            "sell" => TradingAction::Sell,
            "hold" => TradingAction::Hold,
            _ => TradingAction::Hold,
        };
        
        let confidence = parsed["confidence"].as_f64().unwrap_or(0.5);
        let reasoning = parsed["reasoning"].as_str()
            .unwrap_or("AI analysis completed")
            .to_string();
        
        Ok(TradingDecision {
            action,
            confidence,
            reasoning,
            source: DecisionSource::AI,
        })
    }
    
    /// Get fallback decision using static rules
    fn get_fallback_decision(&self, market_data: &MarketData) -> TradingDecision {
        let market_condition = MarketCondition {
            price: market_data.price,
            price_change_24h: market_data.price_change_24h,
            volume: market_data.volume,
            volatility: market_data.volatility,
            rsi: market_data.rsi,
            ma_short: market_data.ma_short,
            ma_long: market_data.ma_long,
        };
        
        let fallback_decision = self.fallback_engine.make_decision(&market_condition);
        
        TradingDecision {
            action: fallback_decision.action,
            confidence: fallback_decision.confidence,
            reasoning: fallback_decision.reasoning,
            source: DecisionSource::Fallback,
        }
    }
    
    /// Get system statistics
    pub async fn get_stats(&self) -> String {
        let stats = self.chimera_client.stats().await;
        let cb_stats = self.chimera_client.circuit_breaker_status().await;
        
        format!(
            "🔧 THE OVERMIND PROTOCOL Statistics:\n\
             📊 Total Requests: {}\n\
             ✅ Successful: {} ({:.1}%)\n\
             ❌ Failed: {} ({:.1}%)\n\
             🔄 Fallback Used: {} times\n\
             ⚡ Circuit Breaker Trips: {}\n\
             🔁 Total Retries: {}\n\
             🛡️ Circuit Breaker: {:?}",
            stats.total_requests,
            stats.successful_requests,
            if stats.total_requests > 0 { 
                stats.successful_requests as f64 / stats.total_requests as f64 * 100.0 
            } else { 0.0 },
            stats.failed_requests,
            if stats.total_requests > 0 { 
                stats.failed_requests as f64 / stats.total_requests as f64 * 100.0 
            } else { 0.0 },
            stats.fallback_decisions,
            stats.circuit_breaker_trips,
            stats.total_retry_attempts,
            cb_stats.map_or("Disabled".to_string(), |cb| format!("{:?}", cb.state))
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting THE OVERMIND PROTOCOL AI Integration Demo");
    
    // Get API key
    let api_key = env::var("DEEPSEEK_API_KEY")
        .expect("DEEPSEEK_API_KEY environment variable must be set");
    
    // Initialize THE OVERMIND PROTOCOL AI Engine
    let overmind = OvermindAIEngine::new(api_key)?;
    
    // Simulate market data for different scenarios
    let market_scenarios = vec![
        MarketData {
            symbol: "SOL/USDT".to_string(),
            price: 95.50,
            price_change_24h: 3.2,
            volume: 2500000.0,
            rsi: Some(65.0),
            ma_short: Some(94.20),
            ma_long: Some(91.80),
            volatility: 0.35,
        },
        MarketData {
            symbol: "BTC/USDT".to_string(),
            price: 43250.0,
            price_change_24h: -2.1,
            volume: 15000000.0,
            rsi: Some(35.0),
            ma_short: Some(43800.0),
            ma_long: Some(44200.0),
            volatility: 0.28,
        },
        MarketData {
            symbol: "ETH/USDT".to_string(),
            price: 2650.0,
            price_change_24h: 8.5,
            volume: 8500000.0,
            rsi: Some(78.0),
            ma_short: Some(2580.0),
            ma_long: Some(2520.0),
            volatility: 0.65, // High volatility
        },
    ];
    
    // Analyze each market scenario
    for (i, market_data) in market_scenarios.iter().enumerate() {
        info!("\n📈 === SCENARIO {} ===", i + 1);
        info!("Market: {}", market_data.symbol);
        
        let decision = overmind.analyze_and_decide(market_data).await;
        
        info!("🎯 Trading Decision:");
        info!("   Action: {:?}", decision.action);
        info!("   Confidence: {:.2}", decision.confidence);
        info!("   Source: {:?}", decision.source);
        info!("   Reasoning: {}", decision.reasoning);
        
        // Simulate some delay between analyses
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Display final statistics
    info!("\n{}", overmind.get_stats().await);
    
    info!("🏁 THE OVERMIND PROTOCOL AI Integration Demo completed successfully");
    
    Ok(())
}
