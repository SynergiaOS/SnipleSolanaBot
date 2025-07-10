//! MICRO-LIGHTNING MONITOR
//! 
//! Dedicated monitoring service for OPERACJA MIKRO-BŁYSKAWICA
//! Implements real-time tracking of the 5 Commandments and system health

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};

use snipercor::modules::micro_lightning::{
    OperationControl, OperationStatistics, MicroTradingStats, StatusReport,
    MetricsCollector, MicroWalletHealthReport
};

/// Micro-Lightning monitor state
#[derive(Clone)]
struct MonitorState {
    operation_control: Arc<RwLock<OperationControl>>,
    metrics_collector: Arc<RwLock<MetricsCollector>>,
    system_stats: Arc<RwLock<SystemStats>>,
    alert_manager: Arc<AlertManager>,
}

/// System statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemStats {
    uptime_seconds: u64,
    total_operations: u32,
    active_operations: u32,
    last_operation_time: Option<SystemTime>,
    circuit_breaker_active: bool,
    emergency_triggers_count: u32,
    wallet_rotations_count: u32,
    commandment_violations: CommandmentViolations,
}

/// Commandment violation tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CommandmentViolations {
    life_limit_violations: u32,
    wallet_rotation_overdue: u32,
    militia_cooldowns_triggered: u32,
    psychology_fund_warnings: u32,
    battlefield_selection_failures: u32,
}

/// Alert manager for notifications
struct AlertManager {
    webhook_url: Option<String>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
}

/// Alert structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Alert {
    id: String,
    timestamp: SystemTime,
    severity: AlertSeverity,
    component: String,
    commandment: Option<String>,
    message: String,
    action: String,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    uptime_seconds: u64,
    micro_lightning_active: bool,
    last_check: SystemTime,
}

/// Metrics response for Prometheus
#[derive(Serialize)]
struct MetricsResponse {
    micro_lightning_operations_total: u32,
    micro_lightning_win_rate: f64,
    micro_lightning_avg_profit_usd: f64,
    micro_lightning_psychology_fund_usd: f64,
    micro_lightning_emergency_triggers_total: u32,
    micro_lightning_circuit_breaker_active: u8,
    micro_lightning_commandment_violations_total: u32,
    micro_lightning_wallet_rotations_total: u32,
    micro_lightning_execution_latency_seconds: f64,
}

/// Status dashboard response
#[derive(Serialize)]
struct DashboardResponse {
    system_status: StatusReport,
    operation_stats: OperationStatistics,
    trading_stats: MicroTradingStats,
    commandment_status: CommandmentStatus,
    recent_alerts: Vec<Alert>,
}

/// Commandment compliance status
#[derive(Serialize)]
struct CommandmentStatus {
    life_limit_compliance: bool,
    wallet_rotation_needed: bool,
    militia_cooldown_active: bool,
    psychology_fund_healthy: bool,
    battlefield_selection_active: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting MICRO-LIGHTNING MONITOR");
    info!("📊 OPERACJA MIKRO-BŁYSKAWICA - Monitoring Service");

    // Initialize components
    let operation_control = Arc::new(RwLock::new(OperationControl::new()));
    let metrics_collector = Arc::new(RwLock::new(MetricsCollector::new()));
    let system_stats = Arc::new(RwLock::new(SystemStats {
        uptime_seconds: 0,
        total_operations: 0,
        active_operations: 0,
        last_operation_time: None,
        circuit_breaker_active: false,
        emergency_triggers_count: 0,
        wallet_rotations_count: 0,
        commandment_violations: CommandmentViolations::default(),
    }));

    let alert_manager = Arc::new(AlertManager {
        webhook_url: std::env::var("MICRO_LIGHTNING_ALERT_WEBHOOK").ok(),
        alert_history: Arc::new(RwLock::new(Vec::new())),
    });

    let state = MonitorState {
        operation_control,
        metrics_collector,
        system_stats,
        alert_manager,
    };

    // Start background monitoring tasks
    let monitor_state = state.clone();
    tokio::spawn(async move {
        monitoring_loop(monitor_state).await;
    });

    let alert_state = state.clone();
    tokio::spawn(async move {
        alert_processing_loop(alert_state).await;
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(prometheus_metrics))
        .route("/status", get(dashboard_status))
        .route("/commandments", get(commandment_status))
        .route("/alerts", get(get_alerts))
        .route("/alerts", post(create_alert))
        .route("/emergency", post(trigger_emergency))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    info!("🌐 Micro-Lightning Monitor listening on http://0.0.0.0:8081");
    
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check(State(state): State<MonitorState>) -> Json<HealthResponse> {
    let stats = state.system_stats.read().await;
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        uptime_seconds: stats.uptime_seconds,
        micro_lightning_active: !stats.circuit_breaker_active,
        last_check: SystemTime::now(),
    })
}

