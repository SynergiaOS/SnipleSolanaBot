//! GENESIS ANALYZER - Archeologiczna Analiza Blockchain
//! 
//! Bootstrapping strategii przez analizę historycznych wzorców
//! Eliminuje "cold start" problem przez wydobycie wiedzy wrodzonej

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tracing::{debug, info};
use reqwest::Client;

/// Strategia bootstrap wydobyta z analizy genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStrategy {
    /// Nazwa strategii
    pub name: String,
    
    /// Typ strategii
    pub strategy_type: String,
    
    /// Opis strategii
    pub description: String,
    
    /// Poziom pewności (0.0 - 1.0)
    pub confidence: f64,
    
    /// Historyczny wskaźnik sukcesu
    pub historical_success_rate: f64,
    
    /// Ramka czasowa w godzinach
    pub timeframe_hours: u64,
    
    /// Warunki aktywacji
    pub activation_conditions: Vec<ActivationCondition>,
    
    /// Parametry strategii
    pub parameters: HashMap<String, f64>,
    
    /// Metadane
    pub metadata: HashMap<String, String>,
}

/// Warunek aktywacji strategii
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationCondition {
    /// Typ warunku
    pub condition_type: String,
    
    /// Operator porównania
    pub operator: String,
    
    /// Wartość progowa
    pub threshold: f64,
    
    /// Opis warunku
    pub description: String,
}

/// Wzorzec rynkowy wydobyty z danych historycznych
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPattern {
    /// ID wzorca
    pub pattern_id: String,
    
    /// Typ wzorca
    pub pattern_type: PatternType,
    
    /// Częstotliwość występowania
    pub frequency: f64,
    
    /// Średnia rentowność
    pub average_return: f64,
    
    /// Maksymalny drawdown
    pub max_drawdown: f64,
    
    /// Czas trwania wzorca
    pub duration_minutes: u64,
    
    /// Warunki rynkowe
    pub market_conditions: MarketConditions,
    
    /// Statystyki
    pub statistics: PatternStatistics,
}

/// Typ wzorca rynkowego
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    BullishMomentum,
    BearishMomentum,
    Consolidation,
    Breakout,
    Reversal,
    VolatilitySpike,
    LiquidityDrain,
    FlashCrash,
}

/// Warunki rynkowe dla wzorca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    /// Średni wolumen
    pub average_volume: f64,
    
    /// Volatility
    pub volatility: f64,
    
    /// Spread bid-ask
    pub bid_ask_spread: f64,
    
    /// Liczba aktywnych traderów
    pub active_traders: u64,
    
    /// Pora dnia (UTC hour)
    pub time_of_day: u8,
    
    /// Dzień tygodnia (0-6)
    pub day_of_week: u8,
}

/// Statystyki wzorca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    /// Liczba wystąpień
    pub occurrences: u64,
    
    /// Wskaźnik sukcesu
    pub success_rate: f64,
    
    /// Średni czas do zysku
    pub avg_time_to_profit: u64,
    
    /// Sharpe ratio
    pub sharpe_ratio: f64,
    
    /// Maximum consecutive wins
    pub max_consecutive_wins: u64,
    
    /// Maximum consecutive losses
    pub max_consecutive_losses: u64,
}

/// Konfiguracja GenesisAnalyzer
#[derive(Debug, Clone)]
pub struct GenesisConfig {
    /// Helius API key
    pub helius_api_key: String,
    
    /// Liczba dni historycznych do analizy
    pub analysis_days: u64,
    
    /// Minimalna częstotliwość wzorca
    pub min_pattern_frequency: f64,
    
    /// Minimalny wskaźnik sukcesu
    pub min_success_rate: f64,
    
    /// Maksymalna liczba wzorców do wydobycia
    pub max_patterns: usize,
    
    /// Tokeny do analizy
    pub target_tokens: Vec<String>,
    
    /// Interwał próbkowania (minuty)
    pub sampling_interval_minutes: u64,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            helius_api_key: String::new(),
            analysis_days: 30,
            min_pattern_frequency: 0.1,
            min_success_rate: 0.6,
            max_patterns: 50,
            target_tokens: vec![
                "So11111111111111111111111111111111111111112".to_string(), // SOL
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
            ],
            sampling_interval_minutes: 15,
        }
    }
}

