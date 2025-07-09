//! PHEROMONE BUS - Emergentna Komunikacja Roju
//! 
//! Asynchroniczny system komunikacji inspirowany feromonami
//! Zastępuje SharedKnowledge aktywnym strumieniem sygnałów

use anyhow::{Result, anyhow};
use redis::{Client, Connection, Commands, RedisResult, FromRedisValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Typ sygnału feromonowego
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PheromoneSignal {
    /// Sygnał kupna z siłą
    BuySignal { strength: f64, confidence: f64 },
    
    /// Sygnał sprzedaży z siłą
    SellSignal { strength: f64, confidence: f64 },
    
    /// Ostrzeżenie o ryzyku
    RiskWarning { risk_level: f64, reason: String },
    
    /// Wykrycie okazji
    Opportunity { potential: f64, timeframe: u64 },
    
    /// Sygnał neutralny/wyczekiwania
    HoldSignal { patience_level: f64 },
    
    /// Sygnał paniki/ucieczki
    PanicSignal { urgency: f64, exit_reason: String },
    
    /// Sygnał konsolidacji pozycji
    ConsolidateSignal { target_exposure: f64 },
    
    /// Sygnał zwiększenia aktywności
    ActivityBoost { multiplier: f64, duration: u64 },
}

/// Feromon - jednostka komunikacji w roju
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pheromone {
    /// Unikalny identyfikator feromonu
    pub id: String,
    
    /// ID agenta, który zostawił feromon
    pub agent_id: String,
    
    /// Typ sygnału
    pub signal: PheromoneSignal,
    
    /// Intensywność feromonu (0.0 - 1.0)
    pub intensity: f64,
    
    /// Czas utworzenia (Unix timestamp)
    pub timestamp: u64,
    
    /// Czas wygaśnięcia (Unix timestamp)
    pub expires_at: u64,
    
    /// Kontekst rynkowy
    pub market_context: MarketContext,
    
    /// Metadane dodatkowe
    pub metadata: HashMap<String, String>,
}

/// Kontekst rynkowy dla feromonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContext {
    /// Symbol/token którego dotyczy sygnał
    pub symbol: Option<String>,
    
    /// Cena w momencie sygnału
    pub price: Option<f64>,
    
    /// Wolumen
    pub volume: Option<f64>,
    
    /// Volatility
    pub volatility: Option<f64>,
    
    /// Trend direction (-1.0 to 1.0)
    pub trend: Option<f64>,
}

/// Konfiguracja PheromoneB us
#[derive(Debug, Clone)]
pub struct PheromoneConfig {
    /// Redis connection string
    pub redis_url: String,
    
    /// Nazwa strumienia Redis
    pub stream_name: String,
    
    /// Maksymalny czas życia feromonu (sekundy)
    pub default_ttl: u64,
    
    /// Maksymalna liczba feromonów w strumieniu
    pub max_stream_length: usize,
    
    /// Interwał czyszczenia wygasłych feromonów
    pub cleanup_interval: Duration,
    
    /// Próg intensywności dla filtrowania słabych sygnałów
    pub intensity_threshold: f64,
}

impl Default for PheromoneConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            stream_name: "overmind:pheromones".to_string(),
            default_ttl: 300, // 5 minut
            max_stream_length: 10000,
            cleanup_interval: Duration::from_secs(60),
            intensity_threshold: 0.1,
        }
    }
}

/// PheromoneBus - główny system komunikacji feromonowej
pub struct PheromoneBus {
    /// Konfiguracja
    config: PheromoneConfig,
    
    /// Połączenie Redis
    redis_client: Client,
    
    /// ID tego agenta
    agent_id: String,
    
    /// Cache lokalny feromonów
    local_cache: Arc<RwLock<HashMap<String, Pheromone>>>,
    
    /// Kanał dla nowych feromonów
    pheromone_sender: mpsc::UnboundedSender<Pheromone>,
    
    /// Metryki systemu
    metrics: Arc<RwLock<PheromoneMetrics>>,
}

