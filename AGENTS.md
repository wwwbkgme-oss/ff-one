# AGENTS.md — ff-one (ForgeFabrik MMO / World Engine)

**Treat this repository as part of a federated ForgeFabrik system.**

Maintain compatibility with the canonical event + domain model defined in
[forge-core](https://github.com/wwwbkgme-oss/forge-core).
Do not introduce incompatible abstractions.

---

## This repo's role

`ff-one` is the **Runtime MMO / World Engine** of the ForgeFabrik federation.

| Responsibility | Location |
|---|---|
| Deterministic voxel world simulation | `domain/world` |
| Agent state machine + prompt dispatch | `domain/agents` |
| BLAKE3 world-state consensus | `domain/consensus` |
| Static code security analysis | `domain/security` |
| Resource market economy | `domain/economy` |
| Quest generation and lifecycle | `domain/quests` |
| Free-tier LLM driver adapters | `runtime/drivers/free.rs` |
| Axum HTTP REST API | `runtime/server` |
| CLI binary (`forgefabrik`) | `runtime/cli` |

---

## Canonical types implemented here

| Canonical name | Local type | File |
|---|---|---|
| `Agent` | `Agent` | `foundation/types/src/agent.rs` |
| `AgentKind` / `FreeProvider` | `AgentKind`, `FreeProvider` | same |
| `WorldState` | `WorldState` | `foundation/types/src/world.rs` |
| `WorldEvent` | `WorldEvent` | `foundation/contracts/src/events.rs` |
| `ForgeError` | `FfError` | `foundation/contracts/src/error.rs` |
| `ForgeFabrikPlugin` | `PluginRecord` | `foundation/types/src/plugin.rs` |

> **Note:** `WorldTick` and `WorldSnapshot` are tracked in [`docs/ROADMAP.md`](docs/ROADMAP.md)
> as a pending addition to align with forge-core's canonical primitives.

---

## Layer rules

```
foundation → domain → runtime → plugins
```

| Layer | Allowed | Forbidden |
|---|---|---|
| `foundation/` | types, traits, errors, events | I/O, randomness, business logic |
| `domain/` | deterministic logic, reducers | HTTP, DB, `Utc::now()`, `thread_rng()` |
| `runtime/` | I/O, HTTP, processes, plugins, drivers | domain business logic |
| `plugins/` | domain-behaviour extensions | I/O of any kind |

**Plugin vs Driver boundary (frozen):**

```
plugins/         = domain-behaviour extensions  (pure, no I/O)
runtime/drivers/ = infrastructure I/O adapters  (HTTP, keys, transport)
```

Full spec: [`docs/DRIVER_PLUGIN_BOUNDARY.md`](docs/DRIVER_PLUGIN_BOUNDARY.md)
ADR: [`docs/adr/0001-driver-plugin-boundary.md`](docs/adr/0001-driver-plugin-boundary.md)

---

## Event-First mandate

```
Events are truth. State is projection.
Command → Event → Reducer → State Projection
```

- Every state mutation in `domain/` must return at least one `WorldEvent`.
- Reducers are pure functions — no side effects, no I/O.
- Same events + same seed → same `WorldState` hash (Replay-Safety).

---

## Determinism rules

Forbidden in `domain/` and `foundation/`:

- `chrono::Utc::now()` used to affect state (logging only is OK)
- `rand::thread_rng()` — use seeded RNG only
- Global mutable state

---

## Free-tier LLM providers

New free-tier providers go in `runtime/drivers/free.rs`.
Add a `FreeProvider` variant, a factory function, and an env-var mapping.
Do **not** create a new `AgentKind` top-level variant (provider explosion).

Reference: [`docs/DRIVER_PLUGIN_BOUNDARY.md`](docs/DRIVER_PLUGIN_BOUNDARY.md)

---

## Federation links

- [`forge-core`](https://github.com/wwwbkgme-oss/forge-core) — canonical definitions
- [`docs/SYNC_CONTRACT.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/SYNC_CONTRACT.md) — federation-wide sync contract
- [`docs/PLUGIN_ABI.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/PLUGIN_ABI.md) — canonical plugin ABI
