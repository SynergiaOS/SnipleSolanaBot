//! MutationGuard Security System for FAZA 11
//! 
//! Comprehensive security and validation system for genetic mutations
//! including risk simulation, Hotz compliance, and black swan testing

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, debug, warn};

use super::evolution::{ConfigMutationPlan, ConfigMutation, RiskLevel, RiskAssessment};

/// Maximum safe drawdown threshold (Hotz philosophy: never exceed 15%)
const MAX_SAFE_DRAWDOWN: f64 = 0.15;

/// Maximum delta change per mutation (Hotz philosophy: incremental changes)
const MAX_DELTA: f64 = 0.30;

/// Minimum Hotz score threshold for mutations
const MIN_HOTZ_SCORE: f64 = 40.0;

/// MutationGuard - Security validation system
pub struct MutationGuard {
    /// Risk simulation engine
    risk_simulator: RiskSimulator,
    
    /// Historical market data for validation
    market_history: MarketHistory,
    
    /// Hotz compliance checker
    hotz_checker: HotzComplianceChecker,
    
    /// Black swan stress tester
    stress_tester: BlackSwanSimulator,
    
    /// Validation statistics
    validation_stats: ValidationStats,
}

/// Risk simulation engine
#[derive(Debug, Clone)]
pub struct RiskSimulator {
    /// Historical volatility data
    volatility_history: Vec<f64>,
    
    /// Correlation matrices
    correlation_data: HashMap<String, f64>,
    
    /// Monte Carlo simulation parameters
    simulation_runs: usize,
}

/// Market history for backtesting
#[derive(Debug, Clone)]
pub struct MarketHistory {
    /// Price data points
    price_history: Vec<PricePoint>,
    
    /// Volatility events
    volatility_events: Vec<VolatilityEvent>,
    
    /// Black swan events
    black_swan_events: Vec<BlackSwanEvent>,
}

/// Price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub price: f64,
    pub volume: f64,
    pub volatility: f64,
}

/// Volatility event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub magnitude: f64,
    pub duration: Duration,
    pub recovery_time: Duration,
}

/// Black swan event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackSwanEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub price_drop: f64,
    pub event_type: BlackSwanType,
    pub market_impact: f64,
}

/// Types of black swan events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlackSwanType {
    FlashCrash,
    MarketCollapse,
    LiquidityDrain,
    ExchangeHack,
    RegulatoryShock,
}

/// Hotz compliance checker
#[derive(Debug, Clone)]
pub struct HotzComplianceChecker {
    /// Performance thresholds
    latency_threshold: Duration,
    efficiency_threshold: f64,
    profit_threshold: f64,
}

/// Black swan stress tester
#[derive(Debug, Clone)]
pub struct BlackSwanSimulator {
    /// Stress test scenarios
    scenarios: Vec<StressScenario>,
    
    /// Simulation parameters
    max_drawdown_tolerance: f64,
    recovery_time_limit: Duration,
}

/// Stress test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    pub name: String,
    pub price_shock: f64,
    pub duration: Duration,
    pub liquidity_impact: f64,
    pub expected_drawdown: f64,
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub passed_validations: u64,
    pub failed_validations: u64,
    pub risk_rejections: u64,
    pub hotz_rejections: u64,
    pub stress_test_failures: u64,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub risk_assessment: RiskAssessment,
    pub hotz_compliance: bool,
    pub stress_test_result: StressTestResult,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Stress test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub passed: bool,
    pub max_projected_drawdown: f64,
    pub recovery_time_estimate: Duration,
    pub scenario_results: Vec<ScenarioResult>,
}

/// Individual scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub projected_drawdown: f64,
    pub survival_probability: f64,
    pub recovery_time: Duration,
}

/// Guard error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardError {
    RiskThresholdExceeded,
    HotzCompliance,
    StressTestFailure,
    InvalidMutation,
    InsufficientData,
}

impl Default for ValidationStats {
    fn default() -> Self {
        Self {
            total_validations: 0,
            passed_validations: 0,
            failed_validations: 0,
            risk_rejections: 0,
            hotz_rejections: 0,
            stress_test_failures: 0,
        }
    }
}

