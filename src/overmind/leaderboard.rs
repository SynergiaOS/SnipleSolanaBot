//! SwarmLeaderboard Implementation for FAZA 11
//! 
//! Real-time ranking system with MetricsStore, snapshot functionality,
//! percentile analysis and performance tracking

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, debug};
use std::time::Duration;

use super::swarm::{AgentMetrics, SystemCandidate};

/// Metrics storage and persistence layer
#[derive(Debug, Clone)]
pub struct MetricsStore {
    /// In-memory metrics cache
    metrics_cache: HashMap<Uuid, Vec<AgentMetrics>>,
    
    /// Performance snapshots
    snapshots: Vec<LeaderboardSnapshot>,
    
    /// Configuration
    max_history_entries: usize,
    snapshot_interval: Duration,
}

/// Leaderboard snapshot for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub candidates: Vec<CandidateRanking>,
    pub summary_stats: SummaryStats,
}

/// Individual candidate ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRanking {
    pub candidate_id: Uuid,
    pub rank: usize,
    pub hotz_score: f64,
    pub roi: f64,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub performance_tier: PerformanceTier,
}

/// Performance tier classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceTier {
    Elite,      // Top 10%
    Strong,     // Top 25%
    Average,    // Middle 50%
    Weak,       // Bottom 25%
    Critical,   // Bottom 10%
}

/// Summary statistics for the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_candidates: usize,
    pub avg_hotz_score: f64,
    pub avg_roi: f64,
    pub avg_win_rate: f64,
    pub top_performer_id: Option<Uuid>,
    pub worst_performer_id: Option<Uuid>,
}

/// Percentile analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileAnalysis {
    pub elite_candidates: Vec<Uuid>,      // Top 10%
    pub underperformers: Vec<Uuid>,       // Bottom 10%
    pub p90_hotz_score: f64,
    pub p10_hotz_score: f64,
    pub median_hotz_score: f64,
}

impl MetricsStore {
    /// Create new MetricsStore
    pub fn new() -> Self {
        Self {
            metrics_cache: HashMap::new(),
            snapshots: Vec::new(),
            max_history_entries: 1000,
            snapshot_interval: Duration::from_secs(300), // 5 minutes
        }
    }
    
    /// Write metric to store
    pub async fn write_metric(&mut self, metric: AgentMetrics) -> Result<()> {
        let candidate_id = metric.agent_id;
        
        // Add to cache
        let history = self.metrics_cache.entry(candidate_id).or_insert_with(Vec::new);
        history.push(metric);
        
        // Limit history size
        if history.len() > self.max_history_entries {
            history.remove(0);
        }
        
        debug!("📊 Metric written for candidate {}", candidate_id);
        Ok(())
    }
    
    /// Read historical metrics for candidate
    pub async fn read_historical(&self, candidate_id: Uuid) -> Vec<AgentMetrics> {
        self.metrics_cache.get(&candidate_id).cloned().unwrap_or_default()
    }
    
    /// Create snapshot of current state
    pub async fn create_snapshot(&mut self, candidates: &[SystemCandidate]) -> Result<LeaderboardSnapshot> {
        let mut rankings = Vec::new();
        
        // Create rankings
        for (rank, candidate) in candidates.iter().enumerate() {
            let performance_tier = Self::classify_performance_tier(rank, candidates.len());
            
            rankings.push(CandidateRanking {
                candidate_id: candidate.id,
                rank: rank + 1,
                hotz_score: candidate.performance_metrics.hotz_score,
                roi: candidate.performance_metrics.roi,
                win_rate: candidate.performance_metrics.win_rate,
                sharpe_ratio: candidate.performance_metrics.sharpe_ratio,
                performance_tier,
            });
        }
        
        // Calculate summary stats
        let summary_stats = Self::calculate_summary_stats(&rankings);
        
        let snapshot = LeaderboardSnapshot {
            timestamp: chrono::Utc::now(),
            candidates: rankings,
            summary_stats,
        };
        
        // Store snapshot
        self.snapshots.push(snapshot.clone());
        
        // Limit snapshot history
        if self.snapshots.len() > 100 {
            self.snapshots.remove(0);
        }
        
        info!("📸 Leaderboard snapshot created with {} candidates", candidates.len());
        Ok(snapshot)
    }
    
    /// Classify performance tier based on rank
    fn classify_performance_tier(rank: usize, total: usize) -> PerformanceTier {
        let percentile = (rank as f64 / total as f64) * 100.0;
        
        match percentile {
            p if p <= 10.0 => PerformanceTier::Elite,
            p if p <= 25.0 => PerformanceTier::Strong,
            p if p <= 75.0 => PerformanceTier::Average,
            p if p <= 90.0 => PerformanceTier::Weak,
            _ => PerformanceTier::Critical,
        }
    }
    
