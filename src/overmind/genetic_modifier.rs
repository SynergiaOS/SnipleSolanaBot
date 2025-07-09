//! Genetic Configuration Modifier for FAZA 11
//! 
//! Atomic configuration mutations with percentage-based deltas,
//! validation, and safe rollback mechanisms

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug, warn, error};

use super::evolution::{ConfigMutationPlan, ConfigMutation, MutationType};

/// Configuration modifier with atomic operations
pub struct GeneticConfigModifier {
    /// Backup configurations for rollback
    backups: RwLock<HashMap<Uuid, ConfigBackup>>,
    
    /// Validation rules
    validation_rules: ValidationRules,
    
    /// Mutation statistics
    mutation_stats: RwLock<MutationStats>,
}

/// Configuration backup for rollback
#[derive(Debug, Clone)]
pub struct ConfigBackup {
    pub candidate_id: Uuid,
    pub original_config: String,
    pub backup_path: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Validation rules for mutations
#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub max_delta_percentage: f64,
    pub allowed_config_sections: Vec<String>,
    pub forbidden_keys: Vec<String>,
    pub min_values: HashMap<String, f64>,
    pub max_values: HashMap<String, f64>,
}

/// Mutation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStats {
    pub total_mutations: u64,
    pub successful_mutations: u64,
    pub failed_mutations: u64,
    pub rollbacks_performed: u64,
    pub avg_performance_change: f64,
}

/// Mutation result
#[derive(Debug, Clone)]
pub struct MutationResult {
    pub success: bool,
    pub changes_applied: Vec<String>,
    pub validation_errors: Vec<String>,
    pub backup_created: bool,
    pub config_hash_before: String,
    pub config_hash_after: String,
}

impl Default for ValidationRules {
    fn default() -> Self {
        let mut min_values = HashMap::new();
        let mut max_values = HashMap::new();
        
        // Default safe ranges
        min_values.insert("risk_thresholds.max_drawdown".to_string(), 0.01);
        max_values.insert("risk_thresholds.max_drawdown".to_string(), 0.50);
        
        min_values.insert("hft_params.aggression".to_string(), 0.1);
        max_values.insert("hft_params.aggression".to_string(), 2.0);
        
        Self {
            max_delta_percentage: 0.25, // Max 25% change
            allowed_config_sections: vec![
                "risk_thresholds".to_string(),
                "hft_params".to_string(),
                "trading_strategy".to_string(),
                "position_sizing".to_string(),
            ],
            forbidden_keys: vec![
                "api_keys".to_string(),
                "private_keys".to_string(),
                "wallet_addresses".to_string(),
            ],
            min_values,
            max_values,
        }
    }
}

impl Default for MutationStats {
    fn default() -> Self {
        Self {
            total_mutations: 0,
            successful_mutations: 0,
            failed_mutations: 0,
            rollbacks_performed: 0,
            avg_performance_change: 0.0,
        }
    }
}

impl GeneticConfigModifier {
    /// Create new genetic configuration modifier
    pub fn new() -> Self {
        Self {
            backups: RwLock::new(HashMap::new()),
            validation_rules: ValidationRules::default(),
            mutation_stats: RwLock::new(MutationStats::default()),
        }
    }
    