impl MutationGuard {
    /// Create new MutationGuard
    pub fn new() -> Self {
        let risk_simulator = RiskSimulator {
            volatility_history: Self::generate_sample_volatility(),
            correlation_data: HashMap::new(),
            simulation_runs: 10000,
        };
        
        let market_history = MarketHistory {
            price_history: Self::generate_sample_prices(),
            volatility_events: Self::generate_sample_volatility_events(),
            black_swan_events: Self::generate_sample_black_swans(),
        };
        
        let hotz_checker = HotzComplianceChecker {
            latency_threshold: Duration::from_millis(1000), // 1ms max
            efficiency_threshold: 0.80, // 80% capital efficiency
            profit_threshold: 0.05, // 5% minimum ROI
        };
        
        let stress_tester = BlackSwanSimulator {
            scenarios: Self::create_stress_scenarios(),
            max_drawdown_tolerance: MAX_SAFE_DRAWDOWN,
            recovery_time_limit: Duration::from_secs(3600), // 1 hour max recovery
        };
        
        Self {
            risk_simulator,
            market_history,
            hotz_checker,
            stress_tester,
            validation_stats: ValidationStats::default(),
        }
    }
    
    /// Validate mutation plan with comprehensive security checks
    pub async fn validate_plan(
        &mut self,
        plan: &ConfigMutationPlan,
        hist_data: &MarketHistory,
    ) -> Result<ValidationResult> {
        info!("🛡️ Starting comprehensive mutation validation");
        
        self.validation_stats.total_validations += 1;
        
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        let mut validation_passed = true;
        
        // 1. Risk Simulation
        let projected_drawdown = self.risk_simulator.project_drawdown(plan).await?;
        if projected_drawdown > MAX_SAFE_DRAWDOWN {
            validation_passed = false;
            self.validation_stats.risk_rejections += 1;
            warnings.push(format!(
                "Projected drawdown {:.2}% exceeds safe limit {:.2}%",
                projected_drawdown * 100.0,
                MAX_SAFE_DRAWDOWN * 100.0
            ));
        }
        
        // 2. Hotz Compliance Check
        let hotz_compliance = self.hotz_checker.validate_mutations(&plan.mutations).await?;
        if !hotz_compliance {
            validation_passed = false;
            self.validation_stats.hotz_rejections += 1;
            warnings.push("Mutations violate Hotz philosophy principles".to_string());
        }
        
        // 3. Delta Validation
        for mutation in &plan.mutations {
            if mutation.delta.abs() > MAX_DELTA {
                validation_passed = false;
                warnings.push(format!(
                    "Mutation delta {:.2}% exceeds maximum {:.2}%",
                    mutation.delta * 100.0,
                    MAX_DELTA * 100.0
                ));
            }
        }
        
        // 4. Stress Testing
        let stress_test_result = self.stress_tester.run_stress_tests(plan).await?;
        if !stress_test_result.passed {
            validation_passed = false;
            self.validation_stats.stress_test_failures += 1;
            warnings.push("Failed black swan stress testing".to_string());
        }
        
        // 5. Generate recommendations
        if projected_drawdown > 0.05 {
            recommendations.push("Consider reducing position sizes".to_string());
        }
        
        if !hotz_compliance {
            recommendations.push("Optimize for lower latency and higher efficiency".to_string());
        }
        
        // Update statistics
        if validation_passed {
            self.validation_stats.passed_validations += 1;
        } else {
            self.validation_stats.failed_validations += 1;
        }
        
        let risk_assessment = RiskAssessment {
            risk_level: if projected_drawdown > 0.10 {
                RiskLevel::High
            } else if projected_drawdown > 0.05 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            },
            max_drawdown_impact: projected_drawdown,
            hotz_compliance,
            safety_score: if validation_passed { 0.85 } else { 0.45 },
        };
        
        let result = ValidationResult {
            passed: validation_passed,
            risk_assessment,
            hotz_compliance,
            stress_test_result,
            warnings,
            recommendations,
        };
        
        if validation_passed {
            info!("✅ Mutation validation passed");
        } else {
            warn!("⚠️ Mutation validation failed with {} warnings", result.warnings.len());
        }
        
