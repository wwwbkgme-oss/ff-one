//! Agent-Manager — Spawn, Command-Dispatch, State-Verwaltung.
//!
//! Verwaltet alle lebenden Agents und leitet Befehle über den
//! `AgentDriver`-Trait an die KI-Backends weiter (die in `runtime/drivers` leben).
//!
//! **BKG:** kein HTTP hier — nur Trait-Aufruf über `dyn AgentDriver`.

use crate::{prompt, state as st};
use contracts::{
    error::{FfError, Result},
    traits::AgentDriver,
};
use std::{collections::HashMap, sync::Arc};
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::info;
use types::{
    agent::{Agent, AgentKind},
    position::Position3D,
    world::WorldState,
};
use uuid::Uuid;

/// Verwaltet alle Agent-Instanzen der aktuellen Welt.
pub struct AgentManager {
    agents:  Arc<RwLock<HashMap<Uuid, Agent>>>,
    /// Registrierte KI-Treiber nach Backend-Name (z. B. "Claude", "OpenCode").
    drivers: HashMap<String, Arc<dyn AgentDriver>>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents:  Arc::new(RwLock::new(HashMap::new())),
            drivers: HashMap::new(),
        }
    }

    /// Registriert einen KI-Treiber (aus `runtime/drivers`).
    pub fn register_driver(&mut self, driver: Arc<dyn AgentDriver>) {
        info!(driver = %driver.name(), "AgentDriver registriert");
        self.drivers.insert(driver.name().to_string(), driver);
    }

    /// Spawnt einen neuen Agent an `pos`.
    pub async fn spawn(
        &self,
        name:    impl Into<String>,
        kind:    AgentKind,
        pos:     Position3D,
    ) -> Result<Agent> {
        let mut agent = Agent::new(name, kind);
        agent.position = pos;
        info!(id = %agent.id, name = %agent.name, "Agent gespawnt");
        self.agents.write().await.insert(agent.id, agent.clone());
        Ok(agent)
    }

    /// Gibt einen Agent zurück.
    pub async fn get(&self, id: Uuid) -> Result<Agent> {
        self.agents
            .read().await
            .get(&id)
            .cloned()
            .ok_or_else(|| FfError::AgentNotFound(id.to_string()))
    }

    /// Listet alle lebenden Agents.
    pub async fn list(&self) -> Vec<Agent> {
        self.agents.read().await.values().cloned().collect()
    }

    /// Terminiert einen Agent.
    pub async fn terminate(&self, id: Uuid, reason: impl Into<String>) -> Result<()> {
        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(&id)
            .ok_or_else(|| FfError::AgentNotFound(id.to_string()))?;
        agent.state = st::die(reason);
        Ok(())
    }

    /// Sendet einen natürlichsprachigen Befehl an einen Agent.
    ///
    /// Der Prompt wird deterministisch aus Welt-State und Agent-Daten aufgebaut.
    /// Der eigentliche HTTP-Aufruf erfolgt via `dyn AgentDriver` (runtime).
    pub async fn command(&self, id: Uuid, cmd: &str, world: &WorldState) -> Result<String> {
        let agent = self.get(id).await?;
        if !agent.is_alive() {
            return Err(FfError::AgentDead(agent.name.clone()));
        }
        let driver_name = agent.kind.to_string();
        let driver = self.drivers
            .get(&driver_name)
            .ok_or_else(|| FfError::Other(format!("Kein Treiber für {driver_name}")))?;

        let prompt = prompt::task_prompt(cmd, world);
        let response = driver.complete(&agent, &prompt).await?;

        // State-Transition via reiner Funktion — kein direkter State-Zugriff.
        let new_state = st::start_task(&agent.state, cmd)?;
        let mut agents = self.agents.write().await;
        if let Some(a) = agents.get_mut(&id) {
            a.last_active = Utc::now();
            a.state       = new_state;
        }
        Ok(response)
    }

    /// Generiert Code für eine Aufgabe.
    pub async fn generate_code(
        &self, id: Uuid, task: &str, language: &str,
    ) -> Result<String> {
        let agent = self.get(id).await?;
        let driver_name = agent.kind.to_string();
        let driver = self.drivers
            .get(&driver_name)
            .ok_or_else(|| FfError::Other(format!("Kein Treiber für {driver_name}")))?;
        driver.generate_code(&agent, task, language).await
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}
