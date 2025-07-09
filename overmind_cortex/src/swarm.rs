//! SwarmTopology - Topologia agentów w THE OVERMIND PROTOCOL v4.4
//! 
//! Implementacja minimalistycznej architektury swarm zgodnie z filozofią Hotza

use crate::{CortexResult, CortexError};
use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;

/// Identyfikator agenta w swarm
pub type AgentId = u32;

/// Status agenta w swarm
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    /// Agent aktywny i gotowy do pracy
    Active,
    /// Agent w trakcie wykonywania zadania
    Busy,
    /// Agent nieaktywny
    Inactive,
    /// Agent w stanie błędu
    Error(String),
}

/// Typ agenta zgodnie z dokumentem
#[derive(Debug, Clone, PartialEq)]
pub enum AgentType {
    /// Agent analizy sentymentu
    Sentiment,
    /// Agent oceny ryzyka
    Risk,
    /// Agent analizy płynności
    Liquidity,
    /// Agent wykonawczy
    Executor,
    /// Agent eksperymentalny
    Experimental,
}

/// Metryki wydajności agenta
#[derive(Debug, Clone)]
pub struct AgentMetrics {
    /// Liczba wykonanych zadań
    pub tasks_completed: u64,
    /// Średni czas wykonania (nanosekundy)
    pub avg_execution_ns: u64,
    /// Wskaźnik sukcesu (0.0 - 1.0)
    pub success_rate: f32,
    /// Ostatnia aktywność
    pub last_activity: std::time::Instant,
}

impl AgentMetrics {
    pub fn new() -> Self {
        Self {
            tasks_completed: 0,
            avg_execution_ns: 0,
            success_rate: 1.0,
            last_activity: std::time::Instant::now(),
        }
    }

    /// Aktualizacja metryk po wykonaniu zadania
    pub fn update_task(&mut self, execution_ns: u64, success: bool) {
        self.tasks_completed += 1;
        self.avg_execution_ns = (self.avg_execution_ns + execution_ns) / 2;
        
        // Aktualizacja success rate z wagą dla ostatnich zadań
        let weight = 0.1; // 10% wagi dla nowego wyniku
        if success {
            self.success_rate = self.success_rate * (1.0 - weight) + weight;
        } else {
            self.success_rate = self.success_rate * (1.0 - weight);
        }
        
        self.last_activity = std::time::Instant::now();
    }
}

/// Definicja agenta w swarm
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    /// Unikalny identyfikator
    pub id: AgentId,
    /// Typ agenta
    pub agent_type: AgentType,
    /// Aktualny status
    pub status: AgentStatus,
    /// Metryki wydajności
    pub metrics: AgentMetrics,
    /// Konfiguracja specyficzna dla typu
    pub config: AgentConfig,
}

/// Konfiguracja agenta
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Maksymalny czas wykonania zadania (ms)
    pub max_execution_ms: u64,
    /// Priorytet agenta (0-255)
    pub priority: u8,
    /// Czy agent może być używany równolegle
    pub parallel_capable: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_execution_ms: 5000, // 5 sekund
            priority: 128,           // Średni priorytet
            parallel_capable: true,
        }
    }
}

/// Topologia swarm agentów
pub struct SwarmTopology {
    /// Mapa agentów według ID
    agents: HashMap<AgentId, AgentDefinition>,
    /// Następny dostępny ID agenta
    next_agent_id: AgentId,
    /// Statystyki swarm
    swarm_stats: SwarmStats,
}

/// Statystyki całego swarm
#[derive(Debug, Clone)]
pub struct SwarmStats {
    /// Całkowita liczba agentów
    pub total_agents: usize,
    /// Liczba aktywnych agentów
    pub active_agents: usize,
    /// Średni success rate swarm
    pub avg_success_rate: f32,
    /// Całkowita liczba wykonanych zadań
    pub total_tasks: u64,
}

