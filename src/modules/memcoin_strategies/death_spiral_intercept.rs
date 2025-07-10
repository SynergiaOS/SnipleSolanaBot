//! STRATEGIA "DEATH SPIRAL INTERCEPT"
//! 
//! Cel: Krótkoterminowe wykorzystanie panic sells
//! Zasada: Wykrycie serii dużych sprzedaży → Wejście na minimach → Take profit +8%
//! Timing: Wyjście w ciągu 90 sekund

use super::*;
use crate::modules::strategy::{TradingSignal, TradeAction, StrategyType};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Parametry krytyczne dla DEATH SPIRAL INTERCEPT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathSpiralParams {
    pub panic_volume_threshold: f32,    // 5.0% podaży w 2 min
    pub price_drop_threshold: f32,      // 15.0% spadek ceny
    pub time_window: Duration,          // 120 sekund
    pub profit_target: f32,             // 8.0% zysk
    pub max_hold_time: Duration,        // 90 sekund
    pub capital_allocation: f32,        // 15% kapitału
}

impl Default for DeathSpiralParams {
    fn default() -> Self {
        Self {
            panic_volume_threshold: 5.0,
            price_drop_threshold: 15.0,
            time_window: Duration::from_secs(120),
            profit_target: 8.0,
            max_hold_time: Duration::from_secs(90),
            capital_allocation: 0.15,
        }
    }
}

