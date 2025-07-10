//! STRATEGY COMPILER - DSL to Native Code
//! 
//! Kompiluje DSL strategii do natywnych bibliotek (.so)
//! CI/CD pipeline z TensorZero observability

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use sha2::{Sha256, Digest};
use tempfile::TempDir;

use super::dsl_generator::StrategyDSL;
use super::CompiledArtifact;

/// Strategy Compiler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// Compiler executable path
    pub compiler_path: String,
    
    /// Target architecture
    pub target_arch: String,
    
    /// Optimization level
    pub optimization_level: String,
    
    /// Output directory
    pub output_dir: String,
    
    /// Artifact storage (S3, local, etc.)
    pub artifact_storage: ArtifactStorageConfig,
    
    /// Compilation timeout
    pub timeout_seconds: u64,
    
    /// Enable debug symbols
    pub debug_symbols: bool,
    
    /// Enable LTO (Link Time Optimization)
    pub enable_lto: bool,
    
    /// Target CPU features
    pub cpu_features: Vec<String>,
}

/// Artifact storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStorageConfig {
    pub storage_type: StorageType,
    pub bucket_name: Option<String>,
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub local_path: Option<String>,
}

/// Storage type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    Local,
    S3,
    GCS,
    Azure,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            compiler_path: "rustc".to_string(),
            target_arch: "x86_64-unknown-linux-gnu".to_string(),
            optimization_level: "3".to_string(),
            output_dir: "./artifacts".to_string(),
            artifact_storage: ArtifactStorageConfig {
                storage_type: StorageType::Local,
                bucket_name: None,
                region: None,
                access_key: None,
                secret_key: None,
                local_path: Some("./artifacts".to_string()),
            },
            timeout_seconds: 300, // 5 minutes
            debug_symbols: false,
            enable_lto: true,
            cpu_features: vec![
                "avx2".to_string(),
                "fma".to_string(),
                "sse4.2".to_string(),
            ],
        }
    }
}

/// Strategy Compiler
#[derive(Debug)]
pub struct StrategyCompiler {
    /// Configuration
    config: CompilerConfig,
    
    /// Compilation statistics
    stats: CompilerStats,
    
    /// Template directory
    template_dir: PathBuf,
}

/// Compilation statistics
#[derive(Debug, Default, Clone)]
pub struct CompilerStats {
    pub total_compilations: u64,
    pub successful_compilations: u64,
    pub failed_compilations: u64,
    pub average_compilation_time_ms: u64,
    pub total_artifacts_size_bytes: u64,
    pub cache_hits: u64,
}

/// Compilation result
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub success: bool,
    pub artifact: Option<CompiledArtifact>,
    pub compilation_time: Duration,
    pub output_log: String,
    pub error_log: String,
    pub warnings: Vec<String>,
}

impl StrategyCompiler {
    /// Create new strategy compiler
    pub fn new(config: CompilerConfig) -> Result<Self> {
        info!("⚙️ Initializing Strategy Compiler");
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(&config.output_dir)
            .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;
        
        // Verify compiler exists
        let compiler_check = Command::new(&config.compiler_path)
            .arg("--version")
            .output();
        
        match compiler_check {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                info!("✅ Compiler found: {}", version.trim());
            }
            Ok(_) => {
                warn!("⚠️ Compiler found but version check failed");
            }
            Err(e) => {
                error!("❌ Compiler not found: {}", e);
                return Err(anyhow!("Compiler not accessible: {}", e));
            }
        }
        
        let template_dir = PathBuf::from("src/forge/templates");
        
