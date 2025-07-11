// Neural Execution Engine - Ultra-Low Latency Trading System
// Target: <200μs execution latency, hardware-aware routing, SIMD optimizations
// Zero-copy dispatchers with atomic execution guarantees

pub mod neural_router;
pub mod atomic_executor;
pub mod neural_predictor;
pub mod hardware_accelerator;
pub mod execution_monitor;

// Re-exports for convenience
pub use neural_router::{NeuralRouter, HardwareTopology, RoutingDecision, PathOptimizer};
pub use atomic_executor::{AtomicExecutor, ZeroCopyDispatcher, ExecutionPipeline, SIMDProcessor};
pub use neural_predictor::{NeuralPredictor, MLExecutionModel, TimingPredictor, ReinforcementLearner};
pub use hardware_accelerator::{HardwareAccelerator, FPGAInterface, ASICController, CustomSilicon};
pub use execution_monitor::{ExecutionMonitor, PerformanceProfiler, BottleneckDetector, MicrosecondAnalytics};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Neural Execution Engine Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralExecutionConfig {
    /// Neural router configuration
    pub neural_router: NeuralRouterConfig,
    
    /// Atomic executor configuration
    pub atomic_executor: AtomicExecutorConfig,
    
    /// Neural predictor configuration
    pub neural_predictor: NeuralPredictorConfig,
    
    /// Hardware accelerator configuration
    pub hardware_accelerator: HardwareAcceleratorConfig,
    
    /// Execution monitor configuration
    pub execution_monitor: ExecutionMonitorConfig,
    
    /// Performance targets
    pub performance_targets: PerformanceTargets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralRouterConfig {
    /// Hardware topology discovery
    pub topology_discovery: bool,
    
    /// CPU core affinity
    pub cpu_affinity: Vec<u32>,
    
    /// GPU device IDs
    pub gpu_devices: Vec<u32>,
    
    /// FPGA device paths
    pub fpga_devices: Vec<String>,
    
    /// Load balancing algorithm
    pub load_balancing: LoadBalancingAlgorithm,
    
    /// Predictive routing enabled
    pub predictive_routing: bool,
    
    /// Route optimization interval (μs)
    pub optimization_interval_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    LatencyBased,
    ThroughputOptimized,
    MLPredictive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicExecutorConfig {
    /// Zero-copy enabled
    pub zero_copy_enabled: bool,
    
    /// SIMD instruction set
    pub simd_instruction_set: SIMDInstructionSet,
    
    /// Lock-free queue size
    pub lock_free_queue_size: usize,
    
    /// Execution pipeline depth
    pub pipeline_depth: usize,
    
    /// Atomic batch size
    pub atomic_batch_size: usize,
    
    /// Memory alignment (bytes)
    pub memory_alignment: usize,
    
    /// Cache line size (bytes)
    pub cache_line_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SIMDInstructionSet {
    AVX512,
    AVX2,
    SSE42,
    NEON,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralPredictorConfig {
    /// ML model path
    pub model_path: String,
    
    /// Prediction window (μs)
    pub prediction_window_us: u64,
    
    /// Training enabled
    pub training_enabled: bool,
    
    /// Reinforcement learning rate
    pub learning_rate: f64,
    
    /// Feature vector size
    pub feature_vector_size: usize,
    
    /// Model update interval (ms)
    pub model_update_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareAcceleratorConfig {
    /// FPGA enabled
    pub fpga_enabled: bool,
    
    /// ASIC enabled
    pub asic_enabled: bool,
    
    /// Custom silicon devices
    pub custom_silicon_devices: Vec<CustomSiliconDevice>,
    
    /// Hardware protocol
    pub hardware_protocol: HardwareProtocol,
    
    /// DMA buffer size (MB)
    pub dma_buffer_size_mb: u32,
    
    /// Hardware timeout (μs)
    pub hardware_timeout_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSiliconDevice {
    /// Device ID
    pub device_id: String,
    
    /// Device type
    pub device_type: String,
    
    /// PCIe address
    pub pcie_address: String,
    
    /// Capabilities
    pub capabilities: Vec<String>,
    
    /// Max throughput (ops/s)
    pub max_throughput_ops: u64,
    
    /// Latency (ns)
    pub latency_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareProtocol {
    PCIe,
    NVLink,
    InfiniBand,
    CustomProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMonitorConfig {
    /// Microsecond monitoring enabled
    pub microsecond_monitoring: bool,
    
    /// Performance profiling enabled
    pub performance_profiling: bool,
    
    /// Bottleneck detection enabled
    pub bottleneck_detection: bool,
    
    /// Real-time analytics enabled
    pub realtime_analytics: bool,
    
    /// Monitoring interval (μs)
    pub monitoring_interval_us: u64,
    
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Max execution latency (μs)
    pub max_execution_latency_us: u64,
    
    /// Max queue depth
    pub max_queue_depth: usize,
    
    /// Min throughput (ops/s)
    pub min_throughput_ops: u64,
    
    /// Max error rate (%)
    pub max_error_rate_percent: f64,
    
    /// Max CPU utilization (%)
    pub max_cpu_utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Target execution latency (μs)
    pub target_execution_latency_us: u64,
    
    /// Target throughput (ops/s)
    pub target_throughput_ops: u64,
    
    /// Target success rate (%)
    pub target_success_rate_percent: f64,
    
    /// Target CPU efficiency (%)
    pub target_cpu_efficiency_percent: f64,
    
    /// Target memory efficiency (%)
    pub target_memory_efficiency_percent: f64,
    
    /// Target power efficiency (ops/watt)
    pub target_power_efficiency_ops_per_watt: f64,
}

impl Default for NeuralExecutionConfig {
    fn default() -> Self {
        Self {
            neural_router: NeuralRouterConfig {
                topology_discovery: true,
                cpu_affinity: vec![0, 1, 2, 3], // First 4 cores - dedicated
                gpu_devices: vec![], // No GPU - software only for now
                fpga_devices: vec![], // No FPGA - software only
                load_balancing: LoadBalancingAlgorithm::LatencyBased,
                predictive_routing: true,
                optimization_interval_us: 5, // 5μs - ultra aggressive
            },
            atomic_executor: AtomicExecutorConfig {
                zero_copy_enabled: true,
                simd_instruction_set: SIMDInstructionSet::AVX512, // Force best available
                lock_free_queue_size: 16384, // 16K entries - smaller for speed
                pipeline_depth: 4, // Reduced depth for lower latency
                atomic_batch_size: 8, // Smaller batches for speed
                memory_alignment: 64, // Cache line aligned
                cache_line_size: 64,
            },
            neural_predictor: NeuralPredictorConfig {
                model_path: "./models/execution_predictor.onnx".to_string(),
                prediction_window_us: 100, // 100μs window - much faster
                training_enabled: false, // Disable for now to avoid errors
                learning_rate: 0.01, // Higher learning rate
                feature_vector_size: 32, // Smaller for speed
                model_update_interval_ms: 5000, // 5 seconds - less frequent
            },
            hardware_accelerator: HardwareAcceleratorConfig {
                fpga_enabled: false, // Software only
                asic_enabled: false, // Software only
                custom_silicon_devices: vec![], // No custom hardware
                hardware_protocol: HardwareProtocol::PCIe,
                dma_buffer_size_mb: 8, // Smaller buffers
                hardware_timeout_us: 50, // 50μs timeout
            },
            execution_monitor: ExecutionMonitorConfig {
                microsecond_monitoring: true,
                performance_profiling: false, // Disable to reduce overhead
                bottleneck_detection: false, // Disable to reduce overhead
                realtime_analytics: true,
                monitoring_interval_us: 50, // 50μs - less frequent
                alert_thresholds: AlertThresholds {
                    max_execution_latency_us: 200, // 200μs target
                    max_queue_depth: 100, // Smaller queue
                    min_throughput_ops: 5000, // 5K ops/s - more realistic
                    max_error_rate_percent: 1.0, // 1% - more lenient
                    max_cpu_utilization_percent: 95.0,
                },
            },
            performance_targets: PerformanceTargets {
                target_execution_latency_us: 200, // 200μs target
                target_throughput_ops: 5000, // 5K ops/s - realistic for software
                target_success_rate_percent: 99.0, // 99% - more achievable
                target_cpu_efficiency_percent: 80.0, // 80% - realistic
                target_memory_efficiency_percent: 85.0, // 85% - realistic
                target_power_efficiency_ops_per_watt: 1000.0, // Lower target
            },
        }
    }
}

/// Execution request for neural processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Request ID
    pub id: String,
    
    /// Request type
    pub request_type: ExecutionRequestType,
    
    /// Priority level
    pub priority: ExecutionPriority,
    
    /// Payload data
    pub payload: Vec<u8>,
    
    /// Execution constraints
    pub constraints: ExecutionConstraints,
    
    /// Timestamp (nanoseconds)
    pub timestamp_ns: u64,
    
    /// Deadline (nanoseconds from now)
    pub deadline_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionRequestType {
    /// Solana transaction
    SolanaTransaction,
    
    /// MEV bundle
    MEVBundle,
    
    /// Arbitrage execution
    ArbitrageExecution,
    
    /// Liquidation
    Liquidation,
    
    /// Market making
    MarketMaking,
    
    /// Custom execution
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionPriority {
    /// Ultra-high priority (emergency)
    UltraHigh,
    
    /// High priority (MEV)
    High,
    
    /// Normal priority
    Normal,
    
    /// Low priority (background)
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConstraints {
    /// Maximum latency (μs)
    pub max_latency_us: u64,
    
    /// Required hardware
    pub required_hardware: Vec<HardwareRequirement>,
    
    /// Memory requirements (bytes)
    pub memory_requirements_bytes: u64,
    
    /// CPU requirements (cores)
    pub cpu_requirements_cores: u32,
    
    /// GPU requirements
    pub gpu_requirements: Option<GPURequirement>,
    
    /// Atomic execution required
    pub atomic_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HardwareRequirement {
    CPU(CPURequirement),
    GPU(GPURequirement),
    FPGA(FPGARequirement),
    ASIC(ASICRequirement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPURequirement {
    /// Minimum cores
    pub min_cores: u32,
    
    /// Required instruction sets
    pub instruction_sets: Vec<String>,
    
    /// Cache requirements
    pub cache_requirements: CacheRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPURequirement {
    /// Minimum compute capability
    pub min_compute_capability: f32,
    
    /// Required memory (MB)
    pub required_memory_mb: u32,
    
    /// CUDA cores required
    pub cuda_cores_required: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FPGARequirement {
    /// Required logic elements
    pub logic_elements: u32,
    
    /// Required memory blocks
    pub memory_blocks: u32,
    
    /// Required DSP blocks
    pub dsp_blocks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASICRequirement {
    /// ASIC type
    pub asic_type: String,
    
    /// Required capabilities
    pub capabilities: Vec<String>,
    
    /// Performance requirements
    pub performance_requirements: PerformanceRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheRequirement {
    /// L1 cache size (KB)
    pub l1_cache_kb: u32,
    
    /// L2 cache size (KB)
    pub l2_cache_kb: u32,
    
    /// L3 cache size (KB)
    pub l3_cache_kb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirement {
    /// Required throughput (ops/s)
    pub throughput_ops: u64,
    
    /// Required latency (ns)
    pub latency_ns: u64,
    
    /// Required accuracy (%)
    pub accuracy_percent: f64,
}

/// Execution result with detailed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Request ID
    pub request_id: String,
    
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Result data
    pub result_data: Option<Vec<u8>>,
    
    /// Execution metrics
    pub metrics: ExecutionMetrics,
    
    /// Error information
    pub error: Option<ExecutionError>,
    
    /// Completion timestamp (nanoseconds)
    pub completion_timestamp_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// Successfully completed
    Success,
    
    /// Failed with error
    Failed,
    
    /// Timed out
    Timeout,
    
    /// Cancelled
    Cancelled,
    
    /// Partially completed
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total execution time (μs)
    pub execution_time_us: f64,
    
    /// Queue time (μs)
    pub queue_time_us: f64,
    
    /// Processing time (μs)
    pub processing_time_us: f64,
    
    /// Hardware utilization
    pub hardware_utilization: HardwareUtilization,
    
    /// Memory usage (bytes)
    pub memory_usage_bytes: u64,
    
    /// CPU cycles used
    pub cpu_cycles_used: u64,
    
    /// Cache hits/misses
    pub cache_stats: CacheStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HardwareUtilization {
    /// CPU utilization (%)
    pub cpu_utilization_percent: f64,
    
    /// GPU utilization (%)
    pub gpu_utilization_percent: f64,
    
    /// Memory utilization (%)
    pub memory_utilization_percent: f64,
    
    /// FPGA utilization (%)
    pub fpga_utilization_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// L1 cache hits
    pub l1_hits: u64,
    
    /// L1 cache misses
    pub l1_misses: u64,
    
    /// L2 cache hits
    pub l2_hits: u64,
    
    /// L2 cache misses
    pub l2_misses: u64,
    
    /// L3 cache hits
    pub l3_hits: u64,
    
    /// L3 cache misses
    pub l3_misses: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    /// Error code
    pub error_code: String,
    
    /// Error message
    pub error_message: String,
    
    /// Error category
    pub error_category: ErrorCategory,
    
    /// Recovery suggestions
    pub recovery_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Hardware error
    Hardware,
    
    /// Software error
    Software,
    
    /// Network error
    Network,
    
    /// Timeout error
    Timeout,
    
    /// Resource exhaustion
    ResourceExhaustion,
    
    /// Configuration error
    Configuration,
}

/// Main Neural Execution Engine
pub struct NeuralExecutionEngine {
    /// Configuration
    config: NeuralExecutionConfig,
    
    /// Neural router
    neural_router: Arc<NeuralRouter>,
    
    /// Atomic executor
    atomic_executor: Arc<AtomicExecutor>,
    
    /// Neural predictor
    neural_predictor: Arc<NeuralPredictor>,
    
    /// Hardware accelerator
    hardware_accelerator: Arc<HardwareAccelerator>,
    
    /// Execution monitor
    execution_monitor: Arc<ExecutionMonitor>,
    
    /// Performance metrics
    metrics: Arc<RwLock<NeuralExecutionMetrics>>,
    
    /// Running status
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Default)]
pub struct NeuralExecutionMetrics {
    /// Total executions
    pub total_executions: u64,
    
    /// Successful executions
    pub successful_executions: u64,
    
    /// Average execution time (μs)
    pub avg_execution_time_us: f64,
    
    /// 99th percentile latency (μs)
    pub p99_latency_us: f64,
    
    /// Throughput (ops/s)
    pub throughput_ops_s: f64,
    
    /// Hardware efficiency (%)
    pub hardware_efficiency_percent: f64,
    
    /// Error rate (%)
    pub error_rate_percent: f64,
    
    /// Last update timestamp
    pub last_update_ns: u64,
}

impl NeuralExecutionEngine {
    /// Create new Neural Execution Engine
    pub async fn new(config: NeuralExecutionConfig) -> Result<Self> {
        info!("🧠 Initializing Neural Execution Engine");
        
        // Initialize components
        let neural_router = Arc::new(
            NeuralRouter::new(config.neural_router.clone()).await?
        );
        
        let atomic_executor = Arc::new(
            AtomicExecutor::new(config.atomic_executor.clone()).await?
        );
        
        let neural_predictor = Arc::new(
            NeuralPredictor::new(config.neural_predictor.clone()).await?
        );
        
        let hardware_accelerator = Arc::new(
            HardwareAccelerator::new(config.hardware_accelerator.clone()).await?
        );
        
        let execution_monitor = Arc::new(
            ExecutionMonitor::new(config.execution_monitor.clone()).await?
        );
        
        info!("✅ Neural Execution Engine initialized");
        
        Ok(Self {
            config,
            neural_router,
            atomic_executor,
            neural_predictor,
            hardware_accelerator,
            execution_monitor,
            metrics: Arc::new(RwLock::new(NeuralExecutionMetrics::default())),
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the execution engine
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Neural Execution Engine");
        
        // Start all components
        self.neural_router.start().await?;
        self.atomic_executor.start().await?;
        self.neural_predictor.start().await?;
        self.hardware_accelerator.start().await?;
        self.execution_monitor.start().await?;
        
        *self.running.write().await = true;
        
        // Start background tasks
        self.start_metrics_collection().await;
        self.start_performance_optimization().await;
        
        info!("✅ Neural Execution Engine started");
        Ok(())
    }
    
    /// Stop the execution engine
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Neural Execution Engine");
        
        *self.running.write().await = false;
        
        // Stop all components
        self.execution_monitor.stop().await?;
        self.hardware_accelerator.stop().await?;
        self.neural_predictor.stop().await?;
        self.atomic_executor.stop().await?;
        self.neural_router.stop().await?;
        
        info!("✅ Neural Execution Engine stopped");
        Ok(())
    }
    
    /// Execute request with neural optimization
    pub async fn execute(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let start_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        
        debug!("🎯 Executing request: {} (type: {:?}, priority: {:?})", 
               request.id, request.request_type, request.priority);
        
        // Neural routing decision
        let routing_decision = self.neural_router.route_request(&request).await?;
        
        // Predict optimal execution timing
        let timing_prediction = self.neural_predictor.predict_timing(&request).await?;
        
        // Execute atomically
        let execution_result = self.atomic_executor.execute_atomic(
            request.clone(),
            routing_decision,
            timing_prediction,
        ).await?;
        
        // Monitor execution
        self.execution_monitor.record_execution(&request, &execution_result).await?;
        
        // Update metrics
        let execution_time_us = start_time.elapsed().as_micros() as f64;
        let mut metrics = self.metrics.write().await;
        metrics.total_executions += 1;
        
        if execution_result.status == ExecutionStatus::Success {
            metrics.successful_executions += 1;
        }
        
        metrics.avg_execution_time_us = 
            (metrics.avg_execution_time_us + execution_time_us) / 2.0;
        
        metrics.throughput_ops_s = if execution_time_us > 0.0 {
            1_000_000.0 / execution_time_us
        } else {
            0.0
        };
        
        metrics.error_rate_percent = 
            (1.0 - (metrics.successful_executions as f64 / metrics.total_executions as f64)) * 100.0;
        
        metrics.last_update_ns = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;
        
        debug!("✅ Execution completed: {} in {:.2}μs", request.id, execution_time_us);
        
        Ok(execution_result)
    }
    
    async fn start_metrics_collection(&self) {
        let metrics = Arc::clone(&self.metrics);
        let execution_monitor = Arc::clone(&self.execution_monitor);
        let running = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_micros(100)); // 100μs
            
            while *running.read().await {
                interval.tick().await;
                
                // Collect real-time metrics
                if let Ok(monitor_metrics) = execution_monitor.get_realtime_metrics().await {
                    let mut m = metrics.write().await;
                    m.hardware_efficiency_percent = monitor_metrics.hardware_efficiency;
                    m.p99_latency_us = monitor_metrics.p99_latency_us;
                }
            }
        });
    }
    
    async fn start_performance_optimization(&self) {
        let neural_router = Arc::clone(&self.neural_router);
        let neural_predictor = Arc::clone(&self.neural_predictor);
        let running = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100)); // 100ms
            
            while *running.read().await {
                interval.tick().await;
                
                // Optimize routing based on performance data
                if let Err(e) = neural_router.optimize_routes().await {
                    error!("Failed to optimize routes: {}", e);
                }
                
                // Update ML models
                if let Err(e) = neural_predictor.update_models().await {
                    error!("Failed to update ML models: {}", e);
                }
            }
        });
    }
    
    /// Get engine metrics
    pub async fn get_metrics(&self) -> NeuralExecutionMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Health check
    pub async fn health_check(&self) -> Result<NeuralExecutionHealth> {
        let router_health = self.neural_router.health_check().await?;
        let executor_health = self.atomic_executor.health_check().await?;
        let predictor_health = self.neural_predictor.health_check().await?;
        let accelerator_health = self.hardware_accelerator.health_check().await?;
        let monitor_health = self.execution_monitor.health_check().await?;
        
        Ok(NeuralExecutionHealth {
            neural_router: router_health,
            atomic_executor: executor_health,
            neural_predictor: predictor_health,
            hardware_accelerator: accelerator_health,
            execution_monitor: monitor_health,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralExecutionHealth {
    pub neural_router: ComponentHealth,
    pub atomic_executor: ComponentHealth,
    pub neural_predictor: ComponentHealth,
    pub hardware_accelerator: ComponentHealth,
    pub execution_monitor: ComponentHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub latency_us: f64,
    pub error_rate: f64,
    pub last_check_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
