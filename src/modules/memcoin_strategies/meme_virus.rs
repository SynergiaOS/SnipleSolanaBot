//! STRATEGIA "MEME VIRUS" (DŁUGOTERMINOWA)
//! 
//! Cel: Wykorzystanie długotrwałych trendów memcoinowych
//! Fazy: Akumulacja (25%) → Viral Explosion (40%) → Dump (15%) → Rebound (20%)
//! Cykl: 15s wykrycie → 60s akumulacja → 10s hype → 30s short → 20s odbicie

use super::*;
use crate::modules::strategy::{TradingSignal, TradeAction, StrategyType};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Fazy cyklu życia meme
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemePhase {
    Accumulation,    // Cicha akumulacja
    ViralExplosion,  // Viral explosion
    Dump,            // Spadkowa
    Rebound,         // Odbicie
    Dormant,         // Nieaktywny
}

/// Parametry dla MEME VIRUS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeVirusParams {
    pub accumulation_duration: Duration,    // 60 sekund
    pub viral_duration: Duration,           // 10 sekund
    pub dump_duration: Duration,            // 30 sekund
    pub rebound_duration: Duration,         // 20 sekund
    pub viral_threshold: f32,               // 100% wzrost dla viral
    pub dump_threshold: f32,                // 30% spadek dla dump
    pub rebound_threshold: f32,             // 15% wzrost dla rebound
}

impl Default for MemeVirusParams {
    fn default() -> Self {
        Self {
            accumulation_duration: Duration::from_secs(60),
            viral_duration: Duration::from_secs(10),
            dump_duration: Duration::from_secs(30),
            rebound_duration: Duration::from_secs(20),
            viral_threshold: 100.0,
            dump_threshold: 30.0,
            rebound_threshold: 15.0,
        }
    }
}

/// Stan meme tokena
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeTokenState {
    pub token: String,
    pub current_phase: MemePhase,
    pub phase_start_time: chrono::DateTime<chrono::Utc>,
    pub phase_start_price: f64,
    pub current_price: f64,
    pub narrative: String,
    pub viral_score: f32,
    pub social_momentum: f32,
    pub position_size: f64,
}

/// Narracja meme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeNarrative {
    pub theme: String,              // np. "AI", "Gaming", "DeFi"
    pub keywords: Vec<String>,      // Kluczowe słowa
    pub viral_potential: f32,       // 0.0 - 1.0
    pub lifecycle_stage: String,    // "emerging", "trending", "declining"
}

/// Implementacja strategii MEME VIRUS
pub struct MemeVirusStrategy {
    params: MemeVirusParams,
    strategy_params: MemcoinStrategyParams,
    is_active: bool,
    capital: f64,
    meme_states: Arc<RwLock<HashMap<String, MemeTokenState>>>,
    narratives: Arc<RwLock<HashMap<String, MemeNarrative>>>,
    metrics: StrategyMetrics,
}

impl MemeVirusStrategy {
    /// Tworzy nową instancję strategii MEME VIRUS
    pub fn new(capital: f64) -> Self {
        Self {
            params: MemeVirusParams::default(),
            strategy_params: MemcoinStrategyParams::default(),
            is_active: false,
            capital,
            meme_states: Arc::new(RwLock::new(HashMap::new())),
            narratives: Arc::new(RwLock::new(HashMap::new())),
            metrics: StrategyMetrics::default(),
        }
    }

    /// Alokacja kapitału na podstawie fazy
    pub fn meme_virus_allocation(&self, phase: &MemePhase) -> f32 {
        match phase {
            MemePhase::Accumulation => 0.25,    // 25% kapitału
            MemePhase::ViralExplosion => 0.4,   // 40% kapitału
            MemePhase::Dump => 0.15,            // 15% kapitału (short)
            MemePhase::Rebound => 0.2,          // 20% kapitału
            MemePhase::Dormant => 0.0,          // Brak alokacji
        }
    }