        Ok(Self {
            config,
            stats: CompilerStats::default(),
            template_dir,
        })
    }
    
    /// Compile strategy DSL to native library
    pub async fn compile(&mut self, dsl: &StrategyDSL, agent_id: &str) -> Result<CompiledArtifact> {
        let start_time = Instant::now();
        info!("🔨 Compiling strategy: {} for agent: {}", dsl.name, agent_id);
        
        // Check compilation cache first
        if let Some(cached_artifact) = self.check_compilation_cache(dsl).await? {
            info!("💾 Using cached compilation for strategy: {}", dsl.name);
            self.stats.cache_hits += 1;
            return Ok(cached_artifact);
        }
        
        // Create temporary compilation directory
        let temp_dir = TempDir::new()
            .map_err(|e| anyhow!("Failed to create temp directory: {}", e))?;
        
        // Generate Rust source code from DSL
        let rust_source = self.generate_rust_source(dsl, agent_id)?;
        
        // Write source files
        let source_path = temp_dir.path().join("strategy.rs");
        fs::write(&source_path, rust_source)
            .map_err(|e| anyhow!("Failed to write source file: {}", e))?;
        
        // Copy template files
        self.copy_template_files(temp_dir.path())?;
        
        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml(dsl, agent_id)?;
        let cargo_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_path, cargo_toml)
            .map_err(|e| anyhow!("Failed to write Cargo.toml: {}", e))?;
        
        // Compile to shared library
        let compilation_result = self.compile_to_shared_library(temp_dir.path(), dsl, agent_id).await?;
        
        if !compilation_result.success {
            self.stats.failed_compilations += 1;
            return Err(anyhow!("Compilation failed: {}", compilation_result.error_log));
        }
        
        let artifact = compilation_result.artifact
            .ok_or_else(|| anyhow!("Compilation succeeded but no artifact produced"))?;
        
        // Store artifact
        self.store_artifact(&artifact).await?;
        
        // Update statistics
        let compilation_time = start_time.elapsed().as_millis() as u64;
        self.update_compilation_stats(compilation_time, true, &artifact);
        
        info!("✅ Compilation completed: {} in {}ms", artifact.strategy_id, compilation_time);
        Ok(artifact)
    }
    
    /// Check compilation cache
    async fn check_compilation_cache(&self, dsl: &StrategyDSL) -> Result<Option<CompiledArtifact>> {
        // Calculate DSL hash for cache key
        let dsl_hash = self.calculate_dsl_hash(dsl);
        
        // Check if artifact exists in storage
        let cache_path = format!("{}/cache/{}.so", self.config.output_dir, dsl_hash);
        
        if Path::new(&cache_path).exists() {
            // Load cached artifact metadata
            let metadata_path = format!("{}/cache/{}.json", self.config.output_dir, dsl_hash);
            if let Ok(metadata_content) = fs::read_to_string(&metadata_path) {
                if let Ok(artifact) = serde_json::from_str::<CompiledArtifact>(&metadata_content) {
                    return Ok(Some(artifact));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Generate Rust source code from DSL
    fn generate_rust_source(&self, dsl: &StrategyDSL, agent_id: &str) -> Result<String> {
        let template = include_str!("templates/strategy_template.rs");
        
        // Replace template placeholders with DSL data
        let source = template
            .replace("{{STRATEGY_NAME}}", &dsl.name)
            .replace("{{STRATEGY_ID}}", &dsl.strategy_id)
            .replace("{{AGENT_ID}}", agent_id)
            .replace("{{MAX_DRAWDOWN}}", &dsl.risk_model.max_drawdown.to_string())
            .replace("{{DAILY_LOSS_LIMIT}}", &dsl.risk_model.daily_loss_limit.to_string())
            .replace("{{POSITION_SIZE}}", &dsl.risk_model.position_size.to_string())
            .replace("{{ENTRY_LOGIC}}", &self.generate_entry_logic_code(&dsl.entry_logic)?)
            .replace("{{EXIT_LOGIC}}", &self.generate_exit_logic_code(&dsl.exit_logic)?)
            .replace("{{AI_MODELS}}", &self.generate_ai_models_code(&dsl.ai_models)?);
        
        Ok(source)
    }
    
    /// Generate entry logic code
    fn generate_entry_logic_code(&self, entry_logic: &[super::dsl_generator::TradingRule]) -> Result<String> {
        let mut code = String::new();
        
        for rule in entry_logic {
            code.push_str(&format!(
                "    // Rule: {}\n    if {} {{\n        {};\n    }}\n",
                rule.trigger,
                self.convert_trigger_to_rust(&rule.trigger)?,
                self.convert_action_to_rust(&rule.action)?
            ));
        }
        
        Ok(code)
    }
    
    /// Generate exit logic code
    fn generate_exit_logic_code(&self, exit_logic: &[super::dsl_generator::TradingRule]) -> Result<String> {
        let mut code = String::new();
        
        for rule in exit_logic {
            code.push_str(&format!(
                "    // Rule: {}\n    if {} {{\n        {};\n    }}\n",
                rule.trigger,
                self.convert_trigger_to_rust(&rule.trigger)?,
                self.convert_action_to_rust(&rule.action)?
            ));
        }
        
        Ok(code)
    }
    
    /// Generate AI models code
    fn generate_ai_models_code(&self, ai_models: &[super::dsl_generator::AIModelDef]) -> Result<String> {
        let mut code = String::new();
        
        for model in ai_models {
            code.push_str(&format!(
                "    // AI Model: {} v{}\n    // Purpose: {}\n",
                model.name, model.version, model.purpose
            ));
        }
        
        Ok(code)
    }
    
    /// Convert DSL trigger to Rust code
    fn convert_trigger_to_rust(&self, trigger: &str) -> Result<String> {
        // Simple conversion - in production would use proper DSL parser
        let rust_code = trigger
            .replace("momentum_signal > 0.7", "market_data.momentum_signal > 0.7")
            .replace("profit > 2%", "position.unrealized_pnl > position.size * 0.02")
            .replace("loss > 1%", "position.unrealized_pnl < -position.size * 0.01")
            .replace(" OR ", " || ")
            .replace(" AND ", " && ");
        
        Ok(rust_code)
    }
    
    /// Convert DSL action to Rust code
    fn convert_action_to_rust(&self, action: &str) -> Result<String> {
        let rust_code = match action {
            "market_buy" => "executor.market_buy(position_size)",
            "market_sell" => "executor.market_sell(position_size)",
            "limit_buy" => "executor.limit_buy(position_size, limit_price)",
            "limit_sell" => "executor.limit_sell(position_size, limit_price)",
            _ => "// Unknown action",
        };
        
        Ok(rust_code.to_string())
    }
    
    /// Generate Cargo.toml for compilation
    fn generate_cargo_toml(&self, dsl: &StrategyDSL, agent_id: &str) -> Result<String> {
        let cargo_toml = format!(r#"
[package]
name = "strategy_{}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}

[profile.release]
opt-level = {}
lto = {}
codegen-units = 1
panic = "abort"
strip = true

[target.{}.rustflags]
rustflags = [
    "-C", "target-cpu=native",
    "-C", "target-feature=+{}",
]
"#,
            agent_id,
            self.config.optimization_level,
            self.config.enable_lto,
            self.config.target_arch,
            self.config.cpu_features.join(",+")
        );
        
        Ok(cargo_toml)
    }
    
    /// Copy template files to compilation directory
    fn copy_template_files(&self, target_dir: &Path) -> Result<()> {
        if self.template_dir.exists() {
            for entry in fs::read_dir(&self.template_dir)? {
                let entry = entry?;
                let file_name = entry.file_name();
                
                if file_name.to_string_lossy().ends_with(".rs") {
                    let source = entry.path();
                    let target = target_dir.join(&file_name);
                    fs::copy(&source, &target)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Compile to shared library
    async fn compile_to_shared_library(
        &self,
        source_dir: &Path,
        dsl: &StrategyDSL,
        agent_id: &str,
    ) -> Result<CompilationResult> {
        let start_time = Instant::now();
        
        // Build command
        let mut cmd = Command::new("cargo");
        cmd.arg("build")
           .arg("--release")
           .arg("--target")
           .arg(&self.config.target_arch)
           .current_dir(source_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Set environment variables
        cmd.env("RUSTFLAGS", format!("-C target-cpu=native -C target-feature=+{}", 
                                    self.config.cpu_features.join(",+")));
        
        // Execute compilation with timeout
        let output = tokio::time::timeout(
            Duration::from_secs(self.config.timeout_seconds),
            tokio::task::spawn_blocking(move || cmd.output())
        ).await
        .map_err(|_| anyhow!("Compilation timeout"))?
        .map_err(|e| anyhow!("Failed to spawn compilation process: {}", e))?
        .map_err(|e| anyhow!("Compilation process failed: {}", e))?;
        
        let compilation_time = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if output.status.success() {
            // Find compiled library
            let lib_name = format!("libstrategy_{}.so", agent_id);
            let lib_path = source_dir
                .join("target")
                .join(&self.config.target_arch)
                .join("release")
                .join(&lib_name);
            
            if lib_path.exists() {
                // Calculate checksum
                let lib_content = fs::read(&lib_path)?;
                let checksum = format!("{:x}", Sha256::digest(&lib_content));
                
                // Create final artifact path
                let artifact_name = format!("{}_{}.so", dsl.strategy_id, checksum[..8].to_string());
                let final_path = Path::new(&self.config.output_dir).join(&artifact_name);
                
                // Copy to final location
                fs::copy(&lib_path, &final_path)?;
                
                let artifact = CompiledArtifact {
                    strategy_id: dsl.strategy_id.clone(),
                    binary_path: final_path.to_string_lossy().to_string(),
                    checksum,
                    compilation_time,
                    optimization_level: self.config.optimization_level.clone(),
                };
                
                Ok(CompilationResult {
                    success: true,
                    artifact: Some(artifact),
                    compilation_time,
                    output_log: stdout,
                    error_log: stderr.clone(),
                    warnings: self.extract_warnings(&stderr),
                })
            } else {
                Ok(CompilationResult {
                    success: false,
                    artifact: None,
                    compilation_time,
                    output_log: stdout,
                    error_log: format!("Library not found at expected path: {}", lib_path.display()),
                    warnings: vec![],
                })
            }
        } else {
            Ok(CompilationResult {
                success: false,
                artifact: None,
                compilation_time,
                output_log: stdout,
                error_log: stderr,
                warnings: vec![],
            })
        }
    }
    
    /// Extract warnings from compiler output
    fn extract_warnings(&self, stderr: &str) -> Vec<String> {
        stderr.lines()
            .filter(|line| line.contains("warning:"))
            .map(|line| line.to_string())
            .collect()
    }
    
    /// Store artifact in configured storage
    async fn store_artifact(&self, artifact: &CompiledArtifact) -> Result<()> {
        match self.config.artifact_storage.storage_type {
            StorageType::Local => {
                // Already stored locally during compilation
                Ok(())
            }
            StorageType::S3 => {
                // TODO: Implement S3 storage
                warn!("S3 storage not yet implemented");
                Ok(())
            }
            _ => {
                warn!("Storage type not yet implemented: {:?}", self.config.artifact_storage.storage_type);
                Ok(())
            }
        }
    }
    
    /// Calculate DSL hash for caching
    fn calculate_dsl_hash(&self, dsl: &StrategyDSL) -> String {
        let mut hasher = Sha256::new();
        hasher.update(dsl.source_code.as_bytes());
        hasher.update(serde_json::to_string(&dsl.risk_model).unwrap_or_default().as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }
    
    /// Update compilation statistics
    fn update_compilation_stats(&mut self, compilation_time_ms: u64, success: bool, artifact: &CompiledArtifact) {
        self.stats.total_compilations += 1;
        
        if success {
            self.stats.successful_compilations += 1;
            
            // Update artifact size
            if let Ok(metadata) = fs::metadata(&artifact.binary_path) {
                self.stats.total_artifacts_size_bytes += metadata.len();
            }
        } else {
            self.stats.failed_compilations += 1;
        }
        
        // Update average compilation time
        let total = self.stats.total_compilations;
        self.stats.average_compilation_time_ms = 
            (self.stats.average_compilation_time_ms * (total - 1) + compilation_time_ms) / total;
    }
    
    /// Get compilation statistics
    pub fn get_stats(&self) -> &CompilerStats {
        &self.stats
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &CompilerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trigger_conversion() {
        let compiler = StrategyCompiler::new(CompilerConfig::default()).unwrap();
        
        let trigger = "momentum_signal > 0.7 AND profit > 2%";
        let rust_code = compiler.convert_trigger_to_rust(trigger).unwrap();
        
        assert!(rust_code.contains("market_data.momentum_signal > 0.7"));
        assert!(rust_code.contains("&&"));
    }
    
    #[test]
    fn test_action_conversion() {
        let compiler = StrategyCompiler::new(CompilerConfig::default()).unwrap();
        
        let action = "market_buy";
        let rust_code = compiler.convert_action_to_rust(action).unwrap();
        
        assert_eq!(rust_code, "executor.market_buy(position_size)");
    }
}