    /// Calculate summary statistics
    fn calculate_summary_stats(rankings: &[CandidateRanking]) -> SummaryStats {
        if rankings.is_empty() {
            return SummaryStats {
                total_candidates: 0,
                avg_hotz_score: 0.0,
                avg_roi: 0.0,
                avg_win_rate: 0.0,
                top_performer_id: None,
                worst_performer_id: None,
            };
        }
        
        let total = rankings.len();
        let avg_hotz_score = rankings.iter().map(|r| r.hotz_score).sum::<f64>() / total as f64;
        let avg_roi = rankings.iter().map(|r| r.roi).sum::<f64>() / total as f64;
        let avg_win_rate = rankings.iter().map(|r| r.win_rate).sum::<f64>() / total as f64;
        
        let top_performer_id = rankings.first().map(|r| r.candidate_id);
        let worst_performer_id = rankings.last().map(|r| r.candidate_id);
        
        SummaryStats {
            total_candidates: total,
            avg_hotz_score,
            avg_roi,
            avg_win_rate,
            top_performer_id,
            worst_performer_id,
        }
    }
}

/// SwarmLeaderboard - Main ranking and analysis system
pub struct SwarmLeaderboard {
    /// Metrics storage
    metrics_store: RwLock<MetricsStore>,
    
    /// Current candidates
    candidates: RwLock<HashMap<Uuid, SystemCandidate>>,
    
    /// Update frequency
    update_cycle: Duration,
    
    /// Snapshot frequency
    snapshot_frequency: usize,
    
    /// Current epoch
    current_epoch: RwLock<u64>,
}

impl SwarmLeaderboard {
    /// Create new SwarmLeaderboard
    pub fn new() -> Self {
        Self {
            metrics_store: RwLock::new(MetricsStore::new()),
            candidates: RwLock::new(HashMap::new()),
            update_cycle: Duration::from_secs(5), // Update every 5 seconds
            snapshot_frequency: 60, // Snapshot every 60 updates (5 minutes)
            current_epoch: RwLock::new(0),
        }
    }
    
    /// Add candidate to leaderboard
    pub async fn add_candidate(&self, candidate: SystemCandidate) -> Result<()> {
        let candidate_id = candidate.id;
        
        // Add to candidates
        {
            let mut candidates = self.candidates.write().await;
            candidates.insert(candidate_id, candidate);
        }
        
        info!("🏆 Candidate {} added to leaderboard", candidate_id);
        Ok(())
    }
    
    /// Update candidate metrics
    pub async fn update_metrics(&self, candidate_id: Uuid, metrics: AgentMetrics) -> Result<()> {
        // Update metrics store
        {
            let mut store = self.metrics_store.write().await;
            store.write_metric(metrics.clone()).await?;
        }

        // Update candidate
        {
            let mut candidates = self.candidates.write().await;
            if let Some(candidate) = candidates.get_mut(&candidate_id) {
                candidate.performance_metrics = metrics;
                candidate.calculate_hotz_score();
            }
        }

        debug!("📈 Metrics updated for candidate {}", candidate_id);
        Ok(())
    }
    
    /// Get current snapshot
    pub async fn current_snapshot(&self) -> Result<LeaderboardSnapshot> {
        let candidates = self.candidates.read().await;
        let mut candidate_list: Vec<SystemCandidate> = candidates.values().cloned().collect();
        
        // Sort by Hotz score (descending)
        candidate_list.sort_by(|a, b| {
            b.performance_metrics.hotz_score
                .partial_cmp(&a.performance_metrics.hotz_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let mut store = self.metrics_store.write().await;
        store.create_snapshot(&candidate_list).await
    }
    
    /// Perform percentile analysis
    pub async fn percentile_analysis(&self) -> Result<PercentileAnalysis> {
        let snapshot = self.current_snapshot().await?;
        let rankings = &snapshot.candidates;
        
        if rankings.is_empty() {
            return Ok(PercentileAnalysis {
                elite_candidates: Vec::new(),
                underperformers: Vec::new(),
                p90_hotz_score: 0.0,
                p10_hotz_score: 0.0,
                median_hotz_score: 0.0,
            });
        }
        
        let total = rankings.len();
        let elite_count = (total as f64 * 0.1).ceil() as usize;
        let underperformer_count = (total as f64 * 0.1).ceil() as usize;
        
        let elite_candidates: Vec<Uuid> = rankings
            .iter()
            .take(elite_count)
            .map(|r| r.candidate_id)
            .collect();
            
        let underperformers: Vec<Uuid> = rankings
            .iter()
            .rev()
            .take(underperformer_count)
            .map(|r| r.candidate_id)
            .collect();
        
        // Calculate percentiles
        let mut hotz_scores: Vec<f64> = rankings.iter().map(|r| r.hotz_score).collect();
        hotz_scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let p90_index = ((total as f64) * 0.9) as usize;
        let p10_index = ((total as f64) * 0.1) as usize;
        let median_index = total / 2;
        
        Ok(PercentileAnalysis {
            elite_candidates,
            underperformers,
            p90_hotz_score: hotz_scores.get(p90_index).copied().unwrap_or(0.0),
            p10_hotz_score: hotz_scores.get(p10_index).copied().unwrap_or(0.0),
            median_hotz_score: hotz_scores.get(median_index).copied().unwrap_or(0.0),
        })
    }
}
