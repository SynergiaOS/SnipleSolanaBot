//! MEMCOIN SWARMGUARD: ARCHITEKTURA WOJENNA
//! 
//! Zintegrowane strategie operacyjne dla agentów roju,
//! zaprojektowane do dominacji na memcoinowych polach bitwy
//! 
//! Strategie:
//! - LIQUIDITY TSUNAMI: Wykorzystanie nagłych zmian płynności
//! - SOCIAL FISSION: Eksploatacja hype'u społecznościowego  
//! - WHALE SHADOWING: Śledzenie i preemptywne działanie za wielorybami
//! - DEATH SPIRAL INTERCEPT: Krótkoterminowe wykorzystanie panic sells
//! - MEME VIRUS: Długoterminowe trendy memcoinowe

pub mod liquidity_tsunami;
pub mod social_fission;
pub mod whale_shadowing;
pub mod death_spiral_intercept;
pub mod meme_virus;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::modules::strategy::{TradingSignal, StrategyType};

/// Wspólne typy danych dla strategii memcoin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityEvent {
    pub mint: String,
    pub delta: f64,           // Zmiana płynności w SOL
    pub velocity: f64,        // Tempo zmiany (0.0 - 1.0)
    pub volatility: f64,      // Zmienność ceny
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSignal {
    pub token: String,
    pub intensity: f32,       // Intensywność sygnału (0.0 - 100.0)
    pub sentiment: f32,       // Sentyment (-1.0 do 1.0)
    pub mentions_count: u32,  // Liczba wzmianek
    pub source: String,       // Twitter, Reddit, etc.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleTransaction {
    pub signature: String,
    pub wallet: String,
    pub token: String,
    pub amount: f64,
    pub action: WhaleAction,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhaleAction {
    Accumulation,
    PreDump,
    Idle,
}

impl std::fmt::Display for WhaleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhaleAction::Accumulation => write!(f, "ACCUMULATION"),
            WhaleAction::PreDump => write!(f, "PRE_DUMP"),
            WhaleAction::Idle => write!(f, "IDLE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeBundle {
    pub action: TradeAction,
    pub token: String,
    pub amount: f64,
    pub slippage: f64,
    pub urgency: UrgencyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    MarketBuy,
    MarketSell,
    LimitBuy,
    LimitSell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Flash,      // < 120ms
    Fast,       // < 500ms
    Normal,     // < 2s
    Delayed,    // > 2s
}

/// Parametry strategii memcoin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemcoinStrategyParams {
    pub capital_allocation: f32,    // Procent kapitału (0.0 - 1.0)
    pub risk_tolerance: f32,        // Tolerancja ryzyka (0.0 - 1.0)
    pub max_position_size: f64,     // Maksymalny rozmiar pozycji w SOL
    pub profit_target: f32,         // Cel zysku w procentach
    pub stop_loss: f32,             // Stop loss w procentach
    pub max_hold_time: Duration,    // Maksymalny czas trzymania pozycji
}

impl Default for MemcoinStrategyParams {
    fn default() -> Self {
        Self {
            capital_allocation: 0.15,
            risk_tolerance: 0.6,
            max_position_size: 100.0,
            profit_target: 8.0,
            stop_loss: 5.0,
            max_hold_time: Duration::from_secs(300), // 5 minut
        }
    }
}

/// Trait dla wszystkich strategii memcoin
#[async_trait]
pub trait MemcoinStrategy: Send + Sync {
    /// Nazwa strategii
    fn name(&self) -> &str;
    
    /// Typ strategii
    fn strategy_type(&self) -> StrategyType;
    
    /// Przetwarzanie sygnału rynkowego
    async fn process_signal(&self, signal: &(dyn std::any::Any + Send + Sync)) -> Result<Option<TradingSignal>>;
    
    /// Sprawdzenie czy strategia jest aktywna
    fn is_active(&self) -> bool;
    
    /// Aktywacja strategii
    async fn activate(&mut self) -> Result<()>;
    
    /// Deaktywacja strategii
    async fn deactivate(&mut self) -> Result<()>;
    
    /// Aktualizacja parametrów strategii
    async fn update_params(&mut self, params: MemcoinStrategyParams) -> Result<()>;
}

/// Metryki sukcesu strategii
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub win_rate: f32,              // Procent wygranych transakcji
    pub roi_7d: f32,                // ROI za 7 dni
    pub max_drawdown: f32,          // Maksymalny spadek
    pub avg_trade_duration: Duration, // Średni czas transakcji
    pub total_trades: u32,          // Łączna liczba transakcji
    pub profitable_trades: u32,     // Liczba zyskownych transakcji
}

impl Default for StrategyMetrics {
    fn default() -> Self {
        Self {
            win_rate: 0.0,
            roi_7d: 0.0,
            max_drawdown: 0.0,
            avg_trade_duration: Duration::from_secs(0),
            total_trades: 0,
            profitable_trades: 0,
        }
    }
}

/// Konfiguracja KINETIC SHIELD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KineticShieldConfig {
    pub daily_drawdown_limit: f32,      // 7.5%
    pub hourly_loss_streak_limit: u32,  // 5
    pub max_volatility: f32,            // Maksymalna zmienność
    pub exposure_limit_per_token: f32,  // 12% kapitału na token
}

impl Default for KineticShieldConfig {
    fn default() -> Self {
        Self {
            daily_drawdown_limit: 7.5,
            hourly_loss_streak_limit: 5,
            max_volatility: 0.5,
            exposure_limit_per_token: 0.12,
        }
    }
}
