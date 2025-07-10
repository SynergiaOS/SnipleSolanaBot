//! STRATEGIA "SOCIAL FISSION"
//! 
//! Cel: Eksploatacja hype'u społecznościowego
//! Algorytm: Wykrycie trendu na Twitter → Analiza sentymentu → Weryfikacja on-chain
//! Warunki: Min. 3 wzmianki/5s + Sentyment > 85% + Volume > 1000 SOL

use super::*;
use crate::modules::strategy::{TradingSignal, TradeAction, StrategyType};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Parametry dla SOCIAL FISSION
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialFissionParams {
    pub min_mentions_per_5s: u32,      // 3 wzmianki na 5 sekund
    pub sentiment_threshold: f32,       // 85.0%
    pub volume_threshold: f64,          // 1000 SOL
    pub capital_allocation: f32,        // 7% kapitału
    pub hype_decay_factor: f32,         // 0.8 (wykładnicza waga)
}

impl Default for SocialFissionParams {
    fn default() -> Self {
        Self {
            min_mentions_per_5s: 3,
            sentiment_threshold: 85.0,
            volume_threshold: 1000.0,
            capital_allocation: 0.07,
            hype_decay_factor: 0.8,
        }
    }
}

/// Agregator sygnałów społecznościowych
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialAggregator {
    pub token: String,
    pub total_mentions: u32,
    pub avg_sentiment: f32,
    pub sources: HashMap<String, u32>, // source -> count
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Implementacja strategii SOCIAL FISSION
pub struct SocialFissionStrategy {
    params: SocialFissionParams,
    strategy_params: MemcoinStrategyParams,
    is_active: bool,
    capital: f64,
    hype_buffer: Arc<RwLock<VecDeque<SocialSignal>>>,
    social_aggregators: Arc<RwLock<HashMap<String, SocialAggregator>>>,
    metrics: StrategyMetrics,
}

impl SocialFissionStrategy {
    /// Tworzy nową instancję strategii SOCIAL FISSION
    pub fn new(capital: f64) -> Self {
        Self {
            params: SocialFissionParams::default(),
            strategy_params: MemcoinStrategyParams::default(),
            is_active: false,
            capital,
            hype_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
            social_aggregators: Arc::new(RwLock::new(HashMap::new())),
            metrics: StrategyMetrics::default(),
        }
    }

    /// Aktualizacja bufora hype'u
    pub async fn update_hype_buffer(&self, signal: SocialSignal) {
        let mut buffer = self.hype_buffer.write().await;
        
        buffer.push_back(signal);
        
        // Utrzymuj maksymalnie 50 sygnałów (ostatnie 5 minut przy 1 sygnale/6s)
        if buffer.len() > 50 {
            buffer.pop_front();
        }
        
        debug!("Hype buffer updated, size: {}", buffer.len());
    }

    /// Kalkulacja hype score z wykładniczą wagą
    pub async fn calculate_hype_score(&self, token: &str) -> f32 {
        let buffer = self.hype_buffer.read().await;
        
        // Filtruj sygnały dla konkretnego tokena z ostatnich 5 sekund
        let now = chrono::Utc::now();
        let relevant_signals: Vec<_> = buffer.iter()
            .filter(|s| s.token == token)
            .filter(|s| now.signed_duration_since(s.timestamp).num_seconds() <= 5)
            .collect();

        if relevant_signals.is_empty() {
            return 0.0;
        }

        // Wykładnicza waga ostatnich sygnałów
        let hype_score: f32 = relevant_signals.iter().enumerate()
            .map(|(i, s)| s.intensity * self.params.hype_decay_factor.powi(i as i32))
            .sum();

        debug!("Hype score for {}: {} (from {} signals)", token, hype_score, relevant_signals.len());
        hype_score
    }

    /// Analiza sentymentu Chimera Core
    pub async fn analyze_sentiment(&self, token: &str) -> f32 {
        let buffer = self.hype_buffer.read().await;
        
        // Zbierz sygnały dla tokena z ostatnich 30 sekund
        let now = chrono::Utc::now();
        let recent_signals: Vec<_> = buffer.iter()
            .filter(|s| s.token == token)
            .filter(|s| now.signed_duration_since(s.timestamp).num_seconds() <= 30)
            .collect();

        if recent_signals.is_empty() {
            return 0.0;
        }

        // Średnia ważona sentymentu (waga = intensity)
        let total_weight: f32 = recent_signals.iter().map(|s| s.intensity).sum();
        let weighted_sentiment: f32 = recent_signals.iter()
            .map(|s| s.sentiment * s.intensity)
            .sum();

        let avg_sentiment = if total_weight > 0.0 {
            (weighted_sentiment / total_weight + 1.0) * 50.0 // Konwersja z [-1,1] na [0,100]
        } else {
            0.0
        };

        debug!("Sentiment analysis for {}: {}% (from {} signals)", token, avg_sentiment, recent_signals.len());
        avg_sentiment
    }

    /// Sprawdzenie warunków wejścia
    pub async fn check_entry_conditions(&self, token: &str, volume: f64) -> bool {
        // 1. Sprawdź liczbę wzmianek w ostatnich 5 sekundach
        let buffer = self.hype_buffer.read().await;
        let now = chrono::Utc::now();
        let recent_mentions = buffer.iter()
            .filter(|s| s.token == token)
            .filter(|s| now.signed_duration_since(s.timestamp).num_seconds() <= 5)
            .count() as u32;

        if recent_mentions < self.params.min_mentions_per_5s {
            debug!("Not enough mentions for {}: {} < {}", token, recent_mentions, self.params.min_mentions_per_5s);
            return false;
        }

        // 2. Sprawdź sentyment
        drop(buffer); // Zwolnij lock przed kolejnym wywołaniem
        let sentiment = self.analyze_sentiment(token).await;
        if sentiment < self.params.sentiment_threshold {
            debug!("Sentiment too low for {}: {}% < {}%", token, sentiment, self.params.sentiment_threshold);
            return false;
        }

        // 3. Sprawdź volume on-chain
        if volume < self.params.volume_threshold {
            debug!("Volume too low for {}: {} SOL < {} SOL", token, volume, self.params.volume_threshold);
            return false;
        }

        info!("🚀 SOCIAL FISSION conditions met for {}: mentions={}, sentiment={}%, volume={} SOL", 
              token, recent_mentions, sentiment, volume);
        true
    }

    /// Aktualizacja agregatora społecznościowego
    pub async fn update_social_aggregator(&self, signal: &SocialSignal) {
        let mut aggregators = self.social_aggregators.write().await;
        
        let aggregator = aggregators.entry(signal.token.clone()).or_insert_with(|| {
            SocialAggregator {
                token: signal.token.clone(),
                total_mentions: 0,
                avg_sentiment: 0.0,
                sources: HashMap::new(),
                last_update: chrono::Utc::now(),
            }
        });

        // Aktualizuj statystyki
        aggregator.total_mentions += 1;
        aggregator.avg_sentiment = (aggregator.avg_sentiment + signal.sentiment) / 2.0;
        *aggregator.sources.entry(signal.source.clone()).or_insert(0) += 1;
        aggregator.last_update = chrono::Utc::now();

        debug!("Updated social aggregator for {}: {} mentions, {}% sentiment", 
               signal.token, aggregator.total_mentions, aggregator.avg_sentiment);
    }

    /// Generowanie sygnału handlowego
    pub async fn generate_trading_signal(&self, token: &str, volume: f64) -> Option<TradingSignal> {
        if !self.check_entry_conditions(token, volume).await {
            return None;
        }

        let hype_score = self.calculate_hype_score(token).await;
        let sentiment = self.analyze_sentiment(token).await;

        // Kalkulacja rozmiaru pozycji na podstawie hype score
        let base_size = self.capital * self.params.capital_allocation as f64;
        let hype_multiplier = (hype_score / 100.0).min(2.0); // Max 2x boost
        let position_size = (base_size * hype_multiplier as f64).min(self.strategy_params.max_position_size);

        // Kalkulacja confidence na podstawie sentymentu i hype
        let confidence = ((sentiment / 100.0) * 0.7 + (hype_score / 100.0) * 0.3).min(1.0);

        info!("🚀 SOCIAL FISSION signal generated for {}: size={} SOL, confidence={}", 
              token, position_size, confidence);

        Some(TradingSignal {
            signal_id: uuid::Uuid::new_v4().to_string(),
            symbol: token.to_string(),
            action: TradeAction::Buy,
            quantity: position_size,
            target_price: 0.0, // Market order
            price: None, // Market order
            confidence: confidence as f64,
            timestamp: chrono::Utc::now(),
            strategy_type: StrategyType::SocialFission,
            urgency: None,
            metadata: None,
        })
    }

    /// Sprawdzenie czy token jest w trendzie
    pub async fn is_trending(&self, token: &str) -> bool {
        let aggregators = self.social_aggregators.read().await;
        
        if let Some(aggregator) = aggregators.get(token) {
            let time_since_update = chrono::Utc::now().signed_duration_since(aggregator.last_update);
            
            // Token jest w trendzie jeśli był aktualizowany w ostatnich 60 sekundach
            // i ma pozytywny sentyment
            time_since_update.num_seconds() <= 60 && aggregator.avg_sentiment > 0.0
        } else {
            false
        }
    }
}

#[async_trait]
impl MemcoinStrategy for SocialFissionStrategy {
    fn name(&self) -> &str {
        "SOCIAL FISSION"
    }

    fn strategy_type(&self) -> StrategyType {
        StrategyType::SocialFission
    }

    async fn process_signal(&self, signal: &(dyn std::any::Any + Send + Sync)) -> Result<Option<TradingSignal>> {
        if !self.is_active {
            return Ok(None);
        }

        // Próba konwersji sygnału na SocialSignal
        if let Some(social_signal) = signal.downcast_ref::<SocialSignal>() {
            debug!("Processing social signal for {}: intensity={}, sentiment={}", 
                   social_signal.token, social_signal.intensity, social_signal.sentiment);

            // Aktualizuj bufory
            self.update_hype_buffer(social_signal.clone()).await;
            self.update_social_aggregator(social_signal).await;

            // Symulacja volume (w rzeczywistej implementacji pobrane z Helius)
            let simulated_volume = 1500.0; // > 1000 SOL threshold

            // Generuj sygnał handlowy
            if let Some(trading_signal) = self.generate_trading_signal(&social_signal.token, simulated_volume).await {
                return Ok(Some(trading_signal));
            }
        }

        Ok(None)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    async fn activate(&mut self) -> Result<()> {
        info!("🚀 Activating SOCIAL FISSION strategy");
        self.is_active = true;
        Ok(())
    }

    async fn deactivate(&mut self) -> Result<()> {
        info!("🛑 Deactivating SOCIAL FISSION strategy");
        self.is_active = false;
        Ok(())
    }

    async fn update_params(&mut self, params: MemcoinStrategyParams) -> Result<()> {
        info!("🔧 Updating SOCIAL FISSION parameters");
        self.strategy_params = params;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hype_score_calculation() {
        let strategy = SocialFissionStrategy::new(1000.0);
        
        let signal = SocialSignal {
            token: "test_token".to_string(),
            intensity: 90.0,
            sentiment: 0.8,
            mentions_count: 5,
            source: "twitter".to_string(),
            timestamp: chrono::Utc::now(),
        };

        strategy.update_hype_buffer(signal).await;
        let score = strategy.calculate_hype_score("test_token").await;
        assert!(score > 0.0);
    }

    #[tokio::test]
    async fn test_entry_conditions() {
        let strategy = SocialFissionStrategy::new(1000.0);
        
        // Dodaj wystarczającą liczbę sygnałów
        for _ in 0..5 {
            let signal = SocialSignal {
                token: "test_token".to_string(),
                intensity: 95.0,
                sentiment: 0.9,
                mentions_count: 1,
                source: "twitter".to_string(),
                timestamp: chrono::Utc::now(),
            };
            strategy.update_hype_buffer(signal).await;
        }

        let result = strategy.check_entry_conditions("test_token", 1500.0).await;
        assert!(result);
    }
}
