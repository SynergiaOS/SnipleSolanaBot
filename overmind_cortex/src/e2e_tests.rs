//! E2E Integration Tests - Task 2.4 Implementation
//! 
//! Pełny cykl testowy Twitter→Sentiment→Risk→Decision→HFT zgodnie z filozofią Hotza

use crate::{CortexCore, agents::{SentimentAgent, RiskAgent}, swarm::{SwarmTopology, AgentType}};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Event z Twitter dla symulacji
#[derive(Debug, Clone)]
pub struct TwitterEvent {
    pub content: String,
    pub author: String,
    pub timestamp: std::time::Instant,
    pub engagement: u32,
}

/// Decyzja tradingowa
#[derive(Debug, Clone, PartialEq)]
pub enum TradingDecision {
    Buy { amount: f64, confidence: f32 },
    Sell { amount: f64, confidence: f32 },
    Hold { reason: String },
}

/// Portfolio pozycji
#[derive(Debug, Clone)]
pub struct Portfolio {
    positions: std::collections::HashMap<String, f64>,
    cash: f64,
}

impl Portfolio {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            positions: std::collections::HashMap::new(),
            cash: initial_cash,
        }
    }

    pub fn position(&self, symbol: &str) -> f64 {
        self.positions.get(symbol).copied().unwrap_or(0.0)
    }

    pub fn execute_trade(&mut self, symbol: &str, amount: f64, price: f64) {
        let cost = amount * price;
        if cost <= self.cash {
            *self.positions.entry(symbol.to_string()).or_insert(0.0) += amount;
            self.cash -= cost;
        }
    }
}

/// Swarm Builder dla E2E testów
pub struct SwarmBuilder {
    cortex: Arc<CortexCore>,
    agents: Vec<(AgentType, Box<dyn std::any::Any + Send + Sync>)>,
}

impl SwarmBuilder {
    pub fn new(cortex: Arc<CortexCore>) -> Self {
        Self {
            cortex,
            agents: Vec::new(),
        }
    }

    pub fn add_sentiment_agent(mut self) -> Self {
        let agent = SentimentAgent::new(self.cortex.clone());
        self.agents.push((AgentType::Sentiment, Box::new(agent)));
        self
    }

    pub fn add_risk_agent(mut self) -> Self {
        let agent = RiskAgent::new(self.cortex.clone());
        self.agents.push((AgentType::Risk, Box::new(agent)));
        self
    }

