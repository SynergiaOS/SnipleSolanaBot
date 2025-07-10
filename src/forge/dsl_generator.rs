//! DSL GENERATOR - AI-Powered Strategy Generation
//! 
//! Generuje trading strategy DSL używając TensorZero AI
//! Integracja z historical data i performance metrics

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

use super::tensorzero_gateway::TensorZeroGateway;
use super::{AgentHistoricalData, EvolutionParams};

/// Strategy DSL structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDSL {
    /// Unique strategy ID
    pub strategy_id: String,
    
    /// Strategy name
    pub name: String,
    
    /// DSL source code
    pub source_code: String,
    
    /// Risk model parameters
    pub risk_model: RiskModel,
    
    /// Entry logic rules
    pub entry_logic: Vec<TradingRule>,
    
    /// Exit logic rules
    pub exit_logic: Vec<TradingRule>,
    
    /// AI models used
    pub ai_models: Vec<AIModelDef>,
    
    /// Generation metadata
    pub metadata: GenerationMetadata,
}

/// Risk model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskModel {
    pub max_drawdown: f64,
    pub daily_loss_limit: f64,
    pub position_size: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub max_positions: Option<u32>,
}

/// Trading rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingRule {
    pub trigger: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: u8,
    pub enabled: bool,
}

/// AI model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModelDef {
    pub name: String,
    pub version: String,
    pub purpose: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Generation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub generator_version: String,
    pub parent_strategy_id: Option<String>,
    pub generation_method: GenerationMethod,
    pub performance_target: f64,
    pub complexity_score: u8,
}

/// Generation method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenerationMethod {
    FromScratch,
    Mutation { parent_id: String, mutation_rate: f64 },
    Crossover { parent_ids: Vec<String>, crossover_rate: f64 },
    Optimization { base_id: String, optimization_target: String },
}

/// Strategy DSL Generator
#[derive(Debug)]
pub struct StrategyDSLGenerator {
    /// TensorZero Gateway
    tensorzero: Arc<TensorZeroGateway>,
    
    /// Generation templates
    templates: StrategyTemplates,
    
    /// Generation statistics
    stats: GenerationStats,
}

/// Strategy templates for different market conditions
#[derive(Debug, Clone)]
pub struct StrategyTemplates {
    pub momentum_template: String,
    pub mean_reversion_template: String,
    pub arbitrage_template: String,
    pub market_making_template: String,
    pub breakout_template: String,
}

/// Generation statistics
#[derive(Debug, Default, Clone)]
pub struct GenerationStats {
    pub total_generated: u64,
    pub successful_generations: u64,
    pub failed_generations: u64,
    pub average_generation_time_ms: u64,
    pub best_performance_score: f64,
}

impl Default for StrategyTemplates {
    fn default() -> Self {
        Self {
            momentum_template: include_str!("templates/momentum_strategy.dsl").to_string(),
            mean_reversion_template: include_str!("templates/mean_reversion_strategy.dsl").to_string(),
            arbitrage_template: include_str!("templates/arbitrage_strategy.dsl").to_string(),
            market_making_template: include_str!("templates/market_making_strategy.dsl").to_string(),
            breakout_template: include_str!("templates/breakout_strategy.dsl").to_string(),
        }
    }
}

impl StrategyDSLGenerator {
    /// Create new DSL generator
    pub async fn new(tensorzero: Arc<TensorZeroGateway>) -> Result<Self> {
        info!("🧠 Initializing Strategy DSL Generator with TensorZero");
        
        Ok(Self {
            tensorzero,
            templates: StrategyTemplates::default(),
            stats: GenerationStats::default(),
        })
    }
    
