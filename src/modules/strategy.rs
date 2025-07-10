// Strategy Engine Module
// Analyzes market data and generates trading signals

use crate::modules::data_ingestor::MarketData;
use crate::modules::memcoin_strategies::{
    MemcoinStrategy, LiquidityEvent, SocialSignal, WhaleTransaction,
    liquidity_tsunami::LiquidityTsunamiStrategy,
    social_fission::SocialFissionStrategy,
    whale_shadowing::WhaleShadowingStrategy,
    death_spiral_intercept::{DeathSpiralInterceptStrategy, PanicSellEvent},
    meme_virus::MemeVirusStrategy,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub signal_id: String,
    pub symbol: String,
    pub action: TradeAction,
    pub quantity: f64,
    pub target_price: f64,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub strategy_type: StrategyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeAction {
    Buy,
    Sell,
    Hold,
    MarketBuy,
    MarketSell,
}

impl std::fmt::Display for TradeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradeAction::Buy => write!(f, "BUY"),
            TradeAction::Sell => write!(f, "SELL"),
            TradeAction::Hold => write!(f, "HOLD"),
            TradeAction::MarketBuy => write!(f, "MARKET_BUY"),
            TradeAction::MarketSell => write!(f, "MARKET_SELL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum StrategyType {
    TokenSniping,
    Arbitrage,
    MomentumTrading,
    SoulMeteorSniping,
    MeteoraDAMM,
    DeveloperTracking,
    AxiomMemeCoin,
    AIDecision, // AI-generated decisions
    // NEW ADVANCED STRATEGIES
    MEVArbitrage,       // MEV arbitrage opportunities
    CrossDexArbitrage,  // Cross-DEX arbitrage
    LiquiditySniping,   // Liquidity event sniping
    VolumeAnalysis,     // Volume-based trading
    SocialSentiment,    // Social sentiment analysis
    FlashLoanArbitrage, // Flash loan arbitrage
    YieldFarming,       // Yield farming optimization
    OptionsStrategy,    // Options trading strategies
    // MICRO-LIGHTNING STRATEGY
    MicroLightning,     // High-frequency micro-operations ($20/60min)
    // MEMCOIN SWARMGUARD STRATEGIES - ARCHITEKTURA WOJENNA
    LiquidityTsunami,      // Wykorzystanie nagłych zmian płynności
    SocialFission,         // Eksploatacja hype'u społecznościowego
    WhaleShadowing,        // Śledzenie i preemptywne działanie za wielorybami
    DeathSpiralIntercept,  // Krótkoterminowe wykorzystanie panic sells
    MemeVirus,             // Długoterminowe trendy memcoinowe
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::TokenSniping => write!(f, "token_sniping"),
            StrategyType::Arbitrage => write!(f, "arbitrage"),
            StrategyType::MomentumTrading => write!(f, "momentum_trading"),
            StrategyType::SoulMeteorSniping => write!(f, "soul_meteor"),
            StrategyType::MeteoraDAMM => write!(f, "meteora"),
            StrategyType::DeveloperTracking => write!(f, "developer_tracking"),
            StrategyType::AxiomMemeCoin => write!(f, "axiom_memecoin"),
            StrategyType::AIDecision => write!(f, "ai_decision"),
            StrategyType::MEVArbitrage => write!(f, "mev_arbitrage"),
            StrategyType::CrossDexArbitrage => write!(f, "cross_dex_arbitrage"),
            StrategyType::LiquiditySniping => write!(f, "liquidity_sniping"),
            StrategyType::VolumeAnalysis => write!(f, "volume_analysis"),
            StrategyType::SocialSentiment => write!(f, "social_sentiment"),
            StrategyType::FlashLoanArbitrage => write!(f, "flash_loan_arbitrage"),
            StrategyType::YieldFarming => write!(f, "yield_farming"),
            StrategyType::OptionsStrategy => write!(f, "options_strategy"),
            // MEMCOIN SWARMGUARD STRATEGIES
            StrategyType::LiquidityTsunami => write!(f, "liquidity_tsunami"),
            StrategyType::SocialFission => write!(f, "social_fission"),
            StrategyType::WhaleShadowing => write!(f, "whale_shadowing"),
            StrategyType::DeathSpiralIntercept => write!(f, "death_spiral_intercept"),
            StrategyType::MemeVirus => write!(f, "meme_virus"),
        }
    }
}

pub struct StrategyEngine {
    market_data_receiver: mpsc::UnboundedReceiver<MarketData>,
    signal_sender: mpsc::UnboundedSender<TradingSignal>,
    is_running: bool,
    // MEMCOIN SWARMGUARD STRATEGIES - używamy enum zamiast trait object
    liquidity_tsunami: Option<LiquidityTsunamiStrategy>,
    social_fission: Option<SocialFissionStrategy>,
    whale_shadowing: Option<WhaleShadowingStrategy>,
    death_spiral_intercept: Option<DeathSpiralInterceptStrategy>,
    meme_virus: Option<MemeVirusStrategy>,
    capital: f64,
}

