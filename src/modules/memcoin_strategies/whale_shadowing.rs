//! STRATEGIA "WHALE SHADOWING"
//! 
//! Cel: Śledzenie i preemptywne działanie za wielorybami
//! Technika: Monitorowanie kont > 10% podaży + Analiza wzorców + ML-based front-running
//! Źródła: Helius API + On-chain transaction analysis

use super::*;
use crate::modules::strategy::{TradingSignal, TradeAction, StrategyType};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Parametry dla WHALE SHADOWING
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleShadowingParams {
    pub min_whale_percentage: f32,      // 10% podaży tokena
    pub accumulation_threshold: f64,    // 3.5x średni volume
    pub pre_dump_tx_count: u32,         // 20 transakcji w 15 min
    pub volume_std_threshold: f64,      // 0.1 standard deviation
    pub capital_allocation: f32,        // 20% kapitału
    pub front_run_delay_ms: u64,        // 50ms delay dla front-run
}

impl Default for WhaleShadowingParams {
    fn default() -> Self {
        Self {
            min_whale_percentage: 10.0,
            accumulation_threshold: 3.5,
            pre_dump_tx_count: 20,
            volume_std_threshold: 0.1,
            capital_allocation: 0.20,
            front_run_delay_ms: 50,
        }
    }
}