/// Metryki systemu feromonowego
#[derive(Debug, Default, Clone)]
pub struct PheromoneMetrics {
    pub total_deposited: u64,
    pub total_sensed: u64,
    pub active_pheromones: u64,
    pub expired_pheromones: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl PheromoneBus {
    /// Utwórz nowy PheromoneBus
    pub fn new(config: PheromoneConfig, agent_id: String) -> Result<(Self, mpsc::UnboundedReceiver<Pheromone>)> {
        let redis_client = Client::open(config.redis_url.as_str())
            .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;
        
        let (pheromone_sender, pheromone_receiver) = mpsc::unbounded_channel();
        
        let bus = Self {
            config,
            redis_client,
            agent_id,
            local_cache: Arc::new(RwLock::new(HashMap::new())),
            pheromone_sender,
            metrics: Arc::new(RwLock::new(PheromoneMetrics::default())),
        };
        
        Ok((bus, pheromone_receiver))
    }
    
    /// Zostaw feromon w systemie (deposit_pheromone)
    pub async fn deposit_pheromone(
        &self,
        signal: PheromoneSignal,
        intensity: f64,
        market_context: MarketContext,
        ttl_override: Option<u64>,
    ) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let ttl = ttl_override.unwrap_or(self.config.default_ttl);
        
        let pheromone = Pheromone {
            id: Uuid::new_v4().to_string(),
            agent_id: self.agent_id.clone(),
            signal,
            intensity: intensity.clamp(0.0, 1.0),
            timestamp: now,
            expires_at: now + ttl,
            market_context,
            metadata: HashMap::new(),
        };
        
        // Zapisz do Redis Stream
        self.write_to_redis_stream(&pheromone).await?;
        
        // Dodaj do lokalnego cache
        {
            let mut cache = self.local_cache.write().await;
            cache.insert(pheromone.id.clone(), pheromone.clone());
        }
        
        // Wyślij przez kanał
        if let Err(e) = self.pheromone_sender.send(pheromone.clone()) {
            warn!("Failed to send pheromone through channel: {}", e);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_deposited += 1;
            metrics.active_pheromones += 1;
        }
        
        debug!("Deposited pheromone: {} with intensity {:.3}", pheromone.id, intensity);
        Ok(pheromone.id)
    }
    
    /// Wyczuj feromony w okolicy (sense_pheromones)
    pub async fn sense_pheromones(
        &self,
        max_age_seconds: Option<u64>,
        min_intensity: Option<f64>,
        signal_filter: Option<Vec<PheromoneSignal>>,
    ) -> Result<Vec<Pheromone>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let max_age = max_age_seconds.unwrap_or(300); // 5 minut domyślnie
        let min_intensity = min_intensity.unwrap_or(self.config.intensity_threshold);
        
        // Najpierw sprawdź lokalny cache
        let mut pheromones = Vec::new();
        {
            let cache = self.local_cache.read().await;
            for pheromone in cache.values() {
                if self.matches_criteria(pheromone, now, max_age, min_intensity, &signal_filter) {
                    pheromones.push(pheromone.clone());
                }
            }
        }
        
        // Jeśli cache nie ma wystarczająco danych, czytaj z Redis
        if pheromones.len() < 10 {
            let redis_pheromones = self.read_from_redis_stream(max_age).await?;
            for pheromone in redis_pheromones {
                if self.matches_criteria(&pheromone, now, max_age, min_intensity, &signal_filter) {
                    pheromones.push(pheromone);
                }
            }
        }
        