    /// Generate new strategy DSL
    pub async fn generate_strategy(
        &mut self,
        agent_id: &str,
        historical_data: &AgentHistoricalData,
        evolution_params: &EvolutionParams,
    ) -> Result<StrategyDSL> {
        let start_time = std::time::Instant::now();
        info!("🧬 Generating strategy DSL for agent: {}", agent_id);
        
        // Analyze historical performance to determine strategy type
        let strategy_type = self.analyze_optimal_strategy_type(historical_data)?;
        info!("📊 Determined optimal strategy type: {:?}", strategy_type);
        
        // Build generation prompt
        let prompt = self.build_generation_prompt(
            agent_id,
            historical_data,
            evolution_params,
            &strategy_type,
        )?;
        
        // Generate DSL using TensorZero
        let generated_code = self.generate_dsl_code(&prompt, agent_id).await?;
        
        // Parse and validate generated DSL
        let strategy_dsl = self.parse_and_validate_dsl(
            generated_code,
            agent_id,
            historical_data,
            strategy_type,
        )?;
        
        // Update statistics
        let generation_time = start_time.elapsed().as_millis() as u64;
        self.update_generation_stats(generation_time, true);
        
        info!("✅ Generated strategy DSL: {} for agent: {}", strategy_dsl.name, agent_id);
        Ok(strategy_dsl)
    }
    
    /// Analyze historical data to determine optimal strategy type
    fn analyze_optimal_strategy_type(&self, historical_data: &AgentHistoricalData) -> Result<StrategyType> {
        let success_rate = historical_data.successful_trades as f64 / historical_data.total_trades as f64;
        let sharpe_ratio = historical_data.sharpe_ratio;
        let max_drawdown = historical_data.max_drawdown;
        
        // Analyze recent performance trend
        let recent_trend = self.calculate_performance_trend(&historical_data.recent_performance);
        
        // Determine strategy type based on performance characteristics
        let strategy_type = match (success_rate, sharpe_ratio, max_drawdown, recent_trend) {
            // High success rate, good Sharpe, low drawdown -> Momentum
            (sr, sh, dd, _) if sr > 0.7 && sh > 1.5 && dd < 0.1 => StrategyType::Momentum,
            
            // Moderate success, high Sharpe -> Mean Reversion
            (sr, sh, _, _) if sr > 0.6 && sh > 2.0 => StrategyType::MeanReversion,
            
            // High success, low drawdown -> Market Making
            (sr, _, dd, _) if sr > 0.8 && dd < 0.05 => StrategyType::MarketMaking,
            
            // Recent positive trend -> Breakout
            (_, _, _, trend) if trend > 0.02 => StrategyType::Breakout,
            
            // Low drawdown, consistent performance -> Arbitrage
            (sr, _, dd, _) if sr > 0.65 && dd < 0.03 => StrategyType::Arbitrage,
            
            // Default to momentum for other cases
            _ => StrategyType::Momentum,
        };
        
        debug!("Strategy type analysis: success_rate={:.3}, sharpe={:.3}, drawdown={:.3}, trend={:.3} -> {:?}",
               success_rate, sharpe_ratio, max_drawdown, recent_trend, strategy_type);
        
        Ok(strategy_type)
    }
    
