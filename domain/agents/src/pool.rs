//! Agent-Pool — Round-Robin-Scheduling für Epoch-basierte Ausführung.

use std::collections::VecDeque;
use tokio::sync::Mutex;
use types::agent::Agent;

/// FIFO-Warteschlange für Agent-Scheduling.
pub struct AgentPool {
    queue: Mutex<VecDeque<Agent>>,
}

impl AgentPool {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    pub async fn push(&self, agent: Agent) {
        self.queue.lock().await.push_back(agent);
    }

    pub async fn pop(&self) -> Option<Agent> {
        self.queue.lock().await.pop_front()
    }

    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }
}

impl Default for AgentPool {
    fn default() -> Self {
        Self::new()
    }
}
