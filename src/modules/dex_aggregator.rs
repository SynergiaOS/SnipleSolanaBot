// THE OVERMIND PROTOCOL - DEX Aggregator Module
// Advanced DEX aggregation with Jupiter v6 support and intelligent caching

use anyhow::{anyhow, Result};
use moka::future::Cache;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexQuote {
    pub dex_name: String,
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub fee_amount: u64,
    pub route: Vec<String>,
    pub estimated_gas: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedQuote {
    pub best_quote: DexQuote,
    pub all_quotes: Vec<DexQuote>,
    pub price_difference_percent: f64,
    pub arbitrage_opportunity: bool,
    pub recommended_dex: String,
}

#[derive(Debug, Clone)]
pub struct DexAggregator {
    endpoints: HashMap<&'static str, Url>,
    cache: Arc<Cache<String, AggregatedQuote>>,
    http_client: reqwest::Client,
    cache_ttl: Duration,
    max_slippage: f64,
}

impl DexAggregator {
    /// Create new DEX aggregator with caching
    pub fn new() -> Result<Self> {
        let mut endpoints = HashMap::new();
        
        // Jupiter v6 API
        endpoints.insert(
            "jupiter",
            Url::parse("https://quote-api.jup.ag/v6")
                .map_err(|e| anyhow!("Invalid Jupiter URL: {}", e))?,
        );
        
        // Raydium API
        endpoints.insert(
            "raydium",
            Url::parse("https://api.raydium.io/v2")
                .map_err(|e| anyhow!("Invalid Raydium URL: {}", e))?,
        );
        
        // Orca API
        endpoints.insert(
            "orca",
            Url::parse("https://api.orca.so")
                .map_err(|e| anyhow!("Invalid Orca URL: {}", e))?,
        );

        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(5))
            .max_capacity(1000)
            .build();

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            endpoints,
            cache: Arc::new(cache),
            http_client,
            cache_ttl: Duration::from_secs(5),
            max_slippage: 0.05, // 5% max slippage
        })
    }

    /// Get best quote across all DEXes with caching
    pub async fn get_best_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<AggregatedQuote> {
        let cache_key = format!("{}-{}-{}", input_mint, output_mint, amount);
        
        // Check cache first
        if let Some(cached_quote) = self.cache.get(&cache_key).await {
            debug!("📋 Cache hit for quote: {}", cache_key);
            return Ok(cached_quote);
        }

        info!("🔍 Fetching quotes for {}->{} amount: {}", input_mint, output_mint, amount);

        // Fetch quotes from all DEXes concurrently
        let jupiter_future = self.fetch_jupiter_quote(input_mint, output_mint, amount);
        let raydium_future = self.fetch_raydium_quote(input_mint, output_mint, amount);
        let orca_future = self.fetch_orca_quote(input_mint, output_mint, amount);

        let quote_results = futures::future::join_all(vec![
            Box::pin(jupiter_future) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<DexQuote>>> + Send>>,
            Box::pin(raydium_future) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<DexQuote>>> + Send>>,
            Box::pin(orca_future) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<DexQuote>>> + Send>>,
        ]).await;
        
        // Collect successful quotes
        let mut quotes = Vec::new();
        for (i, result) in quote_results.into_iter().enumerate() {
            match result {
                Ok(Some(quote)) => quotes.push(quote),
                Ok(None) => debug!("No quote from DEX {}", i),
                Err(e) => warn!("Failed to get quote from DEX {}: {}", i, e),
            }
        }

        if quotes.is_empty() {
            return Err(anyhow!("No quotes available from any DEX"));
        }

        // Find best quote (highest output amount)
        let best_quote = quotes
            .iter()
            .max_by_key(|q| q.output_amount)
            .unwrap()
            .clone();

        // Calculate price differences and arbitrage opportunities
        let worst_quote = quotes
            .iter()
            .min_by_key(|q| q.output_amount)
            .unwrap();

        let price_difference_percent = if worst_quote.output_amount > 0 {
            ((best_quote.output_amount as f64 - worst_quote.output_amount as f64) 
                / worst_quote.output_amount as f64) * 100.0
        } else {
            0.0
        };

        let arbitrage_opportunity = price_difference_percent > 1.0; // 1% threshold

        let aggregated = AggregatedQuote {
            best_quote: best_quote.clone(),
            all_quotes: quotes,
            price_difference_percent,
            arbitrage_opportunity,
            recommended_dex: best_quote.dex_name.clone(),
        };

        // Cache the result
        self.cache.insert(cache_key, aggregated.clone()).await;
        
        info!("✅ Best quote: {} from {} ({}% better than worst)", 
              best_quote.output_amount, best_quote.dex_name, price_difference_percent);

        Ok(aggregated)
    }

    /// Fetch quote from Jupiter v6
    async fn fetch_jupiter_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<DexQuote>> {
        let mut url = self.endpoints["jupiter"].clone();
        url.set_path("/v6/quote");
        
        let params = [
            ("inputMint", input_mint),
            ("outputMint", output_mint),
            ("amount", &amount.to_string()),
            ("slippageBps", "50"), // 0.5% slippage
            ("onlyDirectRoutes", "false"),
            ("asLegacyTransaction", "false"),
        ];

        url.query_pairs_mut().extend_pairs(&params);

        debug!("🪐 Fetching Jupiter quote: {}", url);

        let response = self
            .http_client
            .get(url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: Value = response.json().await?;
        
        if let Some(out_amount) = json["outAmount"].as_str() {
            let output_amount = out_amount.parse::<u64>().unwrap_or(0);
            let price_impact = json["priceImpactPct"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let route = json["routePlan"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|step| step["swapInfo"]["label"].as_str())
                        .map(|s| s.to_string())
                        .collect()
                })
                .unwrap_or_default();

            Ok(Some(DexQuote {
                dex_name: "Jupiter".to_string(),
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                input_amount: amount,
                output_amount,
                price_impact,
                fee_amount: 0, // Jupiter doesn't specify fees separately
                route,
                estimated_gas: 5000, // Estimated
                timestamp: chrono::Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Fetch quote from Raydium
    async fn fetch_raydium_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<DexQuote>> {
        let mut url = self.endpoints["raydium"].clone();
        url.set_path("/compute/swap-base-in");
        
        let params = [
            ("inputMint", input_mint),
            ("outputMint", output_mint),
            ("amount", &amount.to_string()),
            ("slippageBps", "50"),
        ];

        url.query_pairs_mut().extend_pairs(&params);

        debug!("🌊 Fetching Raydium quote: {}", url);

        let response = self
            .http_client
            .get(url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: Value = response.json().await?;
        
        if let Some(data) = json["data"].as_object() {
            if let Some(out_amount) = data["outputAmount"].as_str() {
                let output_amount = out_amount.parse::<u64>().unwrap_or(0);
                let price_impact = data["priceImpact"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);

                Ok(Some(DexQuote {
                    dex_name: "Raydium".to_string(),
                    input_mint: input_mint.to_string(),
                    output_mint: output_mint.to_string(),
                    input_amount: amount,
                    output_amount,
                    price_impact,
                    fee_amount: 0,
                    route: vec!["Raydium".to_string()],
                    estimated_gas: 4000,
                    timestamp: chrono::Utc::now(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Fetch quote from Orca
    async fn fetch_orca_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<Option<DexQuote>> {
        let mut url = self.endpoints["orca"].clone();
        url.set_path("/v1/quote");
        
        let params = [
            ("inputMint", input_mint),
            ("outputMint", output_mint),
            ("amount", &amount.to_string()),
            ("slippage", "0.5"),
        ];

        url.query_pairs_mut().extend_pairs(&params);

        debug!("🐋 Fetching Orca quote: {}", url);

        let response = self
            .http_client
            .get(url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let json: Value = response.json().await?;
        
        if let Some(out_amount) = json["expectedOutputAmount"].as_str() {
            let output_amount = out_amount.parse::<u64>().unwrap_or(0);
            let price_impact = json["priceImpact"]
                .as_f64()
                .unwrap_or(0.0);

            Ok(Some(DexQuote {
                dex_name: "Orca".to_string(),
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                input_amount: amount,
                output_amount,
                price_impact,
                fee_amount: 0,
                route: vec!["Orca".to_string()],
                estimated_gas: 3500,
                timestamp: chrono::Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get arbitrage opportunities
    pub async fn find_arbitrage_opportunities(
        &self,
        token_pairs: Vec<(String, String)>,
        min_profit_threshold: f64,
    ) -> Result<Vec<AggregatedQuote>> {
        info!("🔍 Scanning for arbitrage opportunities across {} pairs", token_pairs.len());
        
        let mut opportunities = Vec::new();
        let amount = 1_000_000; // 1 token (assuming 6 decimals)

        for (input_mint, output_mint) in token_pairs {
            match self.get_best_quote(&input_mint, &output_mint, amount).await {
                Ok(quote) => {
                    if quote.arbitrage_opportunity && quote.price_difference_percent >= min_profit_threshold {
                        opportunities.push(quote);
                    }
                }
                Err(e) => {
                    warn!("Failed to get quote for {}->{}: {}", input_mint, output_mint, e);
                }
            }
        }

        info!("💰 Found {} arbitrage opportunities", opportunities.len());
        Ok(opportunities)
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        self.cache.invalidate_all();
        info!("🗑️ DEX aggregator cache cleared");
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (u64, u64) {
        (self.cache.entry_count(), self.cache.weighted_size())
    }

    /// Set maximum slippage tolerance
    pub fn set_max_slippage(&mut self, slippage: f64) {
        self.max_slippage = slippage;
        info!("⚙️ Max slippage set to {:.2}%", slippage * 100.0);
    }

    /// Validate quote against slippage tolerance
    pub fn validate_quote(&self, quote: &DexQuote) -> bool {
        quote.price_impact <= self.max_slippage
    }
}
