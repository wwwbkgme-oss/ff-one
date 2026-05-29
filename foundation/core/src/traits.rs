//! Trait-Verträge für alle Domain- und Runtime-Implementierungen.
//!
//! **BKG-Prinzip:** Domain-Crates implementieren diese Traits.
//! Runtime-Crates orchestrieren sie.  Foundation kennt keine Implementierungen.

use crate::{error::Result, events::WorldEvent};
use async_trait::async_trait;
use types::{
    agent::{Agent, AgentCapability},
    block::Block,
    consensus::{ConsensusRound, WorldHash},
    economy::{AuctionItem, MarketListing, Resource, Wallet},
    plugin::{PluginCapability, PluginManifest, PluginRecord},
    position::{ChunkPos, Position3D},
    quest::Quest,
    sandbox::{ExecutionResult, SandboxConfig, SandboxState},
    security::{SafetyDecision, SecurityAssessment},
    world::{Chunk, WorldState},
};
use uuid::Uuid;

// ── World Simulator ───────────────────────────────────────────────────────────

/// Deterministischer Welt-Simulator.
/// Implementiert von `domain/world`.
#[async_trait]
pub trait WorldSimulator: Send + Sync {
    async fn tick(&mut self, world: &mut WorldState) -> Result<Vec<WorldEvent>>;
    async fn load_chunk(&mut self, world: &mut WorldState, pos: ChunkPos) -> Result<Chunk>;
    async fn place_block(
        &mut self, world: &mut WorldState,
        agent_id: Uuid, pos: Position3D, block: Block,
    ) -> Result<WorldEvent>;
    async fn mine_block(
        &mut self, world: &mut WorldState,
        agent_id: Uuid, pos: Position3D,
    ) -> Result<WorldEvent>;
    fn compute_hash(&self, world: &WorldState) -> WorldHash;
}

// ── Agent Driver ──────────────────────────────────────────────────────────────

/// Verbindung zu einem KI-Backend.
/// Implementiert von `runtime/drivers` — darf I/O machen.
#[async_trait]
pub trait AgentDriver: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<AgentCapability>;
    async fn complete(&self, agent: &Agent, prompt: &str) -> Result<String>;
    async fn generate_code(&self, agent: &Agent, task: &str, language: &str) -> Result<String>;
    async fn is_available(&self) -> bool;
}

// ── Sandbox Executor ──────────────────────────────────────────────────────────

/// Sichere Code-Ausführung.
/// Implementiert von `runtime/sandbox` — darf Prozesse spawnen.
#[async_trait]
pub trait SandboxExecutor: Send + Sync {
    async fn create(&self, config: SandboxConfig) -> Result<Uuid>;
    async fn execute(&self, sandbox_id: Uuid, code: &str) -> Result<ExecutionResult>;
    async fn state(&self, sandbox_id: Uuid) -> Result<SandboxState>;
    async fn destroy(&self, sandbox_id: Uuid) -> Result<()>;
    async fn snapshot(&self, sandbox_id: Uuid) -> Result<String>;
    async fn restore(&self, sandbox_id: Uuid, snapshot_id: &str) -> Result<()>;
}

// ── Security Analyser ─────────────────────────────────────────────────────────

/// Statische Sicherheitsanalyse — deterministisch.
/// Implementiert von `domain/security`.
#[async_trait]
pub trait SecurityAnalyser: Send + Sync {
    async fn assess(
        &self, code: &str, language: &str, agent_id: Uuid,
    ) -> Result<SecurityAssessment>;
    fn decide(&self, assessment: &SecurityAssessment) -> SafetyDecision;
}

// ── Economy Engine ────────────────────────────────────────────────────────────

/// Ressourcen-Wirtschaft — Markt, Auktionen, Wallets.
/// Implementiert von `domain/economy`.
#[async_trait]
pub trait EconomyEngine: Send + Sync {
    async fn list_resource(&self, seller_id: Uuid, resource: Resource, price: u64) -> Result<MarketListing>;
    async fn purchase(&self, listing_id: Uuid, buyer_id: Uuid, quantity: u64) -> Result<()>;
    async fn get_listings(&self) -> Result<Vec<MarketListing>>;
    async fn create_auction(&self, seller_id: Uuid, resource: Resource, start_price: u64, duration_secs: u64) -> Result<AuctionItem>;
    async fn place_bid(&self, auction_id: Uuid, bidder_id: Uuid, amount: u64) -> Result<()>;
    async fn get_auctions(&self) -> Result<Vec<AuctionItem>>;
    async fn get_wallet(&self, agent_id: Uuid) -> Result<Wallet>;
    async fn transfer(&self, from_id: Uuid, to_id: Uuid, amount: u64) -> Result<()>;
}

// ── Quest Manager ─────────────────────────────────────────────────────────────

/// Quest-Generierung und Lifecycle.
/// Implementiert von `domain/quests`.
#[async_trait]
pub trait QuestManager: Send + Sync {
    async fn available_quests(&self) -> Result<Vec<Quest>>;
    async fn accept_quest(&self, quest_id: Uuid, agent_id: Uuid) -> Result<()>;
    async fn update_progress(&self, quest_id: Uuid, progress_pct: u8) -> Result<()>;
    async fn complete_quest(&self, quest_id: Uuid, tick: u64) -> Result<()>;
    async fn fail_quest(&self, quest_id: Uuid, reason: &str) -> Result<()>;
    async fn generate_quest(&self, world: &WorldState, agent: &Agent) -> Result<Quest>;
}

// ── Consensus Coordinator ─────────────────────────────────────────────────────

/// BLAKE3-Weltstate-Konsens.
/// Implementiert von `domain/consensus`.
#[async_trait]
pub trait ConsensusCoordinator: Send + Sync {
    async fn start_round(&self, tick: u64, required_witnesses: u32) -> Result<ConsensusRound>;
    async fn submit_witness(&self, tick: u64, agent_id: Uuid, hash: WorldHash) -> Result<()>;
    async fn get_round(&self, tick: u64) -> Result<ConsensusRound>;
    async fn await_consensus(&self, tick: u64, timeout_ms: u64) -> Result<ConsensusRound>;
}

// ── Plugin Host ───────────────────────────────────────────────────────────────

/// Plugin-Ladevorgang und Lifecycle.
/// Implementiert von `runtime/plugin`.
#[async_trait]
pub trait PluginHost: Send + Sync {
    async fn load(&mut self, manifest_path: &str) -> Result<PluginRecord>;
    async fn unload(&mut self, plugin_id: &str) -> Result<()>;
    fn get(&self, plugin_id: &str) -> Option<&PluginRecord>;
    fn list(&self) -> Vec<&PluginRecord>;
    fn available_capabilities(&self) -> Vec<PluginCapability>;
    fn validate_manifest(&self, manifest: &PluginManifest) -> Result<()>;
}
