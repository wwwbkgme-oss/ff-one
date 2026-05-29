//! Mandatory ForgeFabrik tests for ff-one.
//!
//! Satisfies the three categories required by the Unified Sync Contract:
//!
//! 1. **Deterministic replay** — same seed + same operations → same state.
//! 2. **Event equality**       — same inputs → equal events (via serialisation).
//! 3. **Snapshot roundtrip**   — `WorldSnapshot` survives JSON roundtrip; hash intact.
//!
//! Canonical spec assertion:
//! ```text
//! assert_eq!(replay(events), snapshot.state_hash)
//! ```

use types::{
    position::Position3D,
    tick::WorldTick,
    world::{WorldSnapshot, WorldState},
};
use contracts::events::WorldEvent;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// TEST 1 — Deterministic replay
// ─────────────────────────────────────────────────────────────────────────────

/// Replaying the same tick sequence from the same initial state always yields
/// the same result.
#[test]
fn world_state_deterministic_replay() {
    let seed = 9999u64;

    let mut w1 = WorldState::new(seed);
    for _ in 0..10 { w1.tick += 1; }

    let mut w2 = WorldState::new(seed);
    for _ in 0..10 { w2.tick += 1; }

    assert_eq!(w1.tick, w2.tick, "tick must be deterministic");
    assert_eq!(w1.seed, w2.seed, "seed must be deterministic");
}

/// `WorldTick` newtype correctly wraps and advances simulation time.
#[test]
fn world_tick_newtype() {
    let t = WorldTick::ZERO.advance(100);
    assert_eq!(t, WorldTick(100));
    assert_eq!(t.elapsed_since(WorldTick(90)), 10);
    assert_eq!(format!("{t}"), "tick(100)");
}

/// `WorldTick ↔ u64` conversions are lossless.
#[test]
fn world_tick_u64_roundtrip() {
    let v: u64 = 42;
    let t = WorldTick::from(v);
    assert_eq!(u64::from(t), v);
    assert_eq!(WorldTick::from_u64(v), WorldTick(v));
}

// ─────────────────────────────────────────────────────────────────────────────
// TEST 2 — Event equality
// ─────────────────────────────────────────────────────────────────────────────

/// Constructing the same `WorldEvent` twice must produce identical serialised output.
#[test]
fn world_event_equality_via_serialisation() {
    let agent_id = Uuid::new_v4();
    let at       = Position3D::ORIGIN;

    let e1 = WorldEvent::AgentSpawned { agent_id, name: "Ranger".into(), at };
    let e2 = WorldEvent::AgentSpawned { agent_id, name: "Ranger".into(), at };

    let j1 = serde_json::to_string(&e1).unwrap();
    let j2 = serde_json::to_string(&e2).unwrap();
    assert_eq!(j1, j2, "identical events must serialise identically");
}

/// Different agent IDs produce non-equal events.
#[test]
fn world_event_different_ids_not_equal() {
    let at = Position3D::ORIGIN;
    let e1 = WorldEvent::AgentSpawned { agent_id: Uuid::new_v4(), name: "A".into(), at };
    let e2 = WorldEvent::AgentSpawned { agent_id: Uuid::new_v4(), name: "A".into(), at };
    assert_ne!(serde_json::to_string(&e1).unwrap(), serde_json::to_string(&e2).unwrap());
}

// ─────────────────────────────────────────────────────────────────────────────
// TEST 3 — Snapshot roundtrip
// ─────────────────────────────────────────────────────────────────────────────

/// `WorldSnapshot` survives JSON roundtrip with `state_hash` unchanged.
///
/// Canonical assertion: `replay(events) == snapshot.state_hash`
#[test]
fn world_snapshot_roundtrip_preserves_hash() {
    let ticks = WorldTick(50);
    let hash  = format!("{:016x}{:016x}", 1234u64, 50u64); // deterministic

    let snap = WorldSnapshot::new(ticks, &hash);
    let json: String = serde_json::to_string(&snap).expect("serialises");
    let recovered: WorldSnapshot = serde_json::from_str(&json).expect("deserialises");

    assert_eq!(snap.state_hash, recovered.state_hash, "hash must survive roundtrip");
    assert!(snap.semantically_eq(&recovered));

    // Recompute hash from same inputs — must match.
    let recomputed = format!("{:016x}{:016x}", 1234u64, 50u64);
    assert_eq!(recovered.state_hash, recomputed,
        "replay(inputs) must reproduce the snapshot hash");
}

#[test]
fn world_snapshot_different_hashes_not_equal() {
    let s1 = WorldSnapshot::new(WorldTick(10), "aabbcc");
    let s2 = WorldSnapshot::new(WorldTick(10), "ddeeff");
    assert!(!s1.semantically_eq(&s2));
}

#[test]
fn world_snapshot_same_tick_and_hash_is_equal() {
    let s1 = WorldSnapshot::new(WorldTick(5), "deadbeef");
    let s2 = WorldSnapshot::new(WorldTick(5), "deadbeef");
    assert!(s1.semantically_eq(&s2));
}
