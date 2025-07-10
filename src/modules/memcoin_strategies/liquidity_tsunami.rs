//! STRATEGIA "LIQUIDITY TSUNAMI"
//! 
//! Cel: Wykorzystanie nagłych zmian płynności
//! Mechanizm: Wykrycie anomalii płynności w DLMM + Kinetic Capital Allocation
//! Wykonanie: < 120ms (UrgencyLevel::Flash)

use super::*;
use crate::modules::strategy::{TradingSignal, StrategyType};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Parametry kluczowe dla LIQUIDITY TSUNAMI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityTsunamiParams {
    pub liquidity_threshold: f64,    // 50 SOL
    pub velocity_threshold: f64,     // 0.7
    pub urgency_timeout_ms: u64,     // 120ms
    pub capital_allocation: f32,     // 15% kapitału
}

impl Default for LiquidityTsunamiParams {
    fn default() -> Self {
        Self {
            liquidity_threshold: 50.0,
            velocity_threshold: 0.7,
            urgency_timeout_ms: 120,
            capital_allocation: 0.15,
        }
    }
}

/// Implementacja strategii LIQUIDITY TSUNAMI
pub struct LiquidityTsunamiStrategy {
    params: LiquidityTsunamiParams,
    strategy_params: MemcoinStrategyParams,
    is_active: bool,
    capital: f64,
    liquidity_buffer: Arc<RwLock<VecDeque<LiquidityEvent>>>,
    metrics: StrategyMetrics,
}

impl LiquidityTsunamiStrategy {
    /// Tworzy nową instancję strategii LIQUIDITY TSUNAMI
    pub fn new(capital: f64) -> Self {
        Self {
            params: LiquidityTsunamiParams::default(),
            strategy_params: MemcoinStrategyParams::default(),
            is_active: false,
            capital,
            liquidity_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            metrics: StrategyMetrics::default(),
        }
    }

    /// Wykrycie anomalii płynności w DLMM
    pub async fn detect_liquidity_anomaly(&self, event: &LiquidityEvent) -> bool {
        debug!("Analyzing liquidity event for {}: delta={}, velocity={}", 
               event.mint, event.delta, event.velocity);

        // Sprawdzenie progów
        if event.delta > self.params.liquidity_threshold && event.velocity > self.params.velocity_threshold {
            info!("🌊 LIQUIDITY TSUNAMI detected for {}: delta={} SOL, velocity={}", 
                  event.mint, event.delta, event.velocity);
            return true;
        }

        false
    }

    /// Kinetic Capital Allocation - dynamiczne przydzielanie kapitału
    pub fn calculate_position_size(&self, event: &LiquidityEvent) -> f64 {
        // Bazowy rozmiar pozycji
        let base_size = self.capital * self.params.capital_allocation as f64;
        
        // Korekta na zmienność (wyższa zmienność = mniejsza pozycja)
        let volatility_adjustment = 1.0 - (event.volatility / 2.0).min(0.5);
        
        // Korekta na velocity (wyższa velocity = większa pozycja)
        let velocity_boost = 1.0 + (event.velocity - self.params.velocity_threshold) * 0.5;
        
        let adjusted_size = base_size * volatility_adjustment * velocity_boost;
        
        debug!("Position size calculation: base={}, volatility_adj={}, velocity_boost={}, final={}", 
               base_size, volatility_adjustment, velocity_boost, adjusted_size);
        
        adjusted_size.min(self.strategy_params.max_position_size)
    }

    /// Generowanie bundle transakcji
    pub async fn generate_trade_bundle(&self, event: &LiquidityEvent) -> Option<TradeBundle> {
        if !self.detect_liquidity_anomaly(event).await {
            return None;
        }

        let position_size = self.calculate_position_size(event);
        
        // Sprawdzenie minimalnego rozmiaru pozycji
        if position_size < 1.0 {
            warn!("Position size too small: {} SOL", position_size);
            return None;
        }

        // Kalkulacja slippage na podstawie velocity
        let slippage = (0.5 + (event.velocity - self.params.velocity_threshold) * 0.3).min(2.0);

        Some(TradeBundle {
            action: crate::modules::memcoin_strategies::TradeAction::MarketBuy,
            token: event.mint.clone(),
            amount: position_size,
            slippage,
            urgency: UrgencyLevel::Flash,
        })
    }

    /// Aktualizacja bufora płynności
    pub async fn update_liquidity_buffer(&self, event: LiquidityEvent) {
        let mut buffer = self.liquidity_buffer.write().await;
        
        // Dodaj nowe zdarzenie
        buffer.push_back(event);
        
        // Utrzymuj maksymalnie 100 zdarzeń
        if buffer.len() > 100 {
            buffer.pop_front();
        }
        
        debug!("Liquidity buffer updated, size: {}", buffer.len());
    }

