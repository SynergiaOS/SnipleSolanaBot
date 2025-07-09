//! Agents - Implementacja agentów zgodnie z filozofią Hotza
//! 
//! Task 2.3: SentimentAgent z nanosekundowym profilowaniem

use crate::{CortexResult, CortexError, CortexCore, PerformanceMetrics};
use crate::dispatcher::{AiTaskBuilder, SentimentFlags, FALLBACK_RESPONSE};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent analizy sentymentu z filozofią Hotza
pub struct SentimentAgent {
    /// Referencja do Cortex Core
    cortex: Arc<CortexCore>,
    /// Metryki wydajności agenta
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Cache dla częstych zapytań (statyczny)
    cache: SentimentCache,
}

/// Cache sentymentu z ograniczoną pamięcią
struct SentimentCache {
    /// Statyczna mapa hash→sentiment (256 wpisów)
    entries: [(u64, f32); 256],
    /// Pozycja następnego wpisu (round-robin)
    next_position: usize,
}

impl SentimentCache {
    fn new() -> Self {
        Self {
            entries: [(0, 0.0); 256],
            next_position: 0,
        }
    }

    /// Sprawdzenie cache (O(1) lookup)
    fn get(&self, hash: u64) -> Option<f32> {
        for (cached_hash, sentiment) in &self.entries {
            if *cached_hash == hash && *cached_hash != 0 {
                return Some(*sentiment);
            }
        }
        None
    }

    /// Dodanie do cache (O(1) insert)
    fn insert(&mut self, hash: u64, sentiment: f32) {
        self.entries[self.next_position] = (hash, sentiment);
        self.next_position = (self.next_position + 1) % 256;
    }

    /// Hash tekstu (FNV-1a variant)
    fn hash_text(text: &str) -> u64 {
        let mut hash = 14695981039346656037u64;
        for byte in text.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    }
}

impl SentimentAgent {
    /// Utworzenie nowego SentimentAgent
    pub fn new(cortex: Arc<CortexCore>) -> Self {
        Self {
            cortex,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            cache: SentimentCache::new(),
        }
    }

    /// Analiza sentymentu v2 z filozofią Hotza
    pub async fn analyze_v2(&mut self, text: &str) -> f32 {
        let start = std::time::Instant::now();

        // Sprawdzenie cache (filozofia: każdy cykl ma znaczenie)
        let text_hash = SentimentCache::hash_text(text);
        if let Some(cached_sentiment) = self.cache.get(text_hash) {
            return cached_sentiment;
        }

        // Budowanie zadania z kompresją
        let task = AiTaskBuilder::sentiment(text)
            .with_flags(SentimentFlags::CRYPTO_DOMAIN)
            .compress();

        // Wykonanie przez Cortex z fallback
        let raw_response = match self.cortex.dispatch(task).await {
            Ok(response) => response,
            Err(_) => {
                // Fallback Hotza - statyczna wartość
                return self.hotz_fallback(text);
            }
        };

        // Parsowanie bez kopiowania
        let score = self.parse_f32_hybrid(&raw_response)
            .unwrap_or_else(|_| self.hotz_fallback(text))
            .clamp(-1.0, 1.0);

        // Aktualizacja cache i metryk
        self.cache.insert(text_hash, score);
        
        let duration_ns = start.elapsed().as_nanos() as u64;
        {
            let mut metrics = self.metrics.write().await;
            metrics.update(duration_ns);
        }

        score
    }

    /// Fallback Hotza - deterministyczna analiza bez AI
    fn hotz_fallback(&self, text: &str) -> f32 {
        // Prosta heurystyka oparta na słowach kluczowych
        let positive_words = ["moon", "pump", "bullish", "buy", "hodl", "diamond", "rocket"];
        let negative_words = ["dump", "crash", "bearish", "sell", "rekt", "fud", "scam"];

        let text_lower = text.to_lowercase();
        let mut score = 0.0f32;

        for word in positive_words {
            if text_lower.contains(word) {
                score += 0.2;
            }
        }

        for word in negative_words {
            if text_lower.contains(word) {
                score -= 0.2;
            }
        }

        // Normalizacja do zakresu [-1.0, 1.0]
        score.clamp(-1.0, 1.0)
    }

