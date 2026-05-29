# NEXT — ff-one Planned Work

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the full phased plan.
This file tracks the immediate sprint.

---

## Done ✅

| Item | Notes |
|---|---|
| Layer model: foundation → domain → runtime → plugins | Enforced, frozen |
| `WorldTick` + `WorldSnapshot` canonical types | `foundation/types` |
| Plugin ABI: `ff_plugin_init/tick/shutdown` | All 4 plugins compliant |
| `AgentKind::Free(FreeProvider)` grouping | 9 free LLM providers |
| BLAKE3 consensus protocol | `domain/consensus` |
| Persistent EventLog (Phase 1.2) | `tick_world` appends all `WorldEvent` |
| FORGE_CORE_SYNC.md compliance matrix | SYNC_CONTRACT v0.1 |
| Dockerfile + docker-compose (Phase 4.2) | |

---

## Next sprint — Phase 1 remaining

### P0 · OS-level sandbox isolation (Phase 1.1)

`ProcessSandbox` has no OS-level resource containment.
- Linux: `seccomp-bpf` syscall filter + `cgroups v2` (CPU + memory limits)
- macOS dev: `sandbox-exec` profile
- Acceptance: infinite-allocation script killed within `timeout_ms`

### P1 · Snapshot/restore (Phase 1.3)

`WorldState` can be hashed (BLAKE3) but not serialised to disk.
- Implement `WorldState::to_snapshot()` / `WorldState::from_snapshot()`
- Store in event log alongside events (checkpoint pattern)
- Enables fast recovery after crash

### P1 · CI/CD pipeline (Phase 1.4)

- GitHub Actions: `cargo check`, `cargo clippy -D warnings`, `cargo test`
- Docker image build + push on merge to main

---

## Phase 2

| Item | Priority |
|---|---|
| Real-time event streaming (WebSocket / SSE) | P1 |
| Epoch scheduler | P1 |
| WorldState persistence (Postgres) | P1 |
| Observability (tracing + metrics) | P2 |

---

## Invariants to preserve

- `domain/` is deterministic: same seed → same `WorldState` hash
- `foundation/` has zero I/O
- Plugin ABI is `#[repr(C)]` + `extern "C"` — no vtable instability
- `AgentKind::Free(FreeProvider)` — no top-level provider variants ever
