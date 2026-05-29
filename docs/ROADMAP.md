# ForgeFabrik — Roadmap

This document is the canonical next-steps plan for ForgeFabrik.
Items are grouped by phase in rough priority order. Each item includes a brief
rationale and a pointer to the relevant code that currently limits the feature.

---

## Phase 1 — Foundation Hardening

These items address correctness, security, and operational gaps that block everything else.

### 1.1 OS-level sandbox isolation

**Current state:** `ProcessSandbox` (runtime/sandbox) spawns child processes with
`env_clear()` and a millisecond timeout. There is no OS-level resource containment.
A submitted script can exhaust CPU, allocate unbounded memory, or attempt filesystem
operations that regex rules missed.

**Goal:** Wrap child processes in an isolation layer before spawning:
- Linux: `seccomp-bpf` filter to allowlist only safe syscalls; `cgroups v2` for memory
  and CPU limits; optionally `nsjail` or `bubblewrap` for filesystem namespacing.
- macOS (dev): `sandbox-exec` profile as a best-effort alternative.
- The `SandboxExecutor` trait signature does not need to change — only the implementation.

**Acceptance criteria:**
- A script that calls `os.system('id')` produces `ExitStatus::NonZero` or is killed.
- A script with an infinite allocation loop is killed within `limits.timeout_ms`.
- `cargo test --workspace` still passes.

---

### 1.2 Persistent event log

**Current state:** `WorldEvent` values are returned from domain functions and counted in
HTTP responses, but are never stored. Replaying history or recovering from a crash is
impossible.

**Goal:** Append every emitted `WorldEvent` (wrapped in `TimestampedEvent`) to a durable,
ordered log. Suggested storage: embedded `sled` database or append-only SQLite table.
The log must be queryable by tick range.

**Interface sketch:**

```rust
// foundation/contracts — new trait
#[async_trait]
pub trait EventLog: Send + Sync {
    async fn append(&self, event: TimestampedEvent) -> Result<u64>; // returns sequence number
    async fn since(&self, tick: u64, limit: usize) -> Result<Vec<TimestampedEvent>>;
}
```

**Acceptance criteria:**
- Server restart replays the event log and reconstructs `WorldState`.
- `GET /events?since_tick=N` returns the event slice.

---

### 1.3 Real sandbox snapshot / restore

**Current state:** `ProcessSandbox::snapshot()` inserts an empty `Vec<u8>` into a map.
`restore()` checks the key exists and does nothing else. The feature is a documented stub.

**Goal:** Implement a real filesystem snapshot of the sandbox working directory using
`tar` or `cp -a` into the `TempDir`. `restore()` replaces the working directory contents
with the saved snapshot.

**Acceptance criteria:**
- A file written to the sandbox before a snapshot is present after restore.
- A file written after the snapshot is absent after restore.

---

### 1.4 CI/CD pipeline

**Current state:** No `.github/workflows/` directory exists. There is no automated
build, test, or lint gate on pull requests.

**Goal:** Add a minimal GitHub Actions workflow:

```yaml
# .github/workflows/ci.yml
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check
```

**Acceptance criteria:**
- PRs without passing CI cannot be merged.
- Clippy warnings on new code fail the build.

---

## Phase 2 — Runtime Completeness

These items make the running server production-worthy.

### 2.1 Real-time event streaming (WebSocket / SSE)

**Current state:** Clients poll `GET /world` and `GET /agents`. There is no push channel.
The `WorldEvent` bus has no subscriber mechanism.

**Goal:** Add a `GET /events/stream` endpoint that streams `TimestampedEvent` values as
Server-Sent Events (SSE) or as a WebSocket connection. Axum's `axum::response::Sse`
or `axum-ws` crate are suitable.

Internal plumbing: `AppState` holds a `broadcast::Sender<TimestampedEvent>`. Domain
calls that emit events also send to the channel. Each connected SSE/WS client holds a
`broadcast::Receiver`.

**Acceptance criteria:**
- `curl -N http://localhost:8080/events/stream` prints events as they occur.
- Slow consumers are dropped after a configurable buffer-full policy.

---

### 2.2 Epoch scheduler

**Current state:** `WorldEvent::EpochStarted` and `EpochEnded { dominant_faction }` are
defined in `foundation/contracts`. `WorldState` has an `epoch` field. Nothing advances
the epoch or determines a dominant faction.

**Goal:** Add an epoch tick threshold to `WorldState` (e.g. 1000 ticks per epoch).
In `VoxelSimulator::tick()`:
- When `world.tick % EPOCH_LENGTH == 0`: emit `EpochEnded` (compute dominant faction
  from agent stats), then `EpochStarted` for the next epoch.
- Faction scoring: sum `blocks_mined + blocks_placed` per agent; group by a yet-to-be-
  defined `faction` field on `Agent`.

**Acceptance criteria:**
- After 1000 ticks, `world.epoch.number` increments by 1.
- `EpochEnded` is present in the event log with a non-null `dominant_faction`.

---

### 2.3 WorldState persistence

**Current state:** `WorldState` is reconstructed from scratch on every server start using
only the seed. Chunk data modified by agents (mined/placed blocks) is lost on restart.

