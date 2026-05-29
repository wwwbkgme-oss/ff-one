//! Quest domain types.
//!
//! Economy types live in `ff-types::economy` (separate concept).

use crate::agent::AgentKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Progress state of a quest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestStatus {
    Available,
    Active { progress_pct: u8 },
    Completed { at_tick: u64 },
    Failed { reason: String },
    Expired,
}

/// What the quest requires the agent to do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjective {
    MineBlocks {
        material: String,
        count: u32,
    },
    BuildStructure {
        template_id: String,
    },
    SolveChallenge {
        challenge: String,
        expected_hash: String,
    },
    Defeat {
        target: String,
        count: u32,
    },
    Explore {
        target: String,
        radius: u32,
    },
    Earn {
        amount: u64,
    },
    Freeform {
        description: String,
    },
}

/// Reward granted on completion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestReward {
    pub gold: u64,
    pub xp: u64,
    pub items: Vec<String>,
}

/// Canonical quest record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub objective: QuestObjective,
    pub reward: QuestReward,
    pub difficulty: u8,
    pub status: QuestStatus,
    pub eligible_kinds: Vec<AgentKind>,
    pub accepted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Quest {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        objective: QuestObjective,
        reward: QuestReward,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            objective,
            reward,
            difficulty: 1,
            status: QuestStatus::Available,
            eligible_kinds: Vec::new(),
            accepted_by: None,
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status, QuestStatus::Available)
    }
}
