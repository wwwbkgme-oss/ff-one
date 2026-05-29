//! World event bus.
//!
//! **BKG-Prinzip:** *Events sind Wahrheit. State ist Projektion.*
//!
//! [`WorldEvent`] ist der einzige erlaubte Mutations-Kanal zwischen den Layern.
//! Alle Subsysteme emittieren Events; der State wird ausschließlich durch
//! Reducer auf Basis dieser Events aufgebaut (Replay-Safety).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use types::{
    agent::AgentState,
    block::Block,
    consensus::ConsensusResult,
    economy::{AuctionItem, MarketListing},
    position::Position3D,
    quest::QuestStatus,
    sandbox::ExecutionResult,
    security::SecurityAssessment,
};
use uuid::Uuid;

/// Alle Ereignisse, die im System eintreten können.
///
/// Jede Variante ist vollständig selbstbeschreibend — kein impliziter Kontext.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorldEvent {
    // Agent-Lifecycle
    AgentSpawned {
        agent_id: Uuid,
        name: String,
        at: Position3D,
    },
    AgentMoved {
        agent_id: Uuid,
        from: Position3D,
        to: Position3D,
        tick: u64,
    },
    AgentStateChanged {
        agent_id: Uuid,
        new_state: AgentState,
    },
    AgentDied {
        agent_id: Uuid,
        reason: String,
        tick: u64,
    },

    // Welt-Mutationen
    BlockPlaced {
        agent_id: Uuid,
        position: Position3D,
        block: Block,
        tick: u64,
    },
    BlockMined {
        agent_id: Uuid,
        position: Position3D,
        was: Block,
        tick: u64,
    },
    ChunkLoaded {
        chunk_key: String,
    },
    ExplosionAt {
        center: Position3D,
        radius: u32,
        tick: u64,
    },

    // Sandbox / Security
    CodeSubmitted {
        sandbox_id: Uuid,
        agent_id: Uuid,
        language: String,
    },
    CodeExecuted {
        result: Box<ExecutionResult>,
    },
    SecurityAssessed {
        assessment: Box<SecurityAssessment>,
    },

    // Quests
    QuestCreated {
        quest_id: Uuid,
        title: String,
    },
    QuestAccepted {
        quest_id: Uuid,
        agent_id: Uuid,
    },
    QuestStatusChanged {
        quest_id: Uuid,
        new_status: QuestStatus,
    },

    // Economy
    MarketListingCreated {
        listing: Box<MarketListing>,
    },
    MarketPurchase {
        listing_id: Uuid,
        buyer_id: Uuid,
        quantity: u64,
    },
    AuctionCreated {
        auction: Box<AuctionItem>,
    },
    AuctionBidPlaced {
        auction_id: Uuid,
        bidder_id: Uuid,
        amount: u64,
    },
    AuctionClosed {
        auction_id: Uuid,
        winner: Option<Uuid>,
        final_price: u64,
    },

    // Konsens
    ConsensusRoundStarted {
        tick: u64,
    },
    ConsensusRoundFinalised {
        tick: u64,
        result: ConsensusResult,
    },

    // Epoch
    EpochStarted {
        epoch: u64,
        seed: u64,
    },
    EpochEnded {
        epoch: u64,
        dominant_faction: Option<String>,
    },
}

/// Event mit Tick-Nummer und Wallclock-Zeitstempel (für Logging, nicht für Determinismus).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    pub tick: u64,
    pub timestamp: DateTime<Utc>,
    pub event: WorldEvent,
}

impl TimestampedEvent {
    pub fn new(tick: u64, event: WorldEvent) -> Self {
        Self {
            tick,
            timestamp: Utc::now(),
            event,
        }
    }
}