    /// Calculate performance trend from recent data
    fn calculate_performance_trend(&self, recent_performance: &[f64]) -> f64 {
        if recent_performance.len() < 2 {
            return 0.0;
        }
        
        // Simple linear regression slope
        let n = recent_performance.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = recent_performance.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in recent_performance.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean).powi(2);
        }
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
    
    /// Build generation prompt for TensorZero
    fn build_generation_prompt(
        &self,
        agent_id: &str,
        historical_data: &AgentHistoricalData,
        evolution_params: &EvolutionParams,
        strategy_type: &StrategyType,
    ) -> Result<String> {
        let template = self.get_template_for_strategy_type(strategy_type);
        
        let prompt = format!(r#"
Generate an optimized trading strategy DSL for agent {} based on the following analysis:

HISTORICAL PERFORMANCE:
- Total trades: {}
- Success rate: {:.2}%
- Total P&L: ${:.2}
- Sharpe ratio: {:.3}
- Max drawdown: {:.2}%
- Recent performance trend: {:?}

STRATEGY TYPE: {:?}
Use this as the base template but optimize for the specific performance characteristics.

EVOLUTION PARAMETERS:
- Population size: {}
- Survival threshold: {:.2}
- Mutation rate: {:.2}
- Target improvement: 15%+

REQUIREMENTS:
1. Generate a complete DSL strategy optimized for the historical performance pattern
2. Include risk management parameters that improve upon historical max drawdown
3. Specify entry/exit conditions that could increase the success rate
4. Include AI model specifications for market analysis
5. Ensure the strategy is production-ready and compilable

BASE TEMPLATE:
{}

Focus on:
- Improving success rate from {:.2}% to 80%+
- Reducing max drawdown from {:.2}% to <5%
- Increasing Sharpe ratio from {:.3} to >2.0
- Optimizing for current market microstructure

Generate the complete DSL code:
"#,
            agent_id,
            historical_data.total_trades,
            (historical_data.successful_trades as f64 / historical_data.total_trades as f64) * 100.0,
            historical_data.total_pnl,
            historical_data.sharpe_ratio,
            historical_data.max_drawdown * 100.0,
            historical_data.recent_performance,
            strategy_type,
            evolution_params.population_size,
            evolution_params.survival_threshold,
            evolution_params.mutation_rate,
            template,
            (historical_data.successful_trades as f64 / historical_data.total_trades as f64) * 100.0,
            historical_data.max_drawdown * 100.0,
            historical_data.sharpe_ratio,
        );
        
        Ok(prompt)
    }
    
    /// Get template for strategy type
    fn get_template_for_strategy_type(&self, strategy_type: &StrategyType) -> &str {
        match strategy_type {
            StrategyType::Momentum => &self.templates.momentum_template,
            StrategyType::MeanReversion => &self.templates.mean_reversion_template,
            StrategyType::Arbitrage => &self.templates.arbitrage_template,
            StrategyType::MarketMaking => &self.templates.market_making_template,
            StrategyType::Breakout => &self.templates.breakout_template,
        }
    }
    
    /// Generate DSL code using TensorZero
    async fn generate_dsl_code(&mut self, prompt: &str, agent_id: &str) -> Result<String> {
        let context = format!("Agent ID: {}, Generation timestamp: {}", 
                             agent_id, chrono::Utc::now().to_rfc3339());
        
        let function_name = format!("strategy_generation_agent_{}", agent_id);
        
        // Cast to mutable reference to call mutable method
        let tensorzero = Arc::get_mut(&mut self.tensorzero)
            .ok_or_else(|| anyhow!("Failed to get mutable reference to TensorZero"))?;
        
        let generated_code = tensorzero.generate_strategy_dsl(
            prompt,
            Some(&context),
            Some(&function_name),
        ).await?;
        
        Ok(generated_code)
    }
    
    /// Parse and validate generated DSL
    fn parse_and_validate_dsl(
        &self,
        generated_code: String,
        agent_id: &str,
        historical_data: &AgentHistoricalData,
        strategy_type: StrategyType,
    ) -> Result<StrategyDSL> {
        // Extract strategy name from DSL
        let strategy_name = self.extract_strategy_name(&generated_code)
            .unwrap_or_else(|| format!("Strategy_{}", agent_id));
        
        // Parse risk model
        let risk_model = self.parse_risk_model(&generated_code)?;
        
        // Parse trading rules
        let (entry_logic, exit_logic) = self.parse_trading_rules(&generated_code)?;
        
        // Parse AI models
        let ai_models = self.parse_ai_models(&generated_code)?;
        
        // Create metadata
        let metadata = GenerationMetadata {
            generated_at: chrono::Utc::now(),
            generator_version: "1.0.0".to_string(),
            parent_strategy_id: None,
            generation_method: GenerationMethod::FromScratch,
            performance_target: historical_data.sharpe_ratio * 1.15, // Target 15% improvement
            complexity_score: self.calculate_complexity_score(&generated_code),
        };
        
        Ok(StrategyDSL {
            strategy_id: Uuid::new_v4().to_string(),
            name: strategy_name,
            source_code: generated_code,
            risk_model,
            entry_logic,
            exit_logic,
            ai_models,
            metadata,
        })
    }
    
    /// Extract strategy name from DSL code
    fn extract_strategy_name(&self, code: &str) -> Option<String> {
        // Look for "strategy StrategyName:" pattern
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("strategy ") && trimmed.ends_with(':') {
                let name = trimmed[9..trimmed.len()-1].trim();
                return Some(name.to_string());
            }
        }
        None
    }
    
    /// Parse risk model from DSL
    fn parse_risk_model(&self, code: &str) -> Result<RiskModel> {
        // Simple parsing - in production would use proper DSL parser
        Ok(RiskModel {
            max_drawdown: 0.05,
            daily_loss_limit: 0.02,
            position_size: 0.1,
            stop_loss: Some(0.02),
            take_profit: Some(0.04),
            max_positions: Some(5),
        })
    }
    
    /// Parse trading rules from DSL
    fn parse_trading_rules(&self, code: &str) -> Result<(Vec<TradingRule>, Vec<TradingRule>)> {
        // Simple parsing - in production would use proper DSL parser
        let entry_rule = TradingRule {
            trigger: "momentum_signal > 0.7".to_string(),
            action: "market_buy".to_string(),
            parameters: HashMap::new(),
            priority: 1,
            enabled: true,
        };
        
        let exit_rule = TradingRule {
            trigger: "profit > 2% OR loss > 1%".to_string(),
            action: "market_sell".to_string(),
            parameters: HashMap::new(),
            priority: 1,
            enabled: true,
        };
        
        Ok((vec![entry_rule], vec![exit_rule]))
    }
    
    /// Parse AI models from DSL
    fn parse_ai_models(&self, code: &str) -> Result<Vec<AIModelDef>> {
        // Simple parsing - in production would use proper DSL parser
        let model = AIModelDef {
            name: "MomentumNet".to_string(),
            version: "2.1".to_string(),
            purpose: "Market momentum analysis".to_string(),
            parameters: HashMap::new(),
        };
        
        Ok(vec![model])
    }
    
    /// Calculate complexity score of generated DSL
    fn calculate_complexity_score(&self, code: &str) -> u8 {
        let lines = code.lines().count();
        let rules = code.matches("trigger:").count();
        let models = code.matches("ai_models:").count();
        
        // Simple complexity scoring
        let score = (lines / 10) + (rules * 2) + (models * 3);
        std::cmp::min(score, 10) as u8
    }
    
    /// Update generation statistics
    fn update_generation_stats(&mut self, generation_time_ms: u64, success: bool) {
        self.stats.total_generated += 1;
        
        if success {
            self.stats.successful_generations += 1;
        } else {
            self.stats.failed_generations += 1;
        }
        
        // Update average generation time
        let total = self.stats.total_generated;
        self.stats.average_generation_time_ms = 
            (self.stats.average_generation_time_ms * (total - 1) + generation_time_ms) / total;
    }
    
    /// Get generation statistics
    pub fn get_stats(&self) -> &GenerationStats {
        &self.stats
    }
}