    /// Parsowanie f32 z optymalizacją
    fn parse_f32_hybrid(&self, raw: &[u8]) -> CortexResult<f32> {
        // Próba parsowania jako binary f32
        if raw.len() >= 4 {
            let bytes = [raw[0], raw[1], raw[2], raw[3]];
            let value = f32::from_le_bytes(bytes);
            
            if value >= -1.0 && value <= 1.0 && value.is_finite() {
                return Ok(value);
            }
        }

        // Fallback: parsowanie jako tekst
        let text = std::str::from_utf8(raw)
            .map_err(|_| CortexError::ParseError("Invalid UTF-8".to_string()))?;
        
        text.trim()
            .parse::<f32>()
            .map_err(|_| CortexError::ParseError("Invalid float".to_string()))
    }

    /// Pobranie metryk wydajności
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Reset cache (dla testów)
    pub fn reset_cache(&mut self) {
        self.cache = SentimentCache::new();
    }

    /// Statystyki cache
    pub fn cache_stats(&self) -> CacheStats {
        let mut used_entries = 0;
        for (hash, _) in &self.cache.entries {
            if *hash != 0 {
                used_entries += 1;
            }
        }

        CacheStats {
            used_entries,
            total_capacity: 256,
            next_position: self.cache.next_position,
        }
    }
}

/// Statystyki cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub used_entries: usize,
    pub total_capacity: usize,
    pub next_position: usize,
}

impl CacheStats {
    pub fn utilization(&self) -> f32 {
        self.used_entries as f32 / self.total_capacity as f32
    }
}

/// Agent oceny ryzyka (placeholder dla przyszłej implementacji)
pub struct RiskAgent {
    cortex: Arc<CortexCore>,
    risk_threshold: f32,
}

impl RiskAgent {
    pub fn new(cortex: Arc<CortexCore>) -> Self {
        Self {
            cortex,
            risk_threshold: 0.7, // 70% próg ryzyka
        }
    }

    /// Ocena ryzyka dla tokena
    pub async fn assess_risk(&self, token: &str, amount: f64) -> CortexResult<f32> {
        // Placeholder - implementacja w przyszłości
        // Obecnie zwraca statyczną wartość
        let base_risk = 0.3; // 30% bazowe ryzyko
        let amount_factor = (amount / 1000.0).min(1.0) as f32; // Ryzyko rośnie z kwotą
        
        Ok((base_risk + amount_factor * 0.4).clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentiment_cache() {
        let mut cache = SentimentCache::new();
        let hash = SentimentCache::hash_text("test");
        
        assert_eq!(cache.get(hash), None);
        
        cache.insert(hash, 0.5);
        assert_eq!(cache.get(hash), Some(0.5));
    }

    #[test]
    fn test_hotz_fallback() {
        // Test fallback bez mock - używa tylko lokalnej logiki
        let cortex = Arc::new(CortexCore::new().expect("Failed to create cortex"));
        let agent = SentimentAgent::new(cortex);

        let positive_score = agent.hotz_fallback("BTC to the moon!");
        assert!(positive_score > 0.0);

        let negative_score = agent.hotz_fallback("Market crash incoming");
        assert!(negative_score < 0.0);
    }

    #[test]
    fn test_cache_stats() {
        let cortex = Arc::new(CortexCore::new().expect("Failed to create cortex"));
        let agent = SentimentAgent::new(cortex);

        let stats = agent.cache_stats();
        assert_eq!(stats.used_entries, 0);
        assert_eq!(stats.total_capacity, 256);
        assert_eq!(stats.utilization(), 0.0);
    }

    #[tokio::test]
    async fn test_sentiment_analysis_performance() {
        // Test wydajności fallback - powinien być <580μs zgodnie z metrykami
        let cortex = Arc::new(CortexCore::new().expect("Failed to create cortex"));
        let mut agent = SentimentAgent::new(cortex);

        let start = std::time::Instant::now();
        // Test tylko fallback (bez rzeczywistego AI)
        let _score = agent.hotz_fallback("Test sentiment");
        let duration = start.elapsed();

        // Sprawdzenie czy fallback mieści się w limicie Hotza
        assert!(duration.as_micros() < 100, "Fallback took {}μs, expected <100μs", duration.as_micros());
    }
}