**Goal:** Persist `WorldState` snapshots and the event log (1.2) to disk. On startup,
load the latest snapshot and replay events since that snapshot's tick.

Suggested approach: serialize `WorldState` to `bincode` or `serde_json` and write to
a `sled` tree keyed by tick number. Keep only the last N snapshots to bound disk usage.

**Acceptance criteria:**
- A block placed by an agent at tick T is still present after server restart.
- Startup time for a world with 10 000 ticks is under 5 seconds.

---

### 2.4 Observability

**Current state:** `tracing` is initialised in `runtime/cli` with an env-filter.
No metrics are exported.

**Goal:**
- Emit structured `tracing` spans for every HTTP request (already partially in place
  via `tower-http`'s `TraceLayer`).
- Add Prometheus metrics via `metrics` + `metrics-exporter-prometheus`:
  - `ff_ticks_total` (counter)
  - `ff_agents_active` (gauge)
  - `ff_sandbox_executions_total` (counter, labelled by `exit_status`)
  - `ff_http_request_duration_seconds` (histogram)
- Expose metrics at `GET /metrics`.

---

## Phase 3 — Game Features

These items expand the gameplay surface.

### 3.1 Combat system

**Current state:** `AgentCapability::Combat` is defined in `foundation/types`.
No combat logic, damage model, or related events exist.

**Goal:**
- Add `WorldEvent::AgentAttacked { attacker: Uuid, target: Uuid, damage: u32, tick: u64 }`.
- Add `health: u32` to `AgentStats` (or as a first-class field on `Agent`).
- Add `POST /agents/:id/attack` endpoint accepting `{ target_id: Uuid }`.
- Combat requires `AgentCapability::Combat`; check via `check_capability()`.
- When `health` reaches 0, emit `AgentDied` and transition state to `Dead`.

---

### 3.2 Faction system

**Current state:** `EpochEnded { dominant_faction: Option<String> }` references factions
by name string. There is no `Faction` type, no faction membership, and no scoring.

**Goal:**
- Add `Faction` enum (or newtype) to `foundation/types`.
- Add `faction: Option<Faction>` to `Agent`.
- Track per-faction resource totals in `domain/economy` or a new `domain/factions` crate.
- Compute `dominant_faction` at epoch end based on total resources or territory control.

---

### 3.3 Flesh out plugin implementations

**Current state:** `plugins/plugin-agents`, `plugin-world`, `plugin-gm`, and
`plugin-economy` each contain a minimal stub that compiles but does nothing meaningful.

**Goal:** Implement at least one complete plugin so the plugin system has a real
end-to-end path. Suggested starting point: `plugin-gm` (GameMaster) — it should call
`QuestManager::generate_quest()` on each tick and emit `QuestCreated` events.

---

### 3.4 Multi-agent prompt context

**Current state:** `AgentManager::command()` passes the world state to the driver, but
agents have no awareness of other agents' positions or recent actions.

**Goal:** Build a `PromptContext` struct (in `domain/agents/src/prompt.rs`) that
includes nearby agents, recent `WorldEvent` slice, and current quest. Pass this
to `AgentDriver::complete()` so agents can react to each other.

---

## Phase 4 — Distribution

These items prepare ForgeFabrik for multi-node operation.

### 4.1 Distributed consensus

**Current state:** `ConsensusStore` is a single-node in-memory `HashMap`. It cannot
coordinate across multiple server instances and loses state on restart.

**Goal:** Back `ConsensusCoordinator` with a durable store (Redis, etcd, or a custom
append-only log over TCP). The trait interface is already clean — only the implementation
changes.

---

### 4.2 Containerisation

**Current state:** No `Dockerfile` or `docker-compose.yml` exists.

**Goal:**

```dockerfile
# Dockerfile (multi-stage)
FROM rust:1.78-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p cli

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/forgefabrik /usr/local/bin/
ENTRYPOINT ["forgefabrik"]
CMD ["serve", "--port", "8080"]
```

Add a `docker-compose.yml` that wires the server container, a volume for persistence,
and (optionally) a Redis container for future distributed consensus.

---

### 4.3 Horizontal scaling

**Current state:** `AppState` is a single `Arc<…>` shared in-process. Two instances
cannot share state.

**Goal:** Externalise the mutable state (world, agents, economy) behind the persistence
layer (2.3) and consensus layer (4.1) so that multiple `forgefabrik serve` instances can
serve traffic behind a load balancer with consistent reads.

---

## Tracking

| Item | Phase | Status |
|---|---|---|
| OS-level sandbox isolation | 1 | open |
| Persistent event log | 1 | open |
| Real sandbox snapshot / restore | 1 | open |
| CI/CD pipeline | 1 | open |
| Real-time event streaming | 2 | open |
| Epoch scheduler | 2 | open |
| WorldState persistence | 2 | open |
| Observability | 2 | open |
| Combat system | 3 | open |
| Faction system | 3 | open |
| Plugin implementations | 3 | open |
| Multi-agent prompt context | 3 | open |
| Distributed consensus | 4 | open |
| Containerisation | 4 | open |
| Horizontal scaling | 4 | open |