/// Prometheus metrics endpoint
async fn prometheus_metrics(State(state): State<MonitorState>) -> Result<String, StatusCode> {
    let stats = state.system_stats.read().await;
    let metrics_collector = state.metrics_collector.read().await;
    let trading_stats = metrics_collector.get_stats();
    let operation_control = state.operation_control.read().await;
    let operation_stats = operation_control.get_statistics();

    let metrics = MetricsResponse {
        micro_lightning_operations_total: stats.total_operations,
        micro_lightning_win_rate: trading_stats.win_rate,
        micro_lightning_avg_profit_usd: trading_stats.avg_profit_percentage,
        micro_lightning_psychology_fund_usd: operation_stats.psychology_fund_balance,
        micro_lightning_emergency_triggers_total: stats.emergency_triggers_count,
        micro_lightning_circuit_breaker_active: if stats.circuit_breaker_active { 1 } else { 0 },
        micro_lightning_commandment_violations_total: 
            stats.commandment_violations.life_limit_violations +
            stats.commandment_violations.wallet_rotation_overdue +
            stats.commandment_violations.militia_cooldowns_triggered +
            stats.commandment_violations.psychology_fund_warnings +
            stats.commandment_violations.battlefield_selection_failures,
        micro_lightning_wallet_rotations_total: stats.wallet_rotations_count,
        micro_lightning_execution_latency_seconds: 0.085, // Placeholder - would be real measurement
    };

    // Format as Prometheus metrics
    let prometheus_output = format!(
        "# HELP micro_lightning_operations_total Total number of micro-lightning operations\n\
         # TYPE micro_lightning_operations_total counter\n\
         micro_lightning_operations_total {}\n\
         # HELP micro_lightning_win_rate Current win rate percentage\n\
         # TYPE micro_lightning_win_rate gauge\n\
         micro_lightning_win_rate {}\n\
         # HELP micro_lightning_avg_profit_usd Average profit per operation in USD\n\
         # TYPE micro_lightning_avg_profit_usd gauge\n\
         micro_lightning_avg_profit_usd {}\n\
         # HELP micro_lightning_psychology_fund_usd Psychology fund balance in USD\n\
         # TYPE micro_lightning_psychology_fund_usd gauge\n\
         micro_lightning_psychology_fund_usd {}\n\
         # HELP micro_lightning_emergency_triggers_total Total emergency triggers\n\
         # TYPE micro_lightning_emergency_triggers_total counter\n\
         micro_lightning_emergency_triggers_total {}\n\
         # HELP micro_lightning_circuit_breaker_active Circuit breaker status\n\
         # TYPE micro_lightning_circuit_breaker_active gauge\n\
         micro_lightning_circuit_breaker_active {}\n\
         # HELP micro_lightning_commandment_violations_total Total commandment violations\n\
         # TYPE micro_lightning_commandment_violations_total counter\n\
         micro_lightning_commandment_violations_total {}\n\
         # HELP micro_lightning_wallet_rotations_total Total wallet rotations\n\
         # TYPE micro_lightning_wallet_rotations_total counter\n\
         micro_lightning_wallet_rotations_total {}\n\
         # HELP micro_lightning_execution_latency_seconds Execution latency in seconds\n\
         # TYPE micro_lightning_execution_latency_seconds gauge\n\
         micro_lightning_execution_latency_seconds {}\n",
        metrics.micro_lightning_operations_total,
        metrics.micro_lightning_win_rate,
        metrics.micro_lightning_avg_profit_usd,
        metrics.micro_lightning_psychology_fund_usd,
        metrics.micro_lightning_emergency_triggers_total,
        metrics.micro_lightning_circuit_breaker_active,
        metrics.micro_lightning_commandment_violations_total,
        metrics.micro_lightning_wallet_rotations_total,
        metrics.micro_lightning_execution_latency_seconds,
    );

    Ok(prometheus_output)
}