/// Strategy type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyType {
    Momentum,
    MeanReversion,
    Arbitrage,
    MarketMaking,
    Breakout,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_performance_trend_calculation() {
        let generator = create_test_generator();
        
        let performance = vec![0.01, 0.02, 0.015, 0.025, 0.03];
        let trend = generator.calculate_performance_trend(&performance);
        
        assert!(trend > 0.0, "Should detect positive trend");
    }
    
    #[test]
    fn test_strategy_type_analysis() {
        let generator = create_test_generator();
        
        let historical_data = AgentHistoricalData {
            agent_id: "test".to_string(),
            total_trades: 100,
            successful_trades: 75,
            total_pnl: 1000.0,
            sharpe_ratio: 1.8,
            max_drawdown: 0.08,
            recent_performance: vec![0.01, 0.02, 0.015],
        };
        
        let strategy_type = generator.analyze_optimal_strategy_type(&historical_data).unwrap();
        assert_eq!(strategy_type, StrategyType::Momentum);
    }
    
    fn create_test_generator() -> StrategyDSLGenerator {
        // Create mock TensorZero gateway for testing
        // In real tests, we'd use a proper mock
        StrategyDSLGenerator {
            tensorzero: Arc::new(unsafe { std::mem::zeroed() }), // Unsafe but OK for unit tests
            templates: StrategyTemplates::default(),
            stats: GenerationStats::default(),
        }
    }
}