/// Profil wieloryba
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleProfile {
    pub wallet_address: String,
    pub token: String,
    pub holdings_percentage: f32,
    pub transaction_history: VecDeque<WhaleTransaction>,
    pub behavior_pattern: WhaleBehaviorPattern,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub risk_score: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhaleBehaviorPattern {
    Accumulator,    // Systematycznie kupuje
    Dumper,         // Skłonny do dumpowania
    Swing,          // Swing trading
    Hodler,         // Długoterminowy holder
    Unknown,
}

/// Statystyki transakcyjne
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStats {
    pub last_5m_volume: f64,
    pub avg_hour_volume: f64,
    pub last_15m_count: u32,
    pub volume_std: f64,
}

/// Implementacja strategii WHALE SHADOWING
pub struct WhaleShadowingStrategy {
    params: WhaleShadowingParams,
    strategy_params: MemcoinStrategyParams,
    is_active: bool,
    capital: f64,
    whale_profiles: Arc<RwLock<HashMap<String, WhaleProfile>>>,
    transaction_buffer: Arc<RwLock<VecDeque<WhaleTransaction>>>,
    metrics: StrategyMetrics,
}

impl WhaleShadowingStrategy {
    /// Tworzy nową instancję strategii WHALE SHADOWING
    pub fn new(capital: f64) -> Self {
        Self {
            params: WhaleShadowingParams::default(),
            strategy_params: MemcoinStrategyParams::default(),
            is_active: false,
            capital,
            whale_profiles: Arc::new(RwLock::new(HashMap::new())),
            transaction_buffer: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            metrics: StrategyMetrics::default(),
        }
    }

    /// Wykrywanie wzorców wielorybów
    pub fn detect_whale_pattern(&self, tx_history: &[WhaleTransaction]) -> WhaleAction {
        if tx_history.is_empty() {
            return WhaleAction::Idle;
        }

        let stats = self.calculate_transaction_stats(tx_history);
        
        // Wykrywanie akumulacji
        if stats.last_5m_volume > stats.avg_hour_volume * self.params.accumulation_threshold {
            info!("🐋 Whale accumulation detected: {}x average volume", 
                  stats.last_5m_volume / stats.avg_hour_volume);
            return WhaleAction::Accumulation;
        }
        
        // Wzorzec przygotowania do dumpa
        if stats.last_15m_count > self.params.pre_dump_tx_count && stats.volume_std < self.params.volume_std_threshold {
            warn!("🚨 Whale pre-dump pattern detected: {} transactions, std={}", 
                  stats.last_15m_count, stats.volume_std);
            return WhaleAction::PreDump;
        }
        
        WhaleAction::Idle
    }

    /// Kalkulacja statystyk transakcyjnych
    pub fn calculate_transaction_stats(&self, tx_history: &[WhaleTransaction]) -> TransactionStats {
        let now = chrono::Utc::now();
        
        // Ostatnie 5 minut
        let last_5m: Vec<_> = tx_history.iter()
            .filter(|tx| now.signed_duration_since(tx.timestamp).num_minutes() <= 5)
            .collect();
        
        // Ostatnia godzina
        let last_hour: Vec<_> = tx_history.iter()
            .filter(|tx| now.signed_duration_since(tx.timestamp).num_hours() <= 1)
            .collect();
        
        // Ostatnie 15 minut
        let last_15m: Vec<_> = tx_history.iter()
            .filter(|tx| now.signed_duration_since(tx.timestamp).num_minutes() <= 15)
            .collect();

        let last_5m_volume: f64 = last_5m.iter().map(|tx| tx.amount).sum();
        let avg_hour_volume: f64 = if last_hour.is_empty() { 
            0.0 
        } else { 
            last_hour.iter().map(|tx| tx.amount).sum::<f64>() / last_hour.len() as f64 
        };
        
        let last_15m_count = last_15m.len() as u32;
        
        // Oblicz standard deviation volume
        let volumes: Vec<f64> = tx_history.iter().map(|tx| tx.amount).collect();
        let volume_std = self.calculate_standard_deviation(&volumes);

        TransactionStats {
            last_5m_volume,
            avg_hour_volume,
            last_15m_count,
            volume_std,
        }
    }

    /// Kalkulacja standard deviation
    fn calculate_standard_deviation(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        variance.sqrt()
    }

    /// Aktualizacja profilu wieloryba
    pub async fn update_whale_profile(&self, transaction: WhaleTransaction) {
        let mut profiles = self.whale_profiles.write().await;
        
        let profile = profiles.entry(transaction.wallet.clone()).or_insert_with(|| {
            WhaleProfile {
                wallet_address: transaction.wallet.clone(),
                token: transaction.token.clone(),
                holdings_percentage: 0.0, // Będzie aktualizowane przez Helius API
                transaction_history: VecDeque::with_capacity(100),
                behavior_pattern: WhaleBehaviorPattern::Unknown,
                last_activity: chrono::Utc::now(),
                risk_score: 0.5,
            }
        });

        // Dodaj transakcję do historii
        profile.transaction_history.push_back(transaction.clone());
        if profile.transaction_history.len() > 100 {
            profile.transaction_history.pop_front();
        }

        // Aktualizuj wzorzec zachowania
        let tx_vec: Vec<_> = profile.transaction_history.iter().cloned().collect();
        profile.behavior_pattern = self.analyze_behavior_pattern(&tx_vec);
        profile.last_activity = transaction.timestamp;

        debug!("Updated whale profile for {}: pattern={:?}, transactions={}", 
               transaction.wallet, profile.behavior_pattern, profile.transaction_history.len());
    }

    /// Analiza wzorca zachowania wieloryba
    fn analyze_behavior_pattern(&self, transactions: &[WhaleTransaction]) -> WhaleBehaviorPattern {
        if transactions.len() < 5 {
            return WhaleBehaviorPattern::Unknown;
        }

        let buy_count = transactions.iter().filter(|tx| matches!(tx.action, WhaleAction::Accumulation)).count();
        let total_count = transactions.len();
        let buy_ratio = buy_count as f32 / total_count as f32;

        // Klasyfikacja na podstawie ratio kupna/sprzedaży
        match buy_ratio {
            r if r > 0.8 => WhaleBehaviorPattern::Accumulator,
            r if r < 0.2 => WhaleBehaviorPattern::Dumper,
            r if r >= 0.4 && r <= 0.6 => WhaleBehaviorPattern::Swing,
            _ => WhaleBehaviorPattern::Hodler,
        }
    }

    /// Generowanie sygnału front-run
    pub async fn generate_front_run_signal(&self, whale_action: WhaleAction, token: &str) -> Option<TradingSignal> {
        let action = match whale_action {
            WhaleAction::Accumulation => {
                info!("🐋 Front-running whale accumulation for {}", token);
                TradeAction::Buy
            },
            WhaleAction::PreDump => {
                warn!("🚨 Front-running whale dump for {}", token);
                TradeAction::Sell
            },
            WhaleAction::Idle => return None,
        };

        // Kalkulacja rozmiaru pozycji
        let base_size = self.capital * self.params.capital_allocation as f64;
        let position_size = base_size.min(self.strategy_params.max_position_size);

        // Confidence na podstawie typu akcji
        let confidence = match whale_action {
            WhaleAction::Accumulation => 0.85, // Wysokie zaufanie do akumulacji
            WhaleAction::PreDump => 0.75,      // Średnie zaufanie do pre-dump
            WhaleAction::Idle => 0.0,
        };

        Some(TradingSignal {
            signal_id: uuid::Uuid::new_v4().to_string(),
            symbol: token.to_string(),
            action,
            quantity: position_size,
            target_price: 0.0, // Market order dla szybkości
            confidence,
            timestamp: chrono::Utc::now(),
            strategy_type: StrategyType::WhaleShadowing,
        })
    }

    /// Sprawdzenie czy wallet jest wielorybem
    pub async fn is_whale(&self, wallet: &str, token: &str) -> bool {
        let profiles = self.whale_profiles.read().await;
        
        if let Some(profile) = profiles.get(wallet) {
            profile.token == token && profile.holdings_percentage >= self.params.min_whale_percentage
        } else {
            false
        }
    }

    /// Aktualizacja bufora transakcji
    pub async fn update_transaction_buffer(&self, transaction: WhaleTransaction) {
        let mut buffer = self.transaction_buffer.write().await;
        
        buffer.push_back(transaction);
        
        // Utrzymuj maksymalnie 1000 transakcji
        if buffer.len() > 1000 {
            buffer.pop_front();
        }
    }

    /// Analiza ryzyka wieloryba
    pub async fn calculate_whale_risk(&self, wallet: &str) -> f32 {
        let profiles = self.whale_profiles.read().await;
        
        if let Some(profile) = profiles.get(wallet) {
            match profile.behavior_pattern {
                WhaleBehaviorPattern::Dumper => 0.9,        // Wysokie ryzyko
                WhaleBehaviorPattern::Swing => 0.6,         // Średnie ryzyko
                WhaleBehaviorPattern::Accumulator => 0.3,   // Niskie ryzyko
                WhaleBehaviorPattern::Hodler => 0.2,        // Bardzo niskie ryzyko
                WhaleBehaviorPattern::Unknown => 0.5,       // Neutralne ryzyko
            }
        } else {
            0.5 // Domyślne ryzyko dla nieznanych wielorybów
        }
    }
}

#[async_trait]
impl MemcoinStrategy for WhaleShadowingStrategy {
    fn name(&self) -> &str {
        "WHALE SHADOWING"
    }

    fn strategy_type(&self) -> StrategyType {
        StrategyType::WhaleShadowing
    }

    async fn process_signal(&self, signal: &(dyn std::any::Any + Send + Sync)) -> Result<Option<TradingSignal>> {
        if !self.is_active {
            return Ok(None);
        }

        // Próba konwersji sygnału na WhaleTransaction
        if let Some(whale_tx) = signal.downcast_ref::<WhaleTransaction>() {
            debug!("Processing whale transaction: {} {} {} SOL", 
                   whale_tx.wallet, whale_tx.action, whale_tx.amount);

            // Sprawdź czy to wieloryb
            if !self.is_whale(&whale_tx.wallet, &whale_tx.token).await {
                debug!("Wallet {} is not a whale for token {}", whale_tx.wallet, whale_tx.token);
                return Ok(None);
            }

            // Aktualizuj profile i bufory
            self.update_whale_profile(whale_tx.clone()).await;
            self.update_transaction_buffer(whale_tx.clone()).await;

            // Pobierz historię transakcji dla analizy wzorców
            let profiles = self.whale_profiles.read().await;
            if let Some(profile) = profiles.get(&whale_tx.wallet) {
                let tx_history: Vec<_> = profile.transaction_history.iter().cloned().collect();
                drop(profiles); // Zwolnij lock

                let whale_action = self.detect_whale_pattern(&tx_history);
                
                // Generuj sygnał front-run jeśli wykryto wzorzec
                if let Some(trading_signal) = self.generate_front_run_signal(whale_action, &whale_tx.token).await {
                    return Ok(Some(trading_signal));
                }
            }
        }

        Ok(None)
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    async fn activate(&mut self) -> Result<()> {
        info!("🐋 Activating WHALE SHADOWING strategy");
        self.is_active = true;
        Ok(())
    }

    async fn deactivate(&mut self) -> Result<()> {
        info!("🛑 Deactivating WHALE SHADOWING strategy");
        self.is_active = false;
        Ok(())
    }

    async fn update_params(&mut self, params: MemcoinStrategyParams) -> Result<()> {
        info!("🔧 Updating WHALE SHADOWING parameters");
        self.strategy_params = params;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whale_pattern_detection() {
        let strategy = WhaleShadowingStrategy::new(1000.0);
        
        let mut transactions = Vec::new();
        for i in 0..25 {
            transactions.push(WhaleTransaction {
                signature: format!("tx_{}", i),
                wallet: "whale_wallet".to_string(),
                token: "test_token".to_string(),
                amount: 100.0,
                action: WhaleAction::Accumulation,
                timestamp: chrono::Utc::now() - chrono::Duration::minutes(i),
            });
        }

        let action = strategy.detect_whale_pattern(&transactions);
        assert!(matches!(action, WhaleAction::PreDump)); // 25 > 20 transactions
    }

    #[test]
    fn test_standard_deviation() {
        let strategy = WhaleShadowingStrategy::new(1000.0);
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let std = strategy.calculate_standard_deviation(&values);
        assert!(std > 0.0);
    }
}