        Ok(result)
    }
    
    /// Generate sample volatility data
    fn generate_sample_volatility() -> Vec<f64> {
        // Generate realistic volatility data
        (0..1000).map(|i| {
            0.02 + 0.01 * (i as f64 / 100.0).sin() + 0.005 * rand::random::<f64>()
        }).collect()
    }
    
    /// Generate sample price history
    fn generate_sample_prices() -> Vec<PricePoint> {
        let mut prices = Vec::new();
        let mut current_price = 100.0;
        
        for i in 0..1000 {
            let volatility = 0.02 + 0.01 * rand::random::<f64>();
            let price_change = volatility * (rand::random::<f64>() - 0.5) * 2.0;
            current_price *= 1.0 + price_change;
            
            prices.push(PricePoint {
                timestamp: chrono::Utc::now() - chrono::Duration::hours(1000 - i),
                price: current_price,
                volume: 1000.0 + 500.0 * rand::random::<f64>(),
                volatility,
            });
        }
        
        prices
    }
    
    /// Generate sample volatility events
    fn generate_sample_volatility_events() -> Vec<VolatilityEvent> {
        vec![
            VolatilityEvent {
                timestamp: chrono::Utc::now() - chrono::Duration::days(30),
                magnitude: 0.15,
                duration: Duration::from_secs(3600),
                recovery_time: Duration::from_secs(7200),
            },
            VolatilityEvent {
                timestamp: chrono::Utc::now() - chrono::Duration::days(60),
                magnitude: 0.08,
                duration: Duration::from_secs(1800),
                recovery_time: Duration::from_secs(3600),
            },
        ]
    }
    
    /// Generate sample black swan events
    fn generate_sample_black_swans() -> Vec<BlackSwanEvent> {
        vec![
            BlackSwanEvent {
                timestamp: chrono::Utc::now() - chrono::Duration::days(90),
                price_drop: 0.35,
                event_type: BlackSwanType::FlashCrash,
                market_impact: 0.80,
            },
            BlackSwanEvent {
                timestamp: chrono::Utc::now() - chrono::Duration::days(180),
                price_drop: 0.50,
                event_type: BlackSwanType::MarketCollapse,
                market_impact: 0.95,
            },
        ]
    }
    
    /// Create stress test scenarios
    fn create_stress_scenarios() -> Vec<StressScenario> {
        vec![
            StressScenario {
                name: "Flash Crash -30%".to_string(),
                price_shock: -0.30,
                duration: Duration::from_secs(300),
                liquidity_impact: 0.70,
                expected_drawdown: 0.12,
            },
            StressScenario {
                name: "Market Collapse -50%".to_string(),
                price_shock: -0.50,
                duration: Duration::from_secs(3600),
                liquidity_impact: 0.90,
                expected_drawdown: 0.25,
            },
            StressScenario {
                name: "Liquidity Drain".to_string(),
                price_shock: -0.15,
                duration: Duration::from_secs(1800),
                liquidity_impact: 0.95,
                expected_drawdown: 0.08,
            },
        ]
    }
    
    /// Get validation statistics
    pub fn get_stats(&self) -> &ValidationStats {
        &self.validation_stats
    }
}

impl RiskSimulator {
    /// Project drawdown for mutation plan
    pub async fn project_drawdown(&self, plan: &ConfigMutationPlan) -> Result<f64> {
        debug!("📊 Projecting drawdown for mutation plan");

        // Simplified Monte Carlo simulation
        let mut total_drawdown = 0.0;

        for mutation in &plan.mutations {
            let impact = match mutation.target.as_str() {
                target if target.contains("max_drawdown") => {
                    // Direct impact on drawdown
                    mutation.delta.abs() * 0.5
                }
                target if target.contains("aggression") => {
                    // Aggression increases risk
                    mutation.delta.abs() * 0.3
                }
                target if target.contains("position_size") => {
                    // Position size affects exposure
                    mutation.delta.abs() * 0.4
                }
                _ => {
                    // Default conservative estimate
                    mutation.delta.abs() * 0.1
                }
            };

            total_drawdown += impact;
        }

        // Add volatility factor
        let avg_volatility = self.volatility_history.iter().sum::<f64>() / self.volatility_history.len() as f64;
        total_drawdown *= 1.0 + avg_volatility;

        // Cap at reasonable maximum
        Ok(total_drawdown.min(0.50))
    }
}