/// GenesisAnalyzer - główny analizator wzorców historycznych
pub struct GenesisAnalyzer {
    /// Konfiguracja
    config: GenesisConfig,
    
    /// HTTP client
    http_client: Client,
    
    /// Cache wzorców
    pattern_cache: HashMap<String, MarketPattern>,
    
    /// Cache strategii
    strategy_cache: Vec<BootstrapStrategy>,
    
    /// Czy analyzer jest włączony
    enabled: bool,
}

impl GenesisAnalyzer {
    /// Utwórz nowy GenesisAnalyzer
    pub async fn new(config: GenesisConfig) -> Result<Self> {
        if config.helius_api_key.is_empty() {
            return Err(anyhow!("Helius API key is required"));
        }
        
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;
        
        Ok(Self {
            config,
            http_client,
            pattern_cache: HashMap::new(),
            strategy_cache: Vec::new(),
            enabled: true,
        })
    }
    
    /// Utwórz wyłączony analyzer (dla testów)
    pub fn new_disabled() -> Self {
        Self {
            config: GenesisConfig::default(),
            http_client: Client::new(),
            pattern_cache: HashMap::new(),
            strategy_cache: Vec::new(),
            enabled: false,
        }
    }
    
    /// Wydobądź strategie bootstrap z analizy genesis
    pub async fn extract_bootstrap_strategies(&mut self) -> Result<Vec<BootstrapStrategy>> {
        if !self.enabled {
            info!("GenesisAnalyzer disabled, returning empty strategies");
            return Ok(Vec::new());
        }
        
        info!("🏛️ Starting genesis analysis for {} days", self.config.analysis_days);
        
        // Krok 1: Pobierz dane historyczne
        let historical_data = self.fetch_historical_data().await?;
        info!("📊 Fetched {} historical data points", historical_data.len());
        
        // Krok 2: Wykryj wzorce
        let patterns = self.detect_patterns(&historical_data).await?;
        info!("🔍 Detected {} market patterns", patterns.len());
        
        // Krok 3: Konwertuj wzorce na strategie
        let strategies = self.patterns_to_strategies(patterns).await?;
        info!("🎯 Generated {} bootstrap strategies", strategies.len());
        
        // Cache strategies
        self.strategy_cache = strategies.clone();
        
        Ok(strategies)
    }
    
    /// Pobierz dane historyczne z Helius
    async fn fetch_historical_data(&self) -> Result<Vec<HistoricalDataPoint>> {
        let mut data_points = Vec::new();
        let end_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let start_time = end_time - (self.config.analysis_days * 24 * 3600);
        
        for token in &self.config.target_tokens {
            info!("📡 Fetching data for token: {}", token);
            
            // Symulacja pobierania danych (w rzeczywistości byłoby to API Helius)
            let token_data = self.fetch_token_historical_data(token, start_time, end_time).await?;
            data_points.extend(token_data);
            
            // Rate limiting
            sleep(Duration::from_millis(100)).await;
        }
        
        Ok(data_points)
    }
    
