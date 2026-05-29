# Changelog — ff-one (ForgeFabrik MMO / World Engine)

---

## [Unreleased] — main

### Added
- **Persistent EventLog** — all `WorldEvent` values emitted by `tick_world` are
  now appended to the event log via `EventLog::append` in `runtime/server`.
  Phase 1.2 of the roadmap is complete.
- `WorldTick` newtype in `foundation/types` — replaces bare `u64` tick counters,
  aligned with forge-core SYNC_CONTRACT §3.
- `WorldSnapshot` in `foundation/types` — carries `state_hash` for deterministic
  cross-node verification.
- `FORGE_CORE_SYNC.md` — compliance matrix against forge-core SYNC_CONTRACT v0.1.
- `AGENTS.md` — federation instructions for AI agents.
- `docs/adr/0001-driver-plugin-boundary.md` — Architecture Decision Record:
  frozen plugin-vs-driver boundary.
- `CONTRIBUTING.md` — contribution guidelines.
- Dockerfile + docker-compose for local dev (Phase 4.2 roadmap).

### Architecture
- Layer model enforced: `foundation → domain → runtime → plugins`
- `AgentKind::Free(FreeProvider)` grouping — avoids top-level provider explosion
- Plugin ABI: `ff_plugin_init / ff_plugin_tick / ff_plugin_shutdown` (canonical)
- `export_forgefabrik_plugin!` macro in all four plugins
- BLAKE3 consensus protocol in `domain/consensus`

### Tests
- `world::simulator::tests::same_seed_same_hash` — deterministic replay
- `sandbox::tests::snapshot_roundtrip` — snapshot round-trip