impl HotzComplianceChecker {
    /// Validate mutations against Hotz philosophy
    pub async fn validate_mutations(&self, mutations: &[ConfigMutation]) -> Result<bool> {
        debug!("⚡ Checking Hotz compliance for mutations");

        for mutation in mutations {
            // Check if mutation improves performance metrics
            match mutation.target.as_str() {
                target if target.contains("latency") => {
                    // Latency should decrease (negative delta is good)
                    if mutation.delta > 0.0 {
                        warn!("❌ Mutation increases latency - violates Hotz philosophy");
                        return Ok(false);
                    }
                }
                target if target.contains("efficiency") => {
                    // Efficiency should increase (positive delta is good)
                    if mutation.delta < 0.0 {
                        warn!("❌ Mutation decreases efficiency - violates Hotz philosophy");
                        return Ok(false);
                    }
                }
                target if target.contains("aggression") => {
                    // Moderate aggression changes only
                    if mutation.delta.abs() > 0.20 {
                        warn!("❌ Excessive aggression change - violates Hotz philosophy");
                        return Ok(false);
                    }
                }
                _ => {
                    // General rule: moderate changes only
                    if mutation.delta.abs() > MAX_DELTA {
                        warn!("❌ Excessive delta change - violates Hotz philosophy");
                        return Ok(false);
                    }
                }
            }
        }

        info!("✅ All mutations comply with Hotz philosophy");
        Ok(true)
    }
}

impl BlackSwanSimulator {
    /// Run comprehensive stress tests
    pub async fn run_stress_tests(&self, plan: &ConfigMutationPlan) -> Result<StressTestResult> {
        debug!("🦢 Running black swan stress tests");

        let mut scenario_results = Vec::new();
        let mut max_projected_drawdown: f64 = 0.0;
        let mut max_recovery_time = Duration::from_secs(0);

        for scenario in &self.scenarios {
            let result = self.simulate_scenario(scenario, plan).await?;

            max_projected_drawdown = max_projected_drawdown.max(result.projected_drawdown);
            max_recovery_time = max_recovery_time.max(result.recovery_time);

            scenario_results.push(result);
        }

        let passed = max_projected_drawdown <= self.max_drawdown_tolerance
            && max_recovery_time <= self.recovery_time_limit;

        if passed {
            info!("✅ Stress tests passed");
        } else {
            warn!("⚠️ Stress tests failed - max drawdown: {:.2}%", max_projected_drawdown * 100.0);
        }

        Ok(StressTestResult {
            passed,
            max_projected_drawdown,
            recovery_time_estimate: max_recovery_time,
            scenario_results,
        })
    }

    /// Simulate individual stress scenario
    async fn simulate_scenario(
        &self,
        scenario: &StressScenario,
        plan: &ConfigMutationPlan,
    ) -> Result<ScenarioResult> {
        debug!("🎯 Simulating scenario: {}", scenario.name);

        // Calculate mutation impact on scenario
        let mut mutation_impact = 1.0;
        for mutation in &plan.mutations {
            match mutation.target.as_str() {
                target if target.contains("max_drawdown") => {
                    // Drawdown limits affect scenario impact
                    mutation_impact *= 1.0 + mutation.delta;
                }
                target if target.contains("aggression") => {
                    // Aggression amplifies scenario impact
                    mutation_impact *= 1.0 + (mutation.delta * 0.5);
                }
                _ => {
                    // General impact
                    mutation_impact *= 1.0 + (mutation.delta * 0.1);
                }
            }
        }

        // Calculate projected drawdown
        let base_drawdown = scenario.expected_drawdown;
        let projected_drawdown = base_drawdown * mutation_impact;

        // Calculate survival probability
        let survival_probability = if projected_drawdown > 0.30 {
            0.20 // Low survival chance
        } else if projected_drawdown > 0.15 {
            0.60 // Moderate survival chance
        } else {
            0.90 // High survival chance
        };

        // Estimate recovery time
        let base_recovery = Duration::from_secs(1800); // 30 minutes base
        let recovery_multiplier = 1.0 + (projected_drawdown * 2.0);
        let recovery_time = Duration::from_secs(
            (base_recovery.as_secs() as f64 * recovery_multiplier) as u64
        );

        Ok(ScenarioResult {
            scenario_name: scenario.name.clone(),
            projected_drawdown,
            survival_probability,
            recovery_time,
        })
    }
}