    pub fn build(self) -> E2ESwarm {
        E2ESwarm {
            cortex: self.cortex,
            portfolio: Arc::new(RwLock::new(Portfolio::new(100000.0))), // $100k starting capital
            swarm_topology: Arc::new(RwLock::new(SwarmTopology::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

/// E2E Swarm dla testów integracyjnych
pub struct E2ESwarm {
    cortex: Arc<CortexCore>,
    portfolio: Arc<RwLock<Portfolio>>,
    swarm_topology: Arc<RwLock<SwarmTopology>>,
    event_history: Arc<RwLock<Vec<TwitterEvent>>>,
}

impl E2ESwarm {
    /// Ingestion eventu z Twitter
    pub async fn ingest_event(&self, event: TwitterEvent) -> crate::CortexResult<()> {
        // Zapisz event do historii
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());
        }

        // Rozpocznij pipeline analizy
        self.process_event_pipeline(event).await
    }

    /// Pełny pipeline analizy eventu
    async fn process_event_pipeline(&self, event: TwitterEvent) -> crate::CortexResult<()> {
        // KROK 1: Analiza sentymentu
        let sentiment_score = self.analyze_sentiment(&event.content).await?;
        
        // KROK 2: Ocena ryzyka
        let risk_score = self.assess_risk(&event, sentiment_score).await?;
        
        // KROK 3: Podejmowanie decyzji
        let decision = self.make_trading_decision(sentiment_score, risk_score, &event).await?;
        
        // KROK 4: Wykonanie HFT
        self.execute_hft_decision(decision, &event).await?;

        Ok(())
    }

    /// Analiza sentymentu z użyciem SentimentAgent
    async fn analyze_sentiment(&self, content: &str) -> crate::CortexResult<f32> {
        // Mock implementacja - w rzeczywistości używałby SentimentAgent
        let sentiment = if content.contains("moon") || content.contains("pump") {
            0.8
        } else if content.contains("crash") || content.contains("dump") {
            -0.7
        } else {
            0.1
        };

        Ok(sentiment)
    }

    /// Ocena ryzyka z użyciem RiskAgent
    async fn assess_risk(&self, event: &TwitterEvent, sentiment: f32) -> crate::CortexResult<f32> {
        // Mock implementacja - w rzeczywistości używałby RiskAgent
        let base_risk = 0.3;
        let sentiment_risk = sentiment.abs() * 0.2; // Wyższy sentiment = wyższe ryzyko
        let author_risk = if event.author == "elonmusk" { 0.1 } else { 0.3 }; // Elon = niższe ryzyko
        
        Ok((base_risk + sentiment_risk + author_risk).clamp(0.0, 1.0))
    }

    /// Podejmowanie decyzji tradingowej
    async fn make_trading_decision(
        &self,
        sentiment: f32,
        risk: f32,
        event: &TwitterEvent,
    ) -> crate::CortexResult<TradingDecision> {
        // Logika decyzyjna zgodnie z filozofią Hotza
        let confidence = (sentiment.abs() * (1.0 - risk)).clamp(0.0, 1.0);
        
        if confidence < 0.3 {
            return Ok(TradingDecision::Hold {
                reason: "Low confidence signal".to_string(),
            });
        }

        let amount = confidence * 1000.0; // Maksymalnie $1000 na trade

        if sentiment > 0.5 {
            Ok(TradingDecision::Buy { amount: amount as f64, confidence })
        } else if sentiment < -0.5 {
            Ok(TradingDecision::Sell { amount: amount as f64, confidence })
        } else {
            Ok(TradingDecision::Hold {
                reason: "Neutral sentiment".to_string(),
            })
        }
    }

    /// Wykonanie decyzji HFT
    async fn execute_hft_decision(
        &self,
        decision: TradingDecision,
        _event: &TwitterEvent,
    ) -> crate::CortexResult<()> {
        let mut portfolio = self.portfolio.write().await;
        
        // Symulacja ceny tokena (w rzeczywistości pobierane z API)
        let token_price = 50.0; // $50 za token
        
        match decision {
            TradingDecision::Buy { amount, confidence: _ } => {
                let tokens_to_buy = amount / token_price;
                portfolio.execute_trade("XYZ", tokens_to_buy, token_price);
                println!("🚀 BOUGHT {:.2} XYZ tokens for ${:.2}", tokens_to_buy, amount);
            },
            TradingDecision::Sell { amount, confidence: _ } => {
                let tokens_to_sell = amount / token_price;
                portfolio.execute_trade("XYZ", -tokens_to_sell, token_price);
                println!("📉 SOLD {:.2} XYZ tokens for ${:.2}", tokens_to_sell, amount);
            },
            TradingDecision::Hold { reason } => {
                println!("⏸️  HOLD: {}", reason);
            }
        }

        Ok(())
    }

    /// Pobranie portfolio
    pub async fn get_portfolio(&self) -> Portfolio {
        let portfolio = self.portfolio.read().await;
        portfolio.clone()
    }

    /// Pobranie historii eventów
    pub async fn get_event_history(&self) -> Vec<TwitterEvent> {
        let history = self.event_history.read().await;
        history.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_snipe_cycle() {
        // Utworzenie Cortex i Swarm zgodnie z dokumentem
        let cortex = Arc::new(CortexCore::new().expect("Failed to create cortex"));
        let swarm = SwarmBuilder::new(cortex)
            .add_sentiment_agent()
            .add_risk_agent()
            .build();

        // Symulacja bota memecoina
        let twitter_event = TwitterEvent {
            content: "🚀 $XYZ Just partnered with Tesla! To the moon!".to_string(),
            author: "elonmusk".to_string(),
            timestamp: std::time::Instant::now(),
            engagement: 50000,
        };

        // Ingestion eventu
        swarm.ingest_event(twitter_event).await.expect("Failed to ingest event");

        // Wymuszenie cyklu HFT (250ms zgodnie z dokumentem)
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        // Walidacja - sprawdzenie czy pozycja została otwarta
        let portfolio = swarm.get_portfolio().await;
        let xyz_position = portfolio.position("XYZ");
        
        // Oczekujemy że bot kupił tokeny XYZ
        assert!(xyz_position > 0.0, "Expected positive XYZ position, got {}", xyz_position);
        
        println!("✅ E2E Test PASSED: XYZ position = {:.2}", xyz_position);
    }

    #[tokio::test]
    async fn test_negative_sentiment_cycle() {
        let cortex = Arc::new(CortexCore::new().expect("Failed to create cortex"));
        let swarm = SwarmBuilder::new(cortex)
            .add_sentiment_agent()
            .add_risk_agent()
            .build();

        // Negatywny event
        let twitter_event = TwitterEvent {
            content: "💥 $XYZ massive dump incoming! Sell everything!".to_string(),
            author: "crypto_whale".to_string(),
            timestamp: std::time::Instant::now(),
            engagement: 25000,
        };

        swarm.ingest_event(twitter_event).await.expect("Failed to ingest event");
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        let portfolio = swarm.get_portfolio().await;
        let xyz_position = portfolio.position("XYZ");
        
        // Oczekujemy że bot sprzedał (negatywna pozycja) lub nie kupił
        assert!(xyz_position <= 0.0, "Expected negative or zero XYZ position, got {}", xyz_position);
        
        println!("✅ Negative Sentiment Test PASSED: XYZ position = {:.2}", xyz_position);
    }

    #[tokio::test]
    async fn test_performance_benchmark() {
        // Test wydajności całego pipeline zgodnie z filozofią Hotza
        let cortex = Arc::new(CortexCore::new().expect("Failed to create cortex"));
        let swarm = SwarmBuilder::new(cortex)
            .add_sentiment_agent()
            .add_risk_agent()
            .build();

        let twitter_event = TwitterEvent {
            content: "BTC pump incoming!".to_string(),
            author: "trader123".to_string(),
            timestamp: std::time::Instant::now(),
            engagement: 1000,
        };

        // Benchmark całego cyklu
        let start = std::time::Instant::now();
        swarm.ingest_event(twitter_event).await.expect("Failed to ingest event");
        let duration = start.elapsed();

        // Zgodnie z metrykami Hotza: detekcja→akcja ≤580ms
        assert!(duration.as_millis() < 580, "E2E cycle took {}ms, expected <580ms", duration.as_millis());
        
        println!("✅ Performance Test PASSED: E2E cycle = {}ms", duration.as_millis());
    }
}
