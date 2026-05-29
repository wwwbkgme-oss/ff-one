//! Canonical deterministic simulation time primitive.
//!
//! **Sync rule (forge-core):** `WorldTick` replaces all wall-clock usage inside
//! `domain/`. The domain never calls `Utc::now()` or `thread_rng()`.
//!
//! ## Migration note
//!
//! `WorldState.tick` is currently `u64`. The canonical type is `WorldTick(u64)`.
//! Future migration: replace `u64` fields with `WorldTick` via a non-breaking
//! rename PR (same wire format, stronger type safety).

use serde::{Deserialize, Serialize};

/// Monotonically increasing simulation counter.
/// One tick = one logical step; wall-clock duration is runtime-defined.
///
/// Semantically identical to `forge-core::WorldTick` — same wire format (`u64`).
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct WorldTick(pub u64);

impl WorldTick {
    pub const ZERO: Self = Self(0);

    pub fn advance(self, delta: u64) -> Self {
        Self(self.0 + delta)
    }
    pub fn elapsed_since(self, earlier: Self) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
    /// Convert from the legacy bare `u64` tick field on `WorldState`.
    pub fn from_u64(t: u64) -> Self {
        Self(t)
    }
}

impl From<u64> for WorldTick {
    fn from(t: u64) -> Self {
        Self(t)
    }
}

impl From<WorldTick> for u64 {
    fn from(t: WorldTick) -> u64 {
        t.0
    }
}

impl std::fmt::Display for WorldTick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tick({})", self.0)
    }
}

/// Number of ticks that make up one in-world day.
pub const DAY_LENGTH_TICKS: u64 = 2400;