/// Zdarzenie panic sell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicSellEvent {
    pub token: String,
    pub volume_percentage: f32,         // Procent podaży sprzedany
    pub price_drop: f32,                // Procent spadku ceny
    pub sell_transactions: Vec<SellTransaction>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellTransaction {
    pub signature: String,
    pub amount: f64,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Stan pozycji w death spiral
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathSpiralPosition {
    pub token: String,
    pub entry_price: f64,
    pub entry_time: chrono::DateTime<chrono::Utc>,
    pub quantity: f64,
    pub target_exit_price: f64,
    pub stop_loss_price: f64,
}

/// Implementacja strategii DEATH SPIRAL INTERCEPT
pub struct DeathSpiralInterceptStrategy {
    params: DeathSpiralParams,
    strategy_params: MemcoinStrategyParams,
    is_active: bool,
    capital: f64,
    panic_events: Arc<RwLock<VecDeque<PanicSellEvent>>>,
    active_positions: Arc<RwLock<Vec<DeathSpiralPosition>>>,
    metrics: StrategyMetrics,
}

impl DeathSpiralInterceptStrategy {
    /// Tworzy nową instancję strategii DEATH SPIRAL INTERCEPT
    pub fn new(capital: f64) -> Self {
        Self {
            params: DeathSpiralParams::default(),
            strategy_params: MemcoinStrategyParams::default(),
            is_active: false,
            capital,
            panic_events: Arc::new(RwLock::new(VecDeque::with_capacity(50))),
            active_positions: Arc::new(RwLock::new(Vec::new())),
            metrics: StrategyMetrics::default(),
        }
    }

    /// Wykrywanie panic sell events
    pub async fn detect_panic_sell(&self, token: &str, transactions: &[SellTransaction], current_price: f64, token_supply: f64) -> Option<PanicSellEvent> {
        let now = chrono::Utc::now();
        
        // Filtruj transakcje z ostatnich 2 minut
        let recent_sells: Vec<_> = transactions.iter()
            .filter(|tx| now.signed_duration_since(tx.timestamp) <= chrono::Duration::from_std(self.params.time_window).unwrap_or_default())
            .collect();

        if recent_sells.is_empty() {
            return None;
        }

        // Oblicz całkowity volume sprzedaży
        let total_sell_volume: f64 = recent_sells.iter().map(|tx| tx.amount).sum();
        let volume_percentage = (total_sell_volume / token_supply) * 100.0;

        // Sprawdź próg volume
        if volume_percentage < self.params.panic_volume_threshold as f64 {
            debug!("Volume threshold not met for {}: {}% < {}%", 
                   token, volume_percentage, self.params.panic_volume_threshold);
            return None;
        }

        // Oblicz spadek ceny (porównaj z najwyższą ceną z ostatnich transakcji)
        let max_recent_price = recent_sells.iter()
            .map(|tx| tx.price)
            .fold(0.0f64, |a, b| a.max(b));

        let price_drop = if max_recent_price > 0.0 {
            ((max_recent_price - current_price) / max_recent_price) * 100.0
        } else {
            0.0
        };

        // Sprawdź próg spadku ceny
        if price_drop < self.params.price_drop_threshold as f64 {
            debug!("Price drop threshold not met for {}: {}% < {}%", 
                   token, price_drop, self.params.price_drop_threshold);
            return None;
        }

        info!("🚨 PANIC SELL detected for {}: volume={}%, price_drop={}%", 
              token, volume_percentage, price_drop);

        Some(PanicSellEvent {
            token: token.to_string(),
            volume_percentage: volume_percentage as f32,
            price_drop: price_drop as f32,
            sell_transactions: recent_sells.into_iter().cloned().collect(),
            timestamp: now,
        })
    }

    /// Sprawdzenie czy to dobre minimum do wejścia
    pub fn is_good_entry_point(&self, panic_event: &PanicSellEvent, current_price: f64) -> bool {
        // Sprawdź czy cena spadła wystarczająco
        if panic_event.price_drop < self.params.price_drop_threshold {
            return false;
        }

        // Sprawdź czy volume był wystarczający (oznacza kapitulację)
        if panic_event.volume_percentage < self.params.panic_volume_threshold {
            return false;
        }

        // Sprawdź czy nie minęło za dużo czasu od panic sell
        let time_since_panic = chrono::Utc::now().signed_duration_since(panic_event.timestamp);
        if time_since_panic > chrono::Duration::from_std(self.params.time_window).unwrap_or_default() {
            debug!("Too much time passed since panic sell: {} seconds", time_since_panic.num_seconds());
            return false;
        }

        info!("✅ Good entry point identified for {} at price {}", panic_event.token, current_price);
        true
    }

    /// Kalkulacja rozmiaru pozycji dla death spiral
    pub fn calculate_position_size(&self, panic_event: &PanicSellEvent) -> f64 {
        let base_size = self.capital * self.params.capital_allocation as f64;
        
        // Boost na podstawie intensywności panic sell
        let panic_intensity = (panic_event.volume_percentage / 10.0).min(2.0); // Max 2x boost
        let price_drop_boost = (panic_event.price_drop / 20.0).min(1.5); // Max 1.5x boost
        
        let boosted_size = base_size * (1.0 + panic_intensity as f64 * 0.3) * (1.0 + price_drop_boost as f64 * 0.2);
        
        debug!("Position size calculation: base={}, panic_boost={}, price_boost={}, final={}", 
               base_size, panic_intensity, price_drop_boost, boosted_size);
        
        boosted_size.min(self.strategy_params.max_position_size)
    }

    /// Generowanie sygnału wejścia na minimum
    pub async fn generate_entry_signal(&self, panic_event: &PanicSellEvent, current_price: f64) -> Option<TradingSignal> {
        if !self.is_good_entry_point(panic_event, current_price) {
            return None;
        }

        let position_size = self.calculate_position_size(panic_event);
        
        // Sprawdź minimalny rozmiar pozycji
        if position_size < 1.0 {
            warn!("Position size too small: {} SOL", position_size);
            return None;
        }

        // Kalkulacja target price (+8% profit)
        let target_price = current_price * (1.0 + self.params.profit_target as f64 / 100.0);
        
        // Kalkulacja confidence na podstawie intensywności panic sell
        let confidence = ((panic_event.volume_percentage / 10.0) * 0.4 + (panic_event.price_drop / 20.0) * 0.6).min(1.0);

        info!("🎯 DEATH SPIRAL entry signal for {}: size={} SOL, target={}, confidence={}", 
              panic_event.token, position_size, target_price, confidence);

        Some(TradingSignal {
            signal_id: uuid::Uuid::new_v4().to_string(),
            symbol: panic_event.token.clone(),
            action: TradeAction::Buy,
            quantity: position_size,
            target_price,
            confidence: confidence as f64,
            timestamp: chrono::Utc::now(),
            strategy_type: StrategyType::DeathSpiralIntercept,
        })
    }

    /// Dodanie aktywnej pozycji
    pub async fn add_active_position(&self, position: DeathSpiralPosition) {
        let mut positions = self.active_positions.write().await;
        positions.push(position);
        info!("Added active death spiral position for {}", positions.last().unwrap().token);
    }

    /// Sprawdzenie warunków wyjścia z pozycji
    pub async fn check_exit_conditions(&self) -> Vec<TradingSignal> {
        let mut exit_signals = Vec::new();
        let mut positions = self.active_positions.write().await;
        let now = chrono::Utc::now();

        positions.retain(|position| {
            let time_held = now.signed_duration_since(position.entry_time);
            
            // Sprawdź max hold time (90 sekund)
            if time_held > chrono::Duration::from_std(self.params.max_hold_time).unwrap_or_default() {
                info!("⏰ Max hold time reached for {}, generating exit signal", position.token);
                
                let exit_signal = TradingSignal {
                    signal_id: uuid::Uuid::new_v4().to_string(),
                    symbol: position.token.clone(),
                    action: TradeAction::Sell,
                    quantity: position.quantity,
                    target_price: 0.0, // Market order dla szybkości
                    confidence: 0.9,
                    timestamp: now,
                    strategy_type: StrategyType::DeathSpiralIntercept,
                };
                
                exit_signals.push(exit_signal);
                return false; // Usuń pozycję
            }

            true // Zachowaj pozycję
        });

        exit_signals
    }

    /// Aktualizacja bufora panic events
    pub async fn update_panic_buffer(&self, event: PanicSellEvent) {
        let mut buffer = self.panic_events.write().await;
        
        buffer.push_back(event);
        
        // Utrzymuj maksymalnie 50 zdarzeń
        if buffer.len() > 50 {
            buffer.pop_front();
        }
        
        debug!("Panic events buffer updated, size: {}", buffer.len());
    }

    /// Analiza historycznych panic sells
    pub async fn analyze_panic_patterns(&self, token: &str) -> f32 {
        let buffer = self.panic_events.read().await;
        
        let token_events: Vec<_> = buffer.iter()
            .filter(|e| e.token == token)
            .collect();

        if token_events.is_empty() {
            return 0.0;
        }

        // Oblicz średnią intensywność panic sells dla tokena
        let avg_intensity: f32 = token_events.iter()
            .map(|e| e.volume_percentage + e.price_drop)
            .sum::<f32>() / token_events.len() as f32;

        debug!("Average panic intensity for {}: {}", token, avg_intensity);
        avg_intensity
    }
}

#[async_trait]
impl MemcoinStrategy for DeathSpiralInterceptStrategy {
    fn name(&self) -> &str {
        "DEATH SPIRAL INTERCEPT"
    }

    fn strategy_type(&self) -> StrategyType {
        StrategyType::DeathSpiralIntercept
    }

    async fn process_signal(&self, signal: &(dyn std::any::Any + Send + Sync)) -> Result<Option<TradingSignal>> {
        if !self.is_active {
            return Ok(None);
        }

        // Sprawdź warunki wyjścia z istniejących pozycji
        let exit_signals = self.check_exit_conditions().await;
        if !exit_signals.is_empty() {
            return Ok(exit_signals.into_iter().next()); // Zwróć pierwszy sygnał wyjścia
        }

        // Próba konwersji sygnału na PanicSellEvent
        if let Some(panic_event) = signal.downcast_ref::<PanicSellEvent>() {
            debug!("Processing panic sell event for {}: volume={}%, drop={}%", 
                   panic_event.token, panic_event.volume_percentage, panic_event.price_drop);

            // Aktualizuj bufor
            self.update_panic_buffer(panic_event.clone()).await;

            // Symulacja current price (w rzeczywistej implementacji z market data)
            let simulated_current_price = 100.0; // Placeholder

            // Generuj sygnał wejścia
            if let Some(trading_signal) = self.generate_entry_signal(panic_event, simulated_current_price).await {
                // Dodaj pozycję do aktywnych
                let position = DeathSpiralPosition {
                    token: panic_event.token.clone(),
                    entry_price: simulated_current_price,
                    entry_time: chrono::Utc::now(),
                    quantity: trading_signal.quantity,
                    target_exit_price: trading_signal.target_price,
                    stop_loss_price: simulated_current_price * 0.95, // 5% stop loss
                };
                
                self.add_active_position(position).await;
                return Ok(Some(trading_signal));
            }
        }

        Ok(None)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    async fn activate(&mut self) -> Result<()> {
        info!("🚨 Activating DEATH SPIRAL INTERCEPT strategy");
        self.is_active = true;
        Ok(())
    }

    async fn deactivate(&mut self) -> Result<()> {
        info!("🛑 Deactivating DEATH SPIRAL INTERCEPT strategy");
        self.is_active = false;
        Ok(())
    }

    async fn update_params(&mut self, params: MemcoinStrategyParams) -> Result<()> {
        info!("🔧 Updating DEATH SPIRAL INTERCEPT parameters");
        self.strategy_params = params;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_panic_sell_detection() {
        let strategy = DeathSpiralInterceptStrategy::new(1000.0);
        
        let transactions = vec![
            SellTransaction {
                signature: "tx1".to_string(),
                amount: 500.0, // 5% of 10000 supply
                price: 100.0,
                timestamp: chrono::Utc::now(),
            }
        ];

        let panic_event = strategy.detect_panic_sell("test_token", &transactions, 85.0, 10000.0).await;
        assert!(panic_event.is_some());
        
        let event = panic_event.unwrap();
        assert!(event.volume_percentage >= 5.0);
        assert!(event.price_drop >= 15.0);
    }

    #[test]
    fn test_position_size_calculation() {
        let strategy = DeathSpiralInterceptStrategy::new(1000.0);
        
        let panic_event = PanicSellEvent {
            token: "test_token".to_string(),
            volume_percentage: 8.0,
            price_drop: 20.0,
            sell_transactions: vec![],
            timestamp: chrono::Utc::now(),
        };

        let size = strategy.calculate_position_size(&panic_event);
        assert!(size > 0.0);
        assert!(size <= strategy.strategy_params.max_position_size);
    }
}
