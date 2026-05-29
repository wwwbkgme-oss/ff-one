//! Agent identity, state, capabilities, and runtime statistics.

use crate::position::Position3D;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Free-tier LLM providers — no credit card required.
///
/// All implement the `AgentDriver` trait via OpenAI-compatible HTTP endpoints.
/// Grouped here so `AgentKind` does not accumulate one variant per provider
/// ("provider explosion" antipattern).
///
/// **Layer:** `foundation/types` — pure data, no I/O.
/// Actual HTTP clients live in `runtime/drivers/free.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FreeProvider {
    /// Groq Cloud — fast Llama 3 / Gemma 2 inference. Env: `GROQ_API_KEY`
    Groq,
    /// SambaNova Cloud — Llama 3.3 / DeepSeek V3. Env: `SAMBANOVA_API_KEY`
    SambaNova,
    /// Local Ollama — runs on your hardware, no key needed.
    Ollama,
    /// OpenRouter — free models carry a `:free` suffix. Env: `OPENROUTER_API_KEY`
    OpenRouter,
    /// Cerebras — ultra-fast RDU inference. Env: `CEREBRAS_API_KEY`
    Cerebras,
}

impl std::fmt::Display for FreeProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Must match the string returned by the corresponding AgentDriver::name().
        let s = match self {
            Self::Groq => "Groq",
            Self::SambaNova => "SambaNova",
            Self::Ollama => "Ollama",
            Self::OpenRouter => "OpenRouter",
            Self::Cerebras => "Cerebras",
        };
        write!(f, "{s}")
    }
}

/// Which AI backend drives this agent.
///
/// Variants represent **agent semantics**, not infrastructure config.
/// Adding a new free-tier provider means adding a `FreeProvider` variant,
/// not a top-level `AgentKind` variant — this keeps the enum stable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentKind {
    // ── Paid / cloud backends ─────────────────────────────────────────────
    Claude,
    OpenCode,
    Codex,
    Amp,
    Pi,
    Cursor,
    // ── Free-tier backends (grouped to prevent provider explosion) ────────
    /// A free-tier provider — see [`FreeProvider`].
    Free(FreeProvider),
    // ── Escape hatch ─────────────────────────────────────────────────────
    /// Any OpenAI-compatible endpoint not covered above.
    Custom {
        name: String,
        endpoint: String,
    },
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free(p) => write!(f, "{p}"),
            Self::Custom { name, .. } => write!(f, "{name}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// Lifecycle phase of an agent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    #[default]
    Idle,
    Active {
        current_task: String,
        started_at: DateTime<Utc>,
    },
    Executing,
    Resting {
        until_tick: u64,
    },
    Dead {
        reason: String,
    },
}

/// Permissions an agent has been granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    BuildAndMine,
    ExecuteCode,
    Combat,
    Trade,
    Witness,
    QuestGiver,
}

/// Accumulated runtime statistics for one agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentStats {
    pub blocks_mined: u64,
    pub blocks_placed: u64,
    pub code_executions: u64,
    pub quests_completed: u64,
    pub gold_earned: u64,
    pub xp: u64,
    pub kills: u32,
    pub deaths: u32,
}

/// The canonical agent record — single source of truth for agent data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub kind: AgentKind,
    pub state: AgentState,
    pub position: Position3D,
    pub capabilities: Vec<AgentCapability>,
    pub stats: AgentStats,
    /// Arbitrary config passed at spawn time.
    pub config: HashMap<String, serde_json::Value>,
    pub spawned_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub epoch: u64,
}

impl Agent {
    pub fn new(name: impl Into<String>, kind: AgentKind) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            state: Default::default(),
            position: Position3D::ORIGIN,
            capabilities: vec![
                AgentCapability::BuildAndMine,
                AgentCapability::ExecuteCode,
                AgentCapability::Trade,
                AgentCapability::Witness,
            ],
            stats: Default::default(),
            config: HashMap::new(),
            spawned_at: now,
            last_active: now,
            epoch: 0,
        }
    }

    pub fn has_capability(&self, cap: AgentCapability) -> bool {
        self.capabilities.contains(&cap)
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.state, AgentState::Dead { .. })
    }

    /// Derived level from XP (1000 XP per level).
    pub fn level(&self) -> u64 {
        self.stats.xp / 1000 + 1
    }
}
