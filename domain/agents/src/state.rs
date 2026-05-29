//! Agent-State-Transition-Logik.
//!
//! Pure Funktionen — kein I/O, deterministisch.
//! Die State-Machine definiert erlaubte Übergänge.

use types::agent::{AgentState, AgentCapability};
use contracts::error::{FfError, Result};
use chrono::Utc;

/// Setzt einen Agent in den Active-Zustand.
pub fn start_task(current: &AgentState, task: impl Into<String>) -> Result<AgentState> {
    match current {
        AgentState::Idle | AgentState::Resting { .. } => Ok(AgentState::Active {
            current_task: task.into(),
            started_at:   Utc::now(),
        }),
        AgentState::Dead { reason } => {
            Err(FfError::AgentDead(reason.clone()))
        }
        _ => Ok(AgentState::Active {
            current_task: task.into(),
            started_at:   Utc::now(),
        }),
    }
}

/// Versetzt den Agent in Ruhezustand bis Tick `until`.
pub fn rest(until_tick: u64) -> AgentState {
    AgentState::Resting { until_tick }
}

/// Markiert den Agent als tot.
pub fn die(reason: impl Into<String>) -> AgentState {
    AgentState::Dead { reason: reason.into() }
}

/// Prüft, ob ein Übergang von `from` nach `to` erlaubt ist.
pub fn can_transition(from: &AgentState, to: &AgentState) -> bool {
    match (from, to) {
        // Aus Dead gibt es keinen Rückweg.
        (AgentState::Dead { .. }, _) => false,
        // Alles andere ist erlaubt.
        _ => true,
    }
}

/// Prüft, ob der Agent eine bestimmte Fähigkeit besitzt.
pub fn check_capability(capabilities: &[AgentCapability], required: AgentCapability) -> Result<()> {
    if capabilities.contains(&required) {
        Ok(())
    } else {
        Err(FfError::CapabilityDenied(
            "agent".to_string(),
            format!("{required:?}"),
        ))
    }
}