        // Sortuj według intensywności i czasu
        pheromones.sort_by(|a, b| {
            let intensity_cmp = b.intensity.partial_cmp(&a.intensity).unwrap_or(std::cmp::Ordering::Equal);
            if intensity_cmp == std::cmp::Ordering::Equal {
                b.timestamp.cmp(&a.timestamp)
            } else {
                intensity_cmp
            }
        });
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_sensed += pheromones.len() as u64;
        }
        
        debug!("Sensed {} pheromones matching criteria", pheromones.len());
        Ok(pheromones)
    }
    
    /// Sprawdź czy feromon spełnia kryteria
    fn matches_criteria(
        &self,
        pheromone: &Pheromone,
        now: u64,
        max_age: u64,
        min_intensity: f64,
        signal_filter: &Option<Vec<PheromoneSignal>>,
    ) -> bool {
        // Sprawdź wiek
        if now - pheromone.timestamp > max_age {
            return false;
        }
        
        // Sprawdź czy nie wygasł
        if pheromone.expires_at < now {
            return false;
        }
        
        // Sprawdź intensywność
        if pheromone.intensity < min_intensity {
            return false;
        }
        
        // Sprawdź filtr sygnałów
        if let Some(filters) = signal_filter {
            if !filters.contains(&pheromone.signal) {
                return false;
            }
        }
        
        // Nie czytaj własnych feromonów (opcjonalnie)
        if pheromone.agent_id == self.agent_id {
            return false;
        }
        
        true
    }
    
    /// Zapisz feromon do Redis Stream
    async fn write_to_redis_stream(&self, pheromone: &Pheromone) -> Result<()> {
        let mut conn = self.redis_client.get_connection()
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;
        
        let pheromone_json = serde_json::to_string(pheromone)
            .map_err(|e| anyhow!("Serialization error: {}", e))?;
        
        let _: RedisResult<String> = conn.xadd(
            &self.config.stream_name,
            "*",
            &[("data", pheromone_json)]
        );
        
        // Ogranicz długość strumienia
        let _: RedisResult<i32> = conn.xtrim(
            &self.config.stream_name,
            redis::streams::StreamMaxlen::Approx(self.config.max_stream_length)
        );
        
        Ok(())
    }
    
    /// Czytaj feromony z Redis Stream
    async fn read_from_redis_stream(&self, max_age_seconds: u64) -> Result<Vec<Pheromone>> {
        let mut conn = self.redis_client.get_connection()
            .map_err(|e| anyhow!("Redis connection error: {}", e))?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let min_timestamp = now - max_age_seconds;
        
        // Czytaj ostatnie wpisy ze strumienia
        let results: RedisResult<redis::streams::StreamReadReply> = conn.xread_options(
            &[&self.config.stream_name],
            &["0"], // Od początku strumienia
            &redis::streams::StreamReadOptions::default().count(1000)
        );
        
        let mut pheromones = Vec::new();
        
        if let Ok(reply) = results {
            for stream_key in reply.keys {
                for stream_id in stream_key.ids {
                    for (field, value) in stream_id.map {
                        if field == "data" {
                            if let Ok(pheromone_str) = String::from_redis_value(&value) {
                                if let Ok(pheromone) = serde_json::from_str::<Pheromone>(&pheromone_str) {
                                    if pheromone.timestamp >= min_timestamp {
                                        pheromones.push(pheromone);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(pheromones)
    }
    
    /// Wyczyść wygasłe feromony
    pub async fn cleanup_expired_pheromones(&self) -> Result<u64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut removed_count = 0;
        
        // Wyczyść lokalny cache
        {
            let mut cache = self.local_cache.write().await;
            let initial_size = cache.len();
            cache.retain(|_, pheromone| pheromone.expires_at > now);
            removed_count = initial_size - cache.len();
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.expired_pheromones += removed_count as u64;
            metrics.active_pheromones = metrics.active_pheromones.saturating_sub(removed_count as u64);
        }
        
        debug!("Cleaned up {} expired pheromones", removed_count);
        Ok(removed_count as u64)
    }
    
    /// Pobierz metryki systemu
    pub async fn get_metrics(&self) -> PheromoneMetrics {
        (*self.metrics.read().await).clone()
    }
    
    /// Pobierz ID agenta
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
}

/// Inicjalizuj PheromoneBus z domyślną konfiguracją
pub async fn init_pheromone_bus(agent_id: String) -> Result<(PheromoneBus, mpsc::UnboundedReceiver<Pheromone>)> {
    let config = PheromoneConfig::default();
    PheromoneBus::new(config, agent_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pheromone_creation() {
        let signal = PheromoneSignal::BuySignal { strength: 0.8, confidence: 0.9 };
        let context = MarketContext {
            symbol: Some("SOL".to_string()),
            price: Some(100.0),
            volume: Some(1000.0),
            volatility: Some(0.05),
            trend: Some(0.3),
        };
        
        // Test będzie działał tylko z działającym Redis
        // W rzeczywistym środowisku testowym
    }
    
    #[test]
    fn test_signal_matching() {
        let buy_signal = PheromoneSignal::BuySignal { strength: 0.8, confidence: 0.9 };
        let sell_signal = PheromoneSignal::SellSignal { strength: 0.7, confidence: 0.8 };
        
        assert_ne!(buy_signal, sell_signal);
    }
}
