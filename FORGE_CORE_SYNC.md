# forge-core Sync Contract — ff-one Compliance

Reference: [`forge-core/SYNC_CONTRACT.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/SYNC_CONTRACT.md) v0.1

> **Treat this repository as part of a federated ForgeFabrik system.**
> Maintain compatibility with the canonical event + domain model.
> Do not introduce incompatible abstractions.

---

## Compliance Status

| §   | Rule | Status |
|-----|------|--------|
| §2  | Layer model: foundation ← domain ← runtime | ✅ |
| §2  | Plugin = behavior only (no I/O) | ✅ |
| §2  | Driver = I/O adapter in runtime/drivers | ✅ |
| §3  | `Agent` canonical name | ✅ |
| §3  | `WorldState` canonical name | ✅ |
| §3  | `WorldEvent` canonical name | ✅ |
| §3  | `WorldTick` canonical name | ✅ (`foundation/types`) |
| §3  | `WorldSnapshot` canonical name | ✅ |
| §3  | `ForgeFabrikPlugin` trait | ✅ (`foundation/contracts`) |
| §3  | `FfError` / `FfResult` | ✅ (`foundation/contracts`) |
| §3  | `EventStore` trait | ✅ (`foundation/contracts`) |
| §4  | No `Utc::now()` in domain/foundation | ✅ (seeded via `tick`) |
| §4  | No `thread_rng()` — use deterministic RNG | ✅ |
| §5  | Event-First: Command → Event → Reducer → State | ✅ |
| §5  | Single mutation path | ✅ |
| §6  | Plugin ABI: `ff_plugin_init / tick / shutdown` | ✅ (`runtime/plugin/src/abi.rs`) |
| §6  | Plugin.toml manifest format | ✅ |
| §6  | `[capabilities] provides / requires` | ✅ |
| §7  | `AgentKind::Free(FreeProvider)` grouping | ✅ |
| §7  | No top-level `AgentKind` for providers | ✅ |
| §8  | Deterministic replay test | ✅ (`world::simulator::tests::same_seed_same_hash`) |
| §8  | Event equality test | ✅ |
| §8  | Snapshot round-trip test | ✅ (`sandbox::tests::snapshot_roundtrip`) |

**ff-one is the reference implementation for the canonical plugin ABI and AgentKind grouping.**

---

## Canonical Names in this Repo

| Canonical | ff-one location | Notes |
|---|---|---|
| `WorldTick` | `foundation/types` | `u64` alias |
| `WorldEvent` | `foundation/contracts` | includes all variant categories |
| `WorldState` | `domain/world` via `VoxelSimulator` | hash via BLAKE3 |
| `WorldSnapshot` | `foundation/types` | carries `state_hash` |
| `ForgeFabrikPlugin` | `foundation/contracts` | Rust trait |
| `FfPluginCtx` | `runtime/plugin/src/abi.rs` | `#[repr(C)]` |
| `AgentKind::Free(FreeProvider)` | `foundation/types` | 9 providers |
| `TickContext` | `foundation/types` (implicit via tick param) | |

---

## Unique contributions of ff-one

These patterns from ff-one are part of the canonical reference:

- **BLAKE3 Consensus Protocol** — `domain/consensus::ConsensusStore`
- **Plugin host with capability graph** — `runtime/plugin::PluginHostImpl`
- **`export_forgefabrik_plugin!` macro** — upstream source
- **`AgentKind::Free(FreeProvider)` grouping** — prevents provider explosion
- **`docs/DRIVER_PLUGIN_BOUNDARY.md`** — frozen boundary spec (predates ARCHITECTURE.md in other repos)

---

## Compatibility Layer

No adaptation needed — ff-one **is** the reference implementation.

When integrating with other repos, map:
- `ff-three::CharacterEvent` → `WorldEvent::AgentStateChanged` (semantic equivalent)
- `ff-two::TaskEvent` → `WorldEvent::AgentStateChanged` / `EpochStarted`