    /// Wykrywanie narracji meme
    pub async fn detect_narrative(&self, token: &str, social_signals: &[SocialSignal]) -> Option<MemeNarrative> {
        if social_signals.is_empty() {
            return None;
        }

        // Analiza słów kluczowych z sygnałów społecznościowych
        let mut keyword_frequency: HashMap<String, u32> = HashMap::new();
        
        for signal in social_signals {
            // Symulacja ekstrakcji słów kluczowych (w rzeczywistości NLP)
            let keywords = vec!["AI", "gaming", "defi", "meme", "viral", "moon"];
            for keyword in keywords {
                if signal.source.to_lowercase().contains(&keyword.to_lowercase()) {
                    *keyword_frequency.entry(keyword.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Znajdź dominującą narrację
        if let Some((theme, &count)) = keyword_frequency.iter().max_by_key(|(_, &count)| count) {
            let viral_potential = (count as f32 / social_signals.len() as f32).min(1.0);
            
            info!("🦠 Detected meme narrative for {}: {} (potential: {})", token, theme, viral_potential);
            
            Some(MemeNarrative {
                theme: theme.clone(),
                keywords: keyword_frequency.keys().cloned().collect(),
                viral_potential,
                lifecycle_stage: "emerging".to_string(),
            })
        } else {
            None
        }
    }

    /// Aktualizacja fazy meme
    pub async fn update_meme_phase(&self, token: &str, current_price: f64) -> Option<MemePhase> {
        let mut states = self.meme_states.write().await;
        
        if let Some(state) = states.get_mut(token) {
            let time_in_phase = chrono::Utc::now().signed_duration_since(state.phase_start_time);
            let price_change = ((current_price - state.phase_start_price) / state.phase_start_price) * 100.0;
            
            let new_phase = match state.current_phase {
                MemePhase::Accumulation => {
                    // Przejście do viral explosion po wzroście lub czasie
                    if price_change >= self.params.viral_threshold as f64 ||
                       time_in_phase >= chrono::Duration::from_std(self.params.accumulation_duration).unwrap_or_default() {
                        Some(MemePhase::ViralExplosion)
                    } else {
                        None
                    }
                },
                MemePhase::ViralExplosion => {
                    // Przejście do dump po czasie lub spadku
                    if time_in_phase >= chrono::Duration::from_std(self.params.viral_duration).unwrap_or_default() ||
                       price_change < -(self.params.dump_threshold as f64) {
                        Some(MemePhase::Dump)
                    } else {
                        None
                    }
                },
                MemePhase::Dump => {
                    // Przejście do rebound po czasie lub wzroście
                    if time_in_phase >= chrono::Duration::from_std(self.params.dump_duration).unwrap_or_default() ||
                       price_change >= self.params.rebound_threshold as f64 {
                        Some(MemePhase::Rebound)
                    } else {
                        None
                    }
                },
                MemePhase::Rebound => {
                    // Powrót do dormant po czasie
                    if time_in_phase >= chrono::Duration::from_std(self.params.rebound_duration).unwrap_or_default() {
                        Some(MemePhase::Dormant)
                    } else {
                        None
                    }
                },
                MemePhase::Dormant => None, // Pozostaje dormant
            };

            if let Some(phase) = new_phase {
                info!("🦠 Phase transition for {}: {:?} → {:?}", token, state.current_phase, phase);
                
                state.current_phase = phase.clone();
                state.phase_start_time = chrono::Utc::now();
                state.phase_start_price = current_price;
                state.current_price = current_price;
                
                return Some(phase);
            } else {
                // Aktualizuj tylko cenę
                state.current_price = current_price;
            }
        }

        None
    }

    /// Inicjalizacja nowego meme tokena
    pub async fn initialize_meme_token(&self, token: &str, price: f64, narrative: MemeNarrative) {
        let mut states = self.meme_states.write().await;
        let mut narratives = self.narratives.write().await;
        
        let state = MemeTokenState {
            token: token.to_string(),
            current_phase: MemePhase::Accumulation,
            phase_start_time: chrono::Utc::now(),
            phase_start_price: price,
            current_price: price,
            narrative: narrative.theme.clone(),
            viral_score: narrative.viral_potential,
            social_momentum: 0.0,
            position_size: 0.0,
        };

        states.insert(token.to_string(), state);
        narratives.insert(token.to_string(), narrative);
        
        info!("🦠 Initialized meme token {}: phase=Accumulation, price={}", token, price);
    }

    /// Generowanie sygnału na podstawie fazy
    pub async fn generate_phase_signal(&self, token: &str, phase: &MemePhase) -> Option<TradingSignal> {
        let states = self.meme_states.read().await;
        let state = states.get(token)?;
        
        let allocation = self.meme_virus_allocation(phase);
        if allocation == 0.0 {
            return None;
        }

        let position_size = (self.capital * allocation as f64).min(self.strategy_params.max_position_size);
        
        let (action, confidence) = match phase {
            MemePhase::Accumulation => (TradeAction::Buy, 0.7),
            MemePhase::ViralExplosion => (TradeAction::Buy, 0.9), // Najwyższa pewność
            MemePhase::Dump => (TradeAction::Sell, 0.8),         // Short
            MemePhase::Rebound => (TradeAction::Buy, 0.6),
            MemePhase::Dormant => return None,
        };

        info!("🦠 MEME VIRUS signal for {}: phase={:?}, action={:?}, size={} SOL", 
              token, phase, action, position_size);

        Some(TradingSignal {
            signal_id: uuid::Uuid::new_v4().to_string(),
            symbol: token.to_string(),
            action,
            quantity: position_size,
            target_price: 0.0, // Market order
            price: None, // Market order
            confidence,
            timestamp: chrono::Utc::now(),
            strategy_type: StrategyType::MemeVirus,
            urgency: None,
            metadata: None,
        })
    }

    /// Kalkulacja viral score
    pub async fn calculate_viral_score(&self, token: &str, social_signals: &[SocialSignal]) -> f32 {
        if social_signals.is_empty() {
            return 0.0;
        }

        // Intensywność sygnałów społecznościowych
        let avg_intensity: f32 = social_signals.iter().map(|s| s.intensity).sum::<f32>() / social_signals.len() as f32;
        
        // Sentyment
        let avg_sentiment: f32 = social_signals.iter().map(|s| s.sentiment).sum::<f32>() / social_signals.len() as f32;
        
        // Liczba wzmianek
        let mention_score = (social_signals.len() as f32 / 100.0).min(1.0); // Normalizacja do 1.0
        
        // Kombinacja wszystkich czynników
        let viral_score = (avg_intensity / 100.0) * 0.4 + 
                         ((avg_sentiment + 1.0) / 2.0) * 0.3 + 
                         mention_score * 0.3;

        debug!("Viral score for {}: intensity={}, sentiment={}, mentions={}, final={}", 
               token, avg_intensity, avg_sentiment, social_signals.len(), viral_score);
        
        viral_score.min(1.0)
    }

    /// Sprawdzenie czy token jest w aktywnej fazie
    pub async fn is_active_phase(&self, token: &str) -> bool {
        let states = self.meme_states.read().await;
        
        if let Some(state) = states.get(token) {
            !matches!(state.current_phase, MemePhase::Dormant)
        } else {
            false
        }
    }

    /// Cleanup nieaktywnych tokenów
    pub async fn cleanup_dormant_tokens(&self) {
        let mut states = self.meme_states.write().await;
        let mut narratives = self.narratives.write().await;
        
        let now = chrono::Utc::now();
        let dormant_threshold = chrono::Duration::hours(24); // 24 godziny nieaktywności
        
        states.retain(|token, state| {
            let inactive_time = now.signed_duration_since(state.phase_start_time);
            let should_keep = !(matches!(state.current_phase, MemePhase::Dormant) && inactive_time > dormant_threshold);
            
            if !should_keep {
                info!("🧹 Cleaning up dormant meme token: {}", token);
                narratives.remove(token);
            }
            
            should_keep
        });
    }
}

#[async_trait]
impl MemcoinStrategy for MemeVirusStrategy {
    fn name(&self) -> &str {
        "MEME VIRUS"
    }

    fn strategy_type(&self) -> StrategyType {
        StrategyType::MemeVirus
    }

    async fn process_signal(&self, signal: &(dyn std::any::Any + Send + Sync)) -> Result<Option<TradingSignal>> {
        if !self.is_active {
            return Ok(None);
        }

        // Cleanup nieaktywnych tokenów
        self.cleanup_dormant_tokens().await;

        // Próba konwersji sygnału na SocialSignal (dla wykrywania narracji)
        if let Some(social_signal) = signal.downcast_ref::<SocialSignal>() {
            debug!("Processing meme virus signal for {}", social_signal.token);

            // Sprawdź czy token już istnieje
            let is_tracked = {
                let states = self.meme_states.read().await;
                states.contains_key(&social_signal.token)
            };

            if !is_tracked {
                // Wykryj narrację dla nowego tokena
                let signals = vec![social_signal.clone()];
                if let Some(narrative) = self.detect_narrative(&social_signal.token, &signals).await {
                    // Symulacja ceny (w rzeczywistości z market data)
                    let simulated_price = 100.0;
                    self.initialize_meme_token(&social_signal.token, simulated_price, narrative).await;
                }
            }

            // Aktualizuj fazę i generuj sygnał
            let simulated_current_price = 120.0; // Symulacja wzrostu ceny
            if let Some(new_phase) = self.update_meme_phase(&social_signal.token, simulated_current_price).await {
                if let Some(trading_signal) = self.generate_phase_signal(&social_signal.token, &new_phase).await {
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
        info!("🦠 Activating MEME VIRUS strategy");
        self.is_active = true;
        Ok(())
    }

    async fn deactivate(&mut self) -> Result<()> {
        info!("🛑 Deactivating MEME VIRUS strategy");
        self.is_active = false;
        Ok(())
    }

    async fn update_params(&mut self, params: MemcoinStrategyParams) -> Result<()> {
        info!("🔧 Updating MEME VIRUS parameters");
        self.strategy_params = params;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capital_allocation() {
        let strategy = MemeVirusStrategy::new(1000.0);
        
        assert_eq!(strategy.meme_virus_allocation(&MemePhase::Accumulation), 0.25);
        assert_eq!(strategy.meme_virus_allocation(&MemePhase::ViralExplosion), 0.4);
        assert_eq!(strategy.meme_virus_allocation(&MemePhase::Dump), 0.15);
        assert_eq!(strategy.meme_virus_allocation(&MemePhase::Rebound), 0.2);
        assert_eq!(strategy.meme_virus_allocation(&MemePhase::Dormant), 0.0);
    }

    #[tokio::test]
    async fn test_narrative_detection() {
        let strategy = MemeVirusStrategy::new(1000.0);
        
        let signals = vec![
            SocialSignal {
                token: "test_token".to_string(),
                intensity: 90.0,
                sentiment: 0.8,
                mentions_count: 10,
                source: "AI gaming revolution".to_string(),
                timestamp: chrono::Utc::now(),
            }
        ];

        let narrative = strategy.detect_narrative("test_token", &signals).await;
        assert!(narrative.is_some());
    }
}
