//! Platform-wide error type.
//!
//! Single Source of Truth für alle Fehler-Varianten.
//! Jede Implementierungsschicht mappt ihre Fehler auf [`FfError`].

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfError {
    // ── World ────────────────────────────────────────────────────────────
    #[error("chunk {0} not found")]
    ChunkNotFound(String),
    #[error("position {0} out of bounds")]
    OutOfBounds(String),

    // ── Agent ────────────────────────────────────────────────────────────
    #[error("agent {0} not found")]
    AgentNotFound(String),
    #[error("agent {0} is dead")]
    AgentDead(String),
    #[error("agent {0} lacks capability {1}")]
    CapabilityDenied(String, String),

    // ── Sandbox ──────────────────────────────────────────────────────────
    #[error("sandbox {0} not found")]
    SandboxNotFound(String),
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
    #[error("sandbox timed out after {0} ms")]
    Timeout(u64),

    // ── Security ─────────────────────────────────────────────────────────
    #[error("code rejected: {0}")]
    SecurityRejection(String),

    // ── Economy ──────────────────────────────────────────────────────────
    #[error("insufficient funds: need {needed}, have {available}")]
    InsufficientFunds { needed: u64, available: u64 },
    #[error("listing {0} not found")]
    ListingNotFound(String),
    #[error("auction {0} is closed")]
    AuctionClosed(String),

    // ── Quests ───────────────────────────────────────────────────────────
    #[error("quest {0} not found")]
    QuestNotFound(String),
    #[error("quest {0} is not available")]
    QuestUnavailable(String),

    // ── Consensus ────────────────────────────────────────────────────────
    #[error("consensus disputed at tick {0}")]
    ConsensusDisputed(u64),

    // ── Plugin ───────────────────────────────────────────────────────────
    #[error("plugin {0} not found")]
    PluginNotFound(String),
    #[error("plugin load failed: {0}")]
    PluginLoadError(String),
    #[error("unsatisfied dependency: {0} requires {1}")]
    UnsatisfiedDependency(String, String),

    // ── I/O + serialisation ───────────────────────────────────────────────
    #[error("serialisation: {0}")]
    Serialisation(#[from] serde_json::Error),
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),

    // ── Catch-all ────────────────────────────────────────────────────────
    #[error("{0}")]
    Other(String),
}

/// Convenience alias used across every layer.
pub type Result<T> = std::result::Result<T, FfError>;