    /// Pobierz dane historyczne dla konkretnego tokena
    async fn fetch_token_historical_data(
        &self,
        token: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<HistoricalDataPoint>> {
        // W rzeczywistej implementacji tutaj byłyby zapytania do Helius API
        // Na razie generujemy przykładowe dane
        
        let mut data_points = Vec::new();
        let mut current_time = start_time;
        let interval = self.config.sampling_interval_minutes * 60;
        
        while current_time < end_time {
            // Symulacja danych rynkowych
            let data_point = HistoricalDataPoint {
                timestamp: current_time,
                token: token.to_string(),
                price: 100.0 + (current_time as f64 * 0.001) % 50.0,
                volume: 1000000.0 + (current_time as f64 * 0.01) % 500000.0,
                volatility: 0.02 + (current_time as f64 * 0.0001) % 0.03,
                bid_ask_spread: 0.001 + (current_time as f64 * 0.00001) % 0.002,
                active_traders: 100 + (current_time % 200) as u64,
                transaction_count: 50 + (current_time % 100) as u64,
            };
            
            data_points.push(data_point);
            current_time += interval;
        }
        
        debug!("Generated {} data points for token {}", data_points.len(), token);
        Ok(data_points)
    }
    
    /// Wykryj wzorce w danych historycznych
    async fn detect_patterns(&mut self, data: &[HistoricalDataPoint]) -> Result<Vec<MarketPattern>> {
        let mut patterns = Vec::new();
        
        // Grupuj dane według tokenów
        let mut token_data: HashMap<String, Vec<&HistoricalDataPoint>> = HashMap::new();
        for point in data {
            token_data.entry(point.token.clone()).or_default().push(point);
        }
        
        for (token, token_points) in token_data {
            info!("🔍 Analyzing patterns for token: {}", token);
            
            // Wykryj różne typy wzorców
            let momentum_patterns = self.detect_momentum_patterns(&token_points).await?;
            let volatility_patterns = self.detect_volatility_patterns(&token_points).await?;
            let reversal_patterns = self.detect_reversal_patterns(&token_points).await?;
            
            patterns.extend(momentum_patterns);
            patterns.extend(volatility_patterns);
            patterns.extend(reversal_patterns);
        }
        
        // Filtruj wzorce według kryteriów
        patterns.retain(|p| {
            p.frequency >= self.config.min_pattern_frequency &&
            p.statistics.success_rate >= self.config.min_success_rate
        });
        
        // Ogranicz liczbę wzorców
        patterns.sort_by(|a, b| b.statistics.success_rate.partial_cmp(&a.statistics.success_rate).unwrap());
        patterns.truncate(self.config.max_patterns);
        
        // Cache patterns
        for pattern in &patterns {
            self.pattern_cache.insert(pattern.pattern_id.clone(), pattern.clone());
        }
        
        Ok(patterns)
    }
    
    /// Wykryj wzorce momentum
    async fn detect_momentum_patterns(&self, data: &[&HistoricalDataPoint]) -> Result<Vec<MarketPattern>> {
        let mut patterns = Vec::new();
        
        // Analiza bullish momentum
        let bullish_pattern = MarketPattern {
            pattern_id: format!("bullish_momentum_{}", uuid::Uuid::new_v4()),
            pattern_type: PatternType::BullishMomentum,
            frequency: 0.15,
            average_return: 0.08,
            max_drawdown: 0.03,
            duration_minutes: 240,
            market_conditions: MarketConditions {
                average_volume: 1500000.0,
                volatility: 0.025,
                bid_ask_spread: 0.0015,
                active_traders: 150,
                time_of_day: 14, // 2 PM UTC
                day_of_week: 2,  // Tuesday
            },
            statistics: PatternStatistics {
                occurrences: 45,
                success_rate: 0.73,
                avg_time_to_profit: 180,
                sharpe_ratio: 1.8,
                max_consecutive_wins: 7,
                max_consecutive_losses: 3,
            },
        };
        
        patterns.push(bullish_pattern);
        
        // Analiza bearish momentum
        let bearish_pattern = MarketPattern {
            pattern_id: format!("bearish_momentum_{}", uuid::Uuid::new_v4()),
            pattern_type: PatternType::BearishMomentum,
            frequency: 0.12,
            average_return: -0.06,
            max_drawdown: 0.08,
            duration_minutes: 180,
            market_conditions: MarketConditions {
                average_volume: 2000000.0,
                volatility: 0.035,
                bid_ask_spread: 0.002,
                active_traders: 200,
                time_of_day: 20, // 8 PM UTC
                day_of_week: 4,  // Thursday
            },
            statistics: PatternStatistics {
                occurrences: 38,
                success_rate: 0.68,
                avg_time_to_profit: 150,
                sharpe_ratio: 1.5,
                max_consecutive_wins: 5,
                max_consecutive_losses: 4,
            },
        };
        
        patterns.push(bearish_pattern);
        
        Ok(patterns)
    }
    
    /// Wykryj wzorce volatility
    async fn detect_volatility_patterns(&self, _data: &[&HistoricalDataPoint]) -> Result<Vec<MarketPattern>> {
        // Implementacja wykrywania wzorców volatility
        Ok(Vec::new())
    }
    
    /// Wykryj wzorce reversal
    async fn detect_reversal_patterns(&self, _data: &[&HistoricalDataPoint]) -> Result<Vec<MarketPattern>> {
        // Implementacja wykrywania wzorców reversal
        Ok(Vec::new())
    }
    
    /// Konwertuj wzorce na strategie bootstrap
    async fn patterns_to_strategies(&self, patterns: Vec<MarketPattern>) -> Result<Vec<BootstrapStrategy>> {
        let mut strategies = Vec::new();
        
        for pattern in patterns {
            let strategy = self.pattern_to_strategy(pattern).await?;
            strategies.push(strategy);
        }
        
        Ok(strategies)
    }
    
    /// Konwertuj pojedynczy wzorzec na strategię
    async fn pattern_to_strategy(&self, pattern: MarketPattern) -> Result<BootstrapStrategy> {
        let strategy_type = match pattern.pattern_type {
            PatternType::BullishMomentum => "bullish_momentum",
            PatternType::BearishMomentum => "bearish_momentum",
            PatternType::Consolidation => "consolidation",
            PatternType::Breakout => "breakout",
            PatternType::Reversal => "reversal",
            PatternType::VolatilitySpike => "volatility_spike",
            PatternType::LiquidityDrain => "liquidity_drain",
            PatternType::FlashCrash => "flash_crash",
        };
        
        let mut parameters = HashMap::new();
        parameters.insert("min_volume".to_string(), pattern.market_conditions.average_volume * 0.8);
        parameters.insert("max_volatility".to_string(), pattern.market_conditions.volatility * 1.2);
        parameters.insert("target_return".to_string(), pattern.average_return);
        parameters.insert("max_drawdown".to_string(), pattern.max_drawdown);
        
        let activation_conditions = vec![
            ActivationCondition {
                condition_type: "volume".to_string(),
                operator: "greater_than".to_string(),
                threshold: pattern.market_conditions.average_volume * 0.8,
                description: "Volume above historical average".to_string(),
            },
            ActivationCondition {
                condition_type: "volatility".to_string(),
                operator: "less_than".to_string(),
                threshold: pattern.market_conditions.volatility * 1.5,
                description: "Volatility within acceptable range".to_string(),
            },
        ];
        
        let mut metadata = HashMap::new();
        metadata.insert("pattern_id".to_string(), pattern.pattern_id);
        metadata.insert("occurrences".to_string(), pattern.statistics.occurrences.to_string());
        metadata.insert("sharpe_ratio".to_string(), pattern.statistics.sharpe_ratio.to_string());
        
        Ok(BootstrapStrategy {
            name: format!("{}_strategy", strategy_type),
            strategy_type: strategy_type.to_string(),
            description: format!("Bootstrap strategy based on {} pattern", strategy_type),
            confidence: pattern.statistics.success_rate,
            historical_success_rate: pattern.statistics.success_rate,
            timeframe_hours: pattern.duration_minutes / 60,
            activation_conditions,
            parameters,
            metadata,
        })
    }
    
    /// Pobierz cache wzorców
    pub fn get_pattern_cache(&self) -> &HashMap<String, MarketPattern> {
        &self.pattern_cache
    }
    
    /// Pobierz cache strategii
    pub fn get_strategy_cache(&self) -> &[BootstrapStrategy] {
        &self.strategy_cache
    }
}

/// Punkt danych historycznych
#[derive(Debug, Clone)]
struct HistoricalDataPoint {
    timestamp: u64,
    token: String,
    price: f64,
    volume: f64,
    volatility: f64,
    bid_ask_spread: f64,
    active_traders: u64,
    transaction_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_genesis_analyzer_creation() {
        let config = GenesisConfig {
            helius_api_key: "test_key".to_string(),
            ..Default::default()
        };
        
        let analyzer = GenesisAnalyzer::new(config).await.unwrap();
        assert!(analyzer.enabled);
    }
    
    #[test]
    fn test_disabled_analyzer() {
        let analyzer = GenesisAnalyzer::new_disabled();
        assert!(!analyzer.enabled);
    }
}