    /// Apply evolution plan to configuration
    pub async fn apply_evolution(
        &self,
        config_path: &Path,
        plan: &ConfigMutationPlan,
    ) -> Result<MutationResult> {
        info!("🧬 Applying evolution plan to {}", config_path.display());
        
        // Create backup first
        let backup = self.create_backup(config_path, plan.target_candidate).await?;
        
        // Read current configuration
        let original_config = fs::read_to_string(config_path)?;
        let config_hash_before = self.calculate_hash(&original_config);
        
        // Validate plan
        let validation_errors = self.validate_plan(plan).await?;
        if !validation_errors.is_empty() {
            warn!("⚠️ Validation errors found: {:?}", validation_errors);
            return Ok(MutationResult {
                success: false,
                changes_applied: Vec::new(),
                validation_errors,
                backup_created: true,
                config_hash_before: config_hash_before.clone(),
                config_hash_after: config_hash_before,
            });
        }
        
        // Parse configuration
        let mut config: toml::Value = toml::from_str(&original_config)?;
        let mut changes_applied = Vec::new();
        
        // Apply mutations
        for mutation in &plan.mutations {
            match self.apply_mutation(&mut config, mutation).await {
                Ok(change_description) => {
                    changes_applied.push(change_description);
                    debug!("✅ Applied mutation: {}", mutation.target);
                }
                Err(e) => {
                    error!("❌ Failed to apply mutation {}: {}", mutation.target, e);
                    // Rollback on any failure
                    self.rollback_configuration(plan.target_candidate).await?;
                    return Ok(MutationResult {
                        success: false,
                        changes_applied,
                        validation_errors: vec![format!("Mutation failed: {}", e)],
                        backup_created: true,
                        config_hash_before: config_hash_before.clone(),
                        config_hash_after: config_hash_before,
                    });
                }
            }
        }
        
        // Serialize modified configuration
        let new_config = toml::to_string_pretty(&config)?;
        let config_hash_after = self.calculate_hash(&new_config);
        
        // Atomic write
        self.atomic_write(config_path, &new_config).await?;
        
        // Update statistics
        self.update_stats(true).await;
        
        info!("✅ Evolution plan applied successfully with {} changes", changes_applied.len());
        
        Ok(MutationResult {
            success: true,
            changes_applied,
            validation_errors: Vec::new(),
            backup_created: true,
            config_hash_before,
            config_hash_after,
        })
    }
    
    /// Create backup of configuration
    async fn create_backup(&self, config_path: &Path, candidate_id: Uuid) -> Result<ConfigBackup> {
        let original_config = fs::read_to_string(config_path)?;
        let backup_dir = format!("/var/swarm/{}/backups", candidate_id);
        let backup_filename = format!("config_backup_{}.toml", chrono::Utc::now().timestamp());
        let backup_path = PathBuf::from(backup_dir).join(backup_filename);
        
        // Create backup directory
        fs::create_dir_all(backup_path.parent().unwrap())?;
        
        // Write backup
        fs::write(&backup_path, &original_config)?;
        
        let backup = ConfigBackup {
            candidate_id,
            original_config,
            backup_path: backup_path.clone(),
            timestamp: chrono::Utc::now(),
        };
        
        // Store backup reference
        {
            let mut backups = self.backups.write().await;
            backups.insert(candidate_id, backup.clone());
        }
        
        debug!("💾 Backup created: {}", backup_path.display());
        Ok(backup)
    }
    
    /// Validate mutation plan
    async fn validate_plan(&self, plan: &ConfigMutationPlan) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        
        for mutation in &plan.mutations {
            // Check if section is allowed
            let section = mutation.target.split('.').next().unwrap_or("");
            if !self.validation_rules.allowed_config_sections.contains(&section.to_string()) {
                errors.push(format!("Section '{}' not allowed for mutation", section));
            }
            
            // Check forbidden keys
            if self.validation_rules.forbidden_keys.iter().any(|key| mutation.target.contains(key)) {
                errors.push(format!("Target '{}' contains forbidden key", mutation.target));
            }
            
            // Check delta percentage
            if mutation.delta.abs() > self.validation_rules.max_delta_percentage {
                errors.push(format!(
                    "Delta {:.2}% exceeds maximum allowed {:.2}%",
                    mutation.delta * 100.0,
                    self.validation_rules.max_delta_percentage * 100.0
                ));
            }
        }
        
