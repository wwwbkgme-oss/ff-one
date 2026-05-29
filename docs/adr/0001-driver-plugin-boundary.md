# ADR-0001 — Driver / Plugin Boundary

| Field | Value |
|---|---|
| **Status** | accepted |
| **Date** | 2026-05-29 |
| **Deciders** | BKG architecture review |
| **Related** | [`docs/DRIVER_PLUGIN_BOUNDARY.md`](../DRIVER_PLUGIN_BOUNDARY.md) |

---

## Context

When free-tier LLM backends (Groq, Cerebras, SambaNova, OpenRouter, Ollama) were
added to ForgeFabrik, a `plugin-free-llm` cdylib was created to house them. This
raised the question: should LLM provider adapters be plugins or drivers?

The existing plugin system (`runtime/plugin`, `plugins/`) was built for
**domain behaviour extensions** — game-mode expansions that change what agents,
worlds, and economies *do*. The four shipped plugins (agents, world, gm, economy)
are all domain-level.

LLM provider adapters are different: they are **transport adapters** that perform
I/O (HTTPS calls) and implement exactly one `foundation/contracts` trait
(`AgentDriver`). They contain no domain logic, no game state, and no deterministic
business rules.

---

## Decision

**LLM provider adapters are drivers, not plugins.**

1. They live in `runtime/drivers/` (compiled into the binary).
2. They are wired at startup by `AppState::new()` based on env-var discovery.
3. They are never loaded or unloaded at runtime.
4. The `plugin-free-llm` cdylib was removed.

**`AgentKind` uses a grouped `Free(FreeProvider)` variant** to prevent provider
explosion. Adding a new free-tier backend requires a `FreeProvider` variant, a
factory function in `runtime/drivers/free.rs`, and a kind-string mapping in
`handlers.rs` — not a new top-level `AgentKind` variant.

---

## Rationale

| Criterion | Plugin | Driver |
|---|---|---|
| Contains I/O | ❌ forbidden | ✅ allowed |
| Hot-swappable at runtime | ✅ | ❌ not needed |
| Implements domain behaviour | ✅ required | ❌ none |
| Loaded via C ABI | ✅ | ❌ direct Rust |
| Wired from env vars | ❌ | ✅ |

LLM adapters score 0/2 on plugin criteria and 2/2 on driver criteria.

Keeping them as drivers also avoids a second plugin taxonomy (domain plugins vs.
infrastructure plugins), which would make the plugin system harder to reason about.

---

## Consequences

**Positive:**
- Clear mental model: plugins = game behaviour, drivers = I/O adapters.
- New LLM providers require ≤ 4 file edits, no new crate.
- `AppState` owns all driver wiring in one place.
- No C-ABI overhead for transport adapters.

**Negative / trade-offs:**
- LLM backends cannot be hot-swapped without restarting the server.
  (Acceptable: env-var changes already require a restart.)
- `FreeProvider` enum in `foundation/types` grows with each new backend.
  Mitigated by the grouping pattern — `AgentKind` stays stable.

---

## References

- [`docs/DRIVER_PLUGIN_BOUNDARY.md`](../DRIVER_PLUGIN_BOUNDARY.md) — full spec
- `runtime/drivers/free.rs` — implementation
- `foundation/types/src/agent.rs` — `FreeProvider` + `AgentKind`