#[allow(dead_code)]
impl StrategyEngine {
    pub fn new(
        market_data_receiver: mpsc::UnboundedReceiver<MarketData>,
        signal_sender: mpsc::UnboundedSender<TradingSignal>,
    ) -> Self {
        Self {
            market_data_receiver,
            signal_sender,
            is_running: false,
            liquidity_tsunami: None,
            social_fission: None,
            whale_shadowing: None,
            death_spiral_intercept: None,
            meme_virus: None,
            capital: 10000.0, // Default capital
        }
    }

    /// Inicjalizacja strategii memcoin
    pub fn initialize_memcoin_strategies(&mut self) -> Result<()> {
        // Inicjalizuj wszystkie strategie MEMCOIN SWARMGUARD
        self.liquidity_tsunami = Some(LiquidityTsunamiStrategy::new(self.capital));
        self.social_fission = Some(SocialFissionStrategy::new(self.capital));
        self.whale_shadowing = Some(WhaleShadowingStrategy::new(self.capital));
        self.death_spiral_intercept = Some(DeathSpiralInterceptStrategy::new(self.capital));
        self.meme_virus = Some(MemeVirusStrategy::new(self.capital));

        info!("🦾 MEMCOIN SWARMGUARD strategies initialized: 5 strategies loaded");
        Ok(())
    }