        Ok(errors)
    }
    
    /// Apply single mutation to configuration
    async fn apply_mutation(
        &self,
        config: &mut toml::Value,
        mutation: &ConfigMutation,
    ) -> Result<String> {
        let path_parts: Vec<&str> = mutation.target.split('.').collect();
        
        // Navigate to the target value
        let mut current = config;
        for (i, part) in path_parts.iter().enumerate() {
            if i == path_parts.len() - 1 {
                // Last part - apply mutation
                return self.apply_value_mutation(current, part, mutation).await;
            } else {
                // Navigate deeper
                current = current
                    .get_mut(part)
                    .ok_or_else(|| anyhow!("Path '{}' not found in configuration", part))?;
            }
        }
        
        Err(anyhow!("Invalid mutation path: {}", mutation.target))
    }
    
    /// Apply mutation to specific value
    async fn apply_value_mutation(
        &self,
        parent: &mut toml::Value,
        key: &str,
        mutation: &ConfigMutation,
    ) -> Result<String> {
        let current_value = parent
            .get(key)
            .ok_or_else(|| anyhow!("Key '{}' not found", key))?;
        
        let new_value = match (&mutation.mutation_type, current_value) {
            (MutationType::Increase, toml::Value::Float(val)) => {
                let new_val = val * (1.0 + mutation.delta);
                self.validate_range(&mutation.target, new_val)?;
                toml::Value::Float(new_val)
            }
            (MutationType::Decrease, toml::Value::Float(val)) => {
                let new_val = val * (1.0 - mutation.delta.abs());
                self.validate_range(&mutation.target, new_val)?;
                toml::Value::Float(new_val)
            }
            (MutationType::Replace, _) => {
                toml::Value::Float(mutation.delta)
            }
            (MutationType::Toggle, toml::Value::Boolean(val)) => {
                toml::Value::Boolean(!val)
            }
            _ => return Err(anyhow!("Incompatible mutation type for value type")),
        };
        
        // Apply the change
        if let Some(table) = parent.as_table_mut() {
            let old_val = table.insert(key.to_string(), new_value.clone());
            return Ok(format!(
                "Changed {}: {:?} -> {:?}",
                mutation.target, old_val, new_value
            ));
        }
        
        Err(anyhow!("Parent is not a table"))
    }
    
    /// Validate value is within allowed range
    fn validate_range(&self, target: &str, value: f64) -> Result<()> {
        if let Some(&min_val) = self.validation_rules.min_values.get(target) {
            if value < min_val {
                return Err(anyhow!("Value {} below minimum {} for {}", value, min_val, target));
            }
        }
        
        if let Some(&max_val) = self.validation_rules.max_values.get(target) {
            if value > max_val {
                return Err(anyhow!("Value {} above maximum {} for {}", value, max_val, target));
            }
        }
        
        Ok(())
    }
    
    /// Atomic write with temporary file
    async fn atomic_write(&self, path: &Path, content: &str) -> Result<()> {
        let temp_path = path.with_extension("tmp");
        
        // Write to temporary file
        fs::write(&temp_path, content)?;
        
        // Atomic rename
        fs::rename(&temp_path, path)?;
        
        debug!("💾 Atomic write completed: {}", path.display());
        Ok(())
    }
    
    /// Calculate configuration hash
    fn calculate_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Rollback configuration to backup
    pub async fn rollback_configuration(&self, candidate_id: Uuid) -> Result<()> {
        let backup = {
            let backups = self.backups.read().await;
            backups.get(&candidate_id).cloned()
        };
        
        if let Some(backup) = backup {
            // Find original config path (assuming it's in the same directory as backup)
            let config_path = backup.backup_path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("config.toml");
            
            // Restore from backup
            fs::write(&config_path, &backup.original_config)?;
            
            // Update statistics
            self.update_stats_rollback().await;
            
            info!("🔄 Configuration rolled back for candidate {}", candidate_id);
            Ok(())
        } else {
            Err(anyhow!("No backup found for candidate {}", candidate_id))
        }
    }
    
    /// Update mutation statistics
    async fn update_stats(&self, success: bool) {
        let mut stats = self.mutation_stats.write().await;
        stats.total_mutations += 1;
        if success {
            stats.successful_mutations += 1;
        } else {
            stats.failed_mutations += 1;
        }
    }
    
    /// Update rollback statistics
    async fn update_stats_rollback(&self) {
        let mut stats = self.mutation_stats.write().await;
        stats.rollbacks_performed += 1;
    }
    
    /// Get mutation statistics
    pub async fn get_stats(&self) -> MutationStats {
        self.mutation_stats.read().await.clone()
    }
}