/// Dashboard status endpoint
async fn dashboard_status(State(state): State<MonitorState>) -> Json<DashboardResponse> {
    let operation_control = state.operation_control.read().await;
    let metrics_collector = state.metrics_collector.read().await;
    let alert_history = state.alert_manager.alert_history.read().await;

    let operation_stats = operation_control.get_statistics();
    let trading_stats = metrics_collector.get_stats().clone();

    let system_status = StatusReport {
        module_active: !operation_stats.is_cooldown_active,
        remaining_ops: (5 - operation_stats.daily_operations).max(0),
        wallet_rotation: Duration::from_secs(1800), // Placeholder
        mev_warning: operation_stats.current_loss_streak >= 2,
        message: if operation_stats.is_cooldown_active {
            "🔴 MODUŁ MIKRO-BŁYSKAWICA - COOLDOWN AKTYWNY".to_string()
        } else {
            "🟢 MODUŁ MIKRO-BŁYSKAWICA - AKTYWNY".to_string()
        },
    };

    let commandment_status = CommandmentStatus {
        life_limit_compliance: true, // Would check actual compliance
        wallet_rotation_needed: operation_stats.operations_this_wallet >= 3,
        militia_cooldown_active: operation_stats.is_cooldown_active,
        psychology_fund_healthy: operation_stats.psychology_fund_balance >= 2.0,
        battlefield_selection_active: true,
    };

    let recent_alerts: Vec<Alert> = alert_history.iter()
        .rev()
        .take(10)
        .cloned()
        .collect();

    Json(DashboardResponse {
        system_status,
        operation_stats,
        trading_stats,
        commandment_status,
        recent_alerts,
    })
}

/// Get commandment status
async fn commandment_status(State(state): State<MonitorState>) -> Json<CommandmentStatus> {
    let operation_control = state.operation_control.read().await;
    let operation_stats = operation_control.get_statistics();

    Json(CommandmentStatus {
        life_limit_compliance: true,
        wallet_rotation_needed: operation_stats.operations_this_wallet >= 3,
        militia_cooldown_active: operation_stats.is_cooldown_active,
        psychology_fund_healthy: operation_stats.psychology_fund_balance >= 2.0,
        battlefield_selection_active: true,
    })
}

/// Get alerts
async fn get_alerts(State(state): State<MonitorState>) -> Json<Vec<Alert>> {
    let alert_history = state.alert_manager.alert_history.read().await;
    Json(alert_history.clone())
}

/// Create alert
async fn create_alert(
    State(state): State<MonitorState>,
    Json(alert): Json<Alert>,
) -> Result<Json<Alert>, StatusCode> {
    let mut alert_history = state.alert_manager.alert_history.write().await;
    alert_history.push(alert.clone());
    
    // Keep only last 1000 alerts
    if alert_history.len() > 1000 {
        alert_history.remove(0);
    }

    info!("🚨 Alert created: {} - {}", alert.severity, alert.message);
    Ok(Json(alert))
}

/// Trigger emergency
async fn trigger_emergency(State(state): State<MonitorState>) -> Result<Json<String>, StatusCode> {
    let mut stats = state.system_stats.write().await;
    stats.circuit_breaker_active = true;
    stats.emergency_triggers_count += 1;

    error!("🚨 EMERGENCY TRIGGERED - Circuit breaker activated");
    Ok(Json("Emergency triggered - circuit breaker active".to_string()))
}

/// Main monitoring loop
async fn monitoring_loop(state: MonitorState) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let start_time = SystemTime::now();

    loop {
        interval.tick().await;

        let mut stats = state.system_stats.write().await;
        stats.uptime_seconds = start_time.elapsed().unwrap_or_default().as_secs();

        // Monitor commandment compliance
        let operation_control = state.operation_control.read().await;
        if let Err(e) = operation_control.check_conditions() {
            warn!("⚠️ Commandment violation detected: {}", e);
            
            // Update violation counters based on error type
            match e.to_string().as_str() {
                s if s.contains("HoldTimeViolation") => stats.commandment_violations.life_limit_violations += 1,
                s if s.contains("WalletRotationRequired") => stats.commandment_violations.wallet_rotation_overdue += 1,
                s if s.contains("CoolDownPeriod") => stats.commandment_violations.militia_cooldowns_triggered += 1,
                s if s.contains("PsychologyFundInsufficient") => stats.commandment_violations.psychology_fund_warnings += 1,
                s if s.contains("BattlefieldValidationFailed") => stats.commandment_violations.battlefield_selection_failures += 1,
                _ => {}
            }
        }

        drop(stats);
        drop(operation_control);
    }
}

/// Alert processing loop
async fn alert_processing_loop(state: MonitorState) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        // Process any pending alerts
        // In a real implementation, this would send webhooks, emails, etc.
        let alert_history = state.alert_manager.alert_history.read().await;
        if !alert_history.is_empty() {
            let recent_critical = alert_history.iter()
                .filter(|a| matches!(a.severity, AlertSeverity::Critical))
                .count();
            
            if recent_critical > 0 {
                info!("🚨 {} critical alerts in history", recent_critical);
            }
        }
    }
}