    /// Aktywacja wszystkich strategii memcoin
    pub async fn activate_memcoin_strategies(&mut self) -> Result<()> {
        if let Some(ref mut strategy) = self.liquidity_tsunami {
            strategy.activate().await?;
            info!("✅ Activated LIQUIDITY TSUNAMI strategy");
        }

        if let Some(ref mut strategy) = self.social_fission {
            strategy.activate().await?;
            info!("✅ Activated SOCIAL FISSION strategy");
        }

        if let Some(ref mut strategy) = self.whale_shadowing {
            strategy.activate().await?;
            info!("✅ Activated WHALE SHADOWING strategy");
        }

        if let Some(ref mut strategy) = self.death_spiral_intercept {
            strategy.activate().await?;
            info!("✅ Activated DEATH SPIRAL INTERCEPT strategy");
        }

        if let Some(ref mut strategy) = self.meme_virus {
            strategy.activate().await?;
            info!("✅ Activated MEME VIRUS strategy");
        }

        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("🧠 StrategyEngine starting...");
        self.is_running = true;

        while self.is_running {
            if let Some(market_data) = self.market_data_receiver.recv().await {
                self.process_market_data(market_data).await?;
            }
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        info!("🛑 StrategyEngine stopping...");
        self.is_running = false;
    }

    async fn process_market_data(&self, data: MarketData) -> Result<()> {
        debug!("Processing market data for symbol: {}", data.symbol);

        // Przetwarzanie przez strategie memcoin
        self.process_memcoin_signals(&data).await?;

        // TODO: Implement actual trading strategies
        // For now, generate a simple signal occasionally
        if data.price > 105.0 {
            // Simple condition instead of random
            let quantity = 100.0;

            // Estimate liquidity (in a real implementation, this would come from market data)
            let estimated_liquidity = data.volume * 0.1; // Simplified estimation

            // Calculate expected slippage
            let slippage = self.calculate_slippage(quantity, estimated_liquidity, data.price);

            // Adjust target price based on slippage
            let target_price = data.price * (1.01 + slippage);

            let signal = TradingSignal {
                signal_id: uuid::Uuid::new_v4().to_string(),
                symbol: data.symbol,
                action: TradeAction::Buy,
                quantity,
                target_price,
                confidence: 0.7 * (1.0 - slippage), // Lower confidence with higher slippage
                timestamp: chrono::Utc::now(),
                strategy_type: StrategyType::TokenSniping,
            };

            if let Err(e) = self.signal_sender.send(signal) {
                error!("Failed to send trading signal: {}", e);
            }
        }

        Ok(())
    }

    /// Przetwarzanie sygnałów przez strategie memcoin
    async fn process_memcoin_signals(&self, data: &MarketData) -> Result<()> {

        // Generuj różne typy sygnałów na podstawie market data

        // 1. LIQUIDITY TSUNAMI - symulacja liquidity event
        if data.volume > 1000.0 {
            let liquidity_event = LiquidityEvent {
                mint: data.symbol.clone(),
                delta: data.volume * 0.05, // 5% of volume as delta
                velocity: (data.price_change_24h.abs() / 100.0).min(1.0),
                volatility: data.price_change_24h.abs() / 100.0,
                timestamp: chrono::Utc::now(),
            };

            if let Some(ref strategy) = self.liquidity_tsunami {
                if let Ok(Some(signal)) = strategy.process_signal(&liquidity_event).await {
                    self.send_signal(signal).await?;
                }
            }
        }

        // 2. SOCIAL FISSION - symulacja social signal
        if data.price_change_1h > 5.0 {
            let social_signal = SocialSignal {
                token: data.symbol.clone(),
                intensity: (data.price_change_1h * 10.0).min(100.0) as f32,
                sentiment: if data.price_change_1h > 0.0 { 0.8 } else { -0.8 },
                mentions_count: (data.volume / 100.0) as u32,
                source: "twitter".to_string(),
                timestamp: chrono::Utc::now(),
            };

            if let Some(ref strategy) = self.social_fission {
                if let Ok(Some(signal)) = strategy.process_signal(&social_signal).await {
                    self.send_signal(signal).await?;
                }
            }
        }

        // 3. DEATH SPIRAL INTERCEPT - symulacja panic sell
        if data.price_change_1h < -15.0 && data.volume > 500.0 {
            let panic_event = PanicSellEvent {
                token: data.symbol.clone(),
                volume_percentage: ((data.volume / 10000.0) * 100.0) as f32, // Symulacja % podaży
                price_drop: data.price_change_1h.abs() as f32,
                sell_transactions: vec![], // Placeholder
                timestamp: chrono::Utc::now(),
            };

            if let Some(ref strategy) = self.death_spiral_intercept {
                if let Ok(Some(signal)) = strategy.process_signal(&panic_event).await {
                    self.send_signal(signal).await?;
                }
            }
        }

        // 4. MEME VIRUS - symulacja viral meme
        if data.price_change_24h > 50.0 {
            let social_signal = SocialSignal {
                token: data.symbol.clone(),
                intensity: 95.0,
                sentiment: 0.9,
                mentions_count: 50,
                source: "viral meme detected".to_string(),
                timestamp: chrono::Utc::now(),
            };

            if let Some(ref strategy) = self.meme_virus {
                if let Ok(Some(signal)) = strategy.process_signal(&social_signal).await {
                    self.send_signal(signal).await?;
                }
            }
        }

        Ok(())
    }

    /// Wysłanie sygnału handlowego
    async fn send_signal(&self, signal: TradingSignal) -> Result<()> {
        info!("🎯 MEMCOIN SWARMGUARD signal: {:?} {} {} SOL (confidence: {})",
              signal.strategy_type, signal.action, signal.quantity, signal.confidence);

        if let Err(e) = self.signal_sender.send(signal) {
            error!("Failed to send memcoin trading signal: {}", e);
            return Err(anyhow::anyhow!("Failed to send signal"));
        }

        Ok(())
    }

    /// Calculates expected slippage for a given order size and liquidity
    pub fn calculate_slippage(&self, order_size: f64, liquidity: f64, price: f64) -> f64 {
        // Guard against division by zero
        if liquidity <= 0.0 {
            return 1.0; // 100% slippage for zero liquidity
        }

        // Calculate impact ratio (order size relative to available liquidity)
        let impact_ratio = order_size / liquidity;

        // Apply non-linear slippage model
        // Small orders: minimal slippage
        // Large orders: exponentially increasing slippage
        let base_slippage = impact_ratio.min(0.5);

        // Apply additional factors based on price volatility
        // This is a simplified model - can be enhanced with historical volatility
        let price_factor = if price < 0.01 {
            // Micro-cap tokens have higher slippage
            1.5
        } else if price < 1.0 {
            // Low-priced tokens
            1.2
        } else {
            // Higher-priced tokens
            1.0
        };

        // Return slippage as a percentage (0.0 to 1.0)
        (base_slippage * price_factor).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_strategy_engine_creation() {
        let (_market_tx, market_rx) = mpsc::unbounded_channel();
        let (signal_tx, _signal_rx) = mpsc::unbounded_channel();

        let engine = StrategyEngine::new(market_rx, signal_tx);
        assert!(!engine.is_running);
    }

    #[test]
    fn test_calculate_slippage() {
        // Create a minimal StrategyEngine for testing
        let (_tx_market, rx_market) = mpsc::unbounded_channel();
        let (tx_signal, _) = mpsc::unbounded_channel();
        let strategy = StrategyEngine::new(rx_market, tx_signal);

        // Test case 1: Zero liquidity should result in 100% slippage
        assert_eq!(strategy.calculate_slippage(100.0, 0.0, 10.0), 1.0);

        // Test case 2: Small order relative to liquidity
        let small_order_slippage = strategy.calculate_slippage(100.0, 10000.0, 10.0);
        assert!(small_order_slippage < 0.05); // Should be less than 5%

        // Test case 3: Large order relative to liquidity
        let large_order_slippage = strategy.calculate_slippage(5000.0, 10000.0, 10.0);
        assert!(large_order_slippage > 0.2); // Should be significant

        // Test case 4: Micro-cap token (price < 0.01)
        let micro_cap_slippage = strategy.calculate_slippage(100.0, 1000.0, 0.001);
        let normal_token_slippage = strategy.calculate_slippage(100.0, 1000.0, 10.0);
        assert!(micro_cap_slippage > normal_token_slippage); // Should have higher slippage
    }
}