    /// Analiza historycznych danych płynności
    pub async fn analyze_liquidity_patterns(&self) -> f64 {
        let buffer = self.liquidity_buffer.read().await;
        
        if buffer.is_empty() {
            return 0.0;
        }

        // Oblicz średnią velocity z ostatnich 10 zdarzeń
        let recent_events: Vec<_> = buffer.iter().rev().take(10).collect();
        let avg_velocity: f64 = recent_events.iter()
            .map(|e| e.velocity)
            .sum::<f64>() / recent_events.len() as f64;

        debug!("Average velocity from last {} events: {}", recent_events.len(), avg_velocity);
        avg_velocity
    }

    /// Sprawdzenie warunków wyjścia z pozycji
    pub fn should_exit_position(&self, current_price: f64, entry_price: f64, entry_time: chrono::DateTime<chrono::Utc>) -> bool {
        let price_change = (current_price - entry_price) / entry_price * 100.0;
        let time_held = chrono::Utc::now().signed_duration_since(entry_time);

        // Take profit
        if price_change >= self.strategy_params.profit_target as f64 {
            info!("🎯 Take profit triggered: {}%", price_change);
            return true;
        }

        // Stop loss
        if price_change <= -(self.strategy_params.stop_loss as f64) {
            warn!("🛑 Stop loss triggered: {}%", price_change);
            return true;
        }

        // Max hold time
        if time_held > chrono::Duration::from_std(self.strategy_params.max_hold_time).unwrap_or_default() {
            info!("⏰ Max hold time reached");
            return true;
        }

        false
    }
}

#[async_trait]
impl MemcoinStrategy for LiquidityTsunamiStrategy {
    fn name(&self) -> &str {
        "LIQUIDITY TSUNAMI"
    }

    fn strategy_type(&self) -> StrategyType {
        StrategyType::LiquidityTsunami
    }

    async fn process_signal(&self, signal: &(dyn std::any::Any + Send + Sync)) -> Result<Option<TradingSignal>> {
        if !self.is_active {
            return Ok(None);
        }

        // Próba konwersji sygnału na LiquidityEvent
        if let Some(liquidity_event) = signal.downcast_ref::<LiquidityEvent>() {
            debug!("Processing liquidity event for {}", liquidity_event.mint);

            // Aktualizuj bufor
            self.update_liquidity_buffer(liquidity_event.clone()).await;

            // Generuj bundle transakcji
            if let Some(trade_bundle) = self.generate_trade_bundle(liquidity_event).await {
                info!("🌊 LIQUIDITY TSUNAMI signal generated for {}: {} SOL", 
                      trade_bundle.token, trade_bundle.amount);

                let trading_signal = TradingSignal {
                    signal_id: uuid::Uuid::new_v4().to_string(),
                    symbol: trade_bundle.token,
                    action: match trade_bundle.action {
                        crate::modules::memcoin_strategies::TradeAction::MarketBuy |
                        crate::modules::memcoin_strategies::TradeAction::LimitBuy => crate::modules::strategy::TradeAction::Buy,
                        crate::modules::memcoin_strategies::TradeAction::MarketSell |
                        crate::modules::memcoin_strategies::TradeAction::LimitSell => crate::modules::strategy::TradeAction::Sell,
                    },
                    quantity: trade_bundle.amount,
                    target_price: 0.0, // Market order
                    price: None, // Market order
                    confidence: (liquidity_event.velocity * 0.8 + 0.2).min(1.0),
                    timestamp: chrono::Utc::now(),
                    strategy_type: StrategyType::LiquidityTsunami,
                    urgency: None,
                    metadata: None,
                };

                return Ok(Some(trading_signal));
            }
        }

        Ok(None)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    async fn activate(&mut self) -> Result<()> {
        info!("🌊 Activating LIQUIDITY TSUNAMI strategy");
        self.is_active = true;
        Ok(())
    }

    async fn deactivate(&mut self) -> Result<()> {
        info!("🛑 Deactivating LIQUIDITY TSUNAMI strategy");
        self.is_active = false;
        Ok(())
    }

    async fn update_params(&mut self, params: MemcoinStrategyParams) -> Result<()> {
        info!("🔧 Updating LIQUIDITY TSUNAMI parameters");
        self.strategy_params = params;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liquidity_anomaly_detection() {
        let strategy = LiquidityTsunamiStrategy::new(1000.0);
        
        let event = LiquidityEvent {
            mint: "test_token".to_string(),
            delta: 60.0,  // Above threshold
            velocity: 0.8, // Above threshold
            volatility: 0.3,
            timestamp: chrono::Utc::now(),
        };

        assert!(strategy.detect_liquidity_anomaly(&event).await);
    }

    #[test]
    fn test_position_size_calculation() {
        let strategy = LiquidityTsunamiStrategy::new(1000.0);
        
        let event = LiquidityEvent {
            mint: "test_token".to_string(),
            delta: 60.0,
            velocity: 0.8,
            volatility: 0.2,
            timestamp: chrono::Utc::now(),
        };

        let size = strategy.calculate_position_size(&event);
        assert!(size > 0.0);
        assert!(size <= strategy.strategy_params.max_position_size);
    }
}