impl SwarmTopology {
    /// Utworzenie nowej topologii swarm
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            next_agent_id: 1,
            swarm_stats: SwarmStats {
                total_agents: 0,
                active_agents: 0,
                avg_success_rate: 1.0,
                total_tasks: 0,
            },
        }
    }

    /// Dodanie nowego agenta do swarm
    pub fn add_agent(&mut self, agent_type: AgentType, config: Option<AgentConfig>) -> AgentId {
        let agent_id = self.next_agent_id;
        self.next_agent_id += 1;

        let agent = AgentDefinition {
            id: agent_id,
            agent_type,
            status: AgentStatus::Active,
            metrics: AgentMetrics::new(),
            config: config.unwrap_or_default(),
        };

        self.agents.insert(agent_id, agent);
        self.update_stats();
        
        agent_id
    }

    /// Pobranie agenta według ID
    pub fn get_agent(&self, agent_id: AgentId) -> Option<&AgentDefinition> {
        self.agents.get(&agent_id)
    }

    /// Pobranie mutable agenta według ID
    pub fn get_agent_mut(&mut self, agent_id: AgentId) -> Option<&mut AgentDefinition> {
        self.agents.get_mut(&agent_id)
    }

    /// Znalezienie najlepszego agenta dla typu zadania
    pub fn find_best_agent(&self, agent_type: AgentType) -> Option<AgentId> {
        self.agents
            .values()
            .filter(|agent| {
                agent.agent_type == agent_type && 
                agent.status == AgentStatus::Active
            })
            .max_by(|a, b| {
                // Sortowanie według success rate i czasu ostatniej aktywności
                a.metrics.success_rate
                    .partial_cmp(&b.metrics.success_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        b.metrics.last_activity.cmp(&a.metrics.last_activity)
                    })
            })
            .map(|agent| agent.id)
    }

    /// Aktualizacja statusu agenta
    pub fn update_agent_status(&mut self, agent_id: AgentId, status: AgentStatus) -> CortexResult<()> {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.status = status;
            self.update_stats();
            Ok(())
        } else {
            Err(CortexError::SwarmError(format!("Agent {} not found", agent_id)))
        }
    }

    /// Aktualizacja metryk agenta po wykonaniu zadania
    pub fn update_agent_metrics(&mut self, agent_id: AgentId, execution_ns: u64, success: bool) -> CortexResult<()> {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.metrics.update_task(execution_ns, success);
            self.update_stats();
            Ok(())
        } else {
            Err(CortexError::SwarmError(format!("Agent {} not found", agent_id)))
        }
    }

    /// Pobranie statystyk swarm
    pub fn get_stats(&self) -> &SwarmStats {
        &self.swarm_stats
    }

    /// Aktualizacja statystyk swarm
    fn update_stats(&mut self) {
        self.swarm_stats.total_agents = self.agents.len();
        self.swarm_stats.active_agents = self.agents
            .values()
            .filter(|agent| agent.status == AgentStatus::Active)
            .count();

        if !self.agents.is_empty() {
            self.swarm_stats.avg_success_rate = self.agents
                .values()
                .map(|agent| agent.metrics.success_rate)
                .sum::<f32>() / self.agents.len() as f32;

            self.swarm_stats.total_tasks = self.agents
                .values()
                .map(|agent| agent.metrics.tasks_completed)
                .sum();
        }
    }

    /// Usunięcie nieaktywnych agentów (cleanup)
    pub fn cleanup_inactive_agents(&mut self, max_inactive_duration: std::time::Duration) {
        let now = std::time::Instant::now();
        let agents_to_remove: Vec<AgentId> = self.agents
            .values()
            .filter(|agent| {
                matches!(agent.status, AgentStatus::Inactive | AgentStatus::Error(_)) &&
                now.duration_since(agent.metrics.last_activity) > max_inactive_duration
            })
            .map(|agent| agent.id)
            .collect();

        for agent_id in agents_to_remove {
            self.agents.remove(&agent_id);
        }

        self.update_stats();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_topology_creation() {
        let swarm = SwarmTopology::new();
        assert_eq!(swarm.agents.len(), 0);
        assert_eq!(swarm.swarm_stats.total_agents, 0);
    }

    #[test]
    fn test_add_agent() {
        let mut swarm = SwarmTopology::new();
        let agent_id = swarm.add_agent(AgentType::Sentiment, None);
        
        assert_eq!(agent_id, 1);
        assert_eq!(swarm.agents.len(), 1);
        assert_eq!(swarm.swarm_stats.total_agents, 1);
        assert_eq!(swarm.swarm_stats.active_agents, 1);
    }

    #[test]
    fn test_find_best_agent() {
        let mut swarm = SwarmTopology::new();
        let agent1 = swarm.add_agent(AgentType::Sentiment, None);
        let agent2 = swarm.add_agent(AgentType::Sentiment, None);
        
        // Aktualizacja metryk - agent2 ma lepszy success rate
        swarm.update_agent_metrics(agent1, 1000000, false).unwrap();
        swarm.update_agent_metrics(agent2, 500000, true).unwrap();
        
        let best = swarm.find_best_agent(AgentType::Sentiment);
        assert_eq!(best, Some(agent2));
    }
}
