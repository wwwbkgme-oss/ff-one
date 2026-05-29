# ForgeFabrik — Driver / Plugin Boundary Spec

**Status:** frozen  
**Scope:** entire workspace  
**Rule:** this line is never crossed without an explicit architectural decision record (ADR).

---

## One-sentence summary

> **Plugins extend domain behaviour. Drivers adapt infrastructure.**

If you are unsure which one to write, answer the question below and stop reading.

```
Does the new code change what agents/world/economy/quests DO?
  yes → plugin
  no  → driver (or plain runtime module)
```

---

## Definitions

### Driver

A **driver** is a runtime adapter that translates between ForgeFabrik's internal trait
contract and an external system. It lives in `runtime/` and is always compiled into
the binary.

Properties:

- Implements exactly one `foundation/contracts` trait.
- Performs I/O (HTTP, TCP, subprocess, filesystem, `dlopen`).
- Is determinism-free — it is the explicit non-deterministic boundary.
- Is wired at startup by `AppState::new()` based on env-var discovery.
- Is never loaded or unloaded at runtime after the server starts.

Examples:

| Driver | Trait | Location |
|---|---|---|
| `ClaudeDriver` | `AgentDriver` | `runtime/drivers/claude.rs` |
| `OpenAiDriver` | `AgentDriver` | `runtime/drivers/openai.rs` |
| `groq()` / `ollama_default()` / … | `AgentDriver` | `runtime/drivers/free.rs` |
| `ProcessSandbox` | `SandboxExecutor` | `runtime/sandbox/executor.rs` |
| `PluginHostImpl` | `PluginHost` | `runtime/plugin/host.rs` |

---

### Plugin

A **plugin** is a dynamically-loaded `cdylib` that extends domain-level game behaviour.
It lives in `plugins/` and is loaded at runtime via `PluginHostImpl`.

Properties:

- Extends game-mode behaviour: what agents can do, how the world evolves,
  what quests exist, how the economy operates.
- Exposes exactly three C-ABI symbols: `ff_plugin_init`, `ff_plugin_tick`,
  `ff_plugin_shutdown`.
- Is deterministic within its tick logic — no uncontrolled wallclock time,
  no random I/O without going through a `foundation/contracts` trait.
- Declares capabilities in `Plugin.toml` (`provides`, `requires`).
- Is loaded and unloaded at runtime without restarting the server.

Examples:

| Plugin | Capability | Responsibility |
|---|---|---|
| `plugin-agents` | `agent` | Agent lifecycle hooks |
| `plugin-world` | `world` | Per-tick world simulation extensions |
| `plugin-gm` | `game-mode` | Quest generation |
| `plugin-economy` | `economy` | Resource market behaviour |

---

## The line

```
┌──────────────────────────────────────────────────────────────┐
│  foundation/                                                 │
│    types · contracts · events                                │
│    (pure data — no I/O, no behaviour)                        │
├──────────────────────────────────────────────────────────────┤
│  domain/                                                     │
│    world · agents · economy · quests · security · consensus  │
│    (deterministic business logic — no I/O)                   │
├───────────────────────────────── ← PLUGIN BOUNDARY ─────────┤
│  plugins/          cdylib  — extends domain behaviour        │
│    plugin-agents · plugin-world · plugin-gm · plugin-economy │
├───────────────────────────────── ← DRIVER BOUNDARY ─────────┤
│  runtime/          compiled-in — adapts infrastructure       │
│    drivers/  sandbox/  plugin_host/  server/  cli/           │
│    (HTTP · processes · dlopen · TCP — all I/O lives here)    │
└──────────────────────────────────────────────────────────────┘
```

The two boundaries are distinct and must never merge:

| | Plugin boundary | Driver boundary |
|---|---|---|
| **What crosses** | Domain behaviour | Infrastructure adapters |
| **Mechanism** | `dlopen` / C ABI | Direct Rust impl + env-var wiring |
| **Loaded** | At runtime, hot-swappable | At startup, fixed for server lifetime |
| **Determinism** | Required in tick logic | Not required |
| **State** | Must be projectable from events | Stateless adapters |

---

## Forbidden patterns

### ❌ Infrastructure plugin

```toml
# Plugin.toml — WRONG
[capabilities]
provides = ["free-llm"]   # LLM transport is not a game capability
```

```rust
// WRONG: a plugin that wires HTTP clients
fn init(_: *const FfPluginCtx) -> i32 {
    register_driver(GroqDriver::new(env::var("GROQ_API_KEY")));
    0
}
```

Why it's wrong: plugins are loaded after drivers. Plugins cannot call back into
`AppState` to register drivers. More fundamentally, adding a new LLM backend is
an infrastructure concern, not a change in game behaviour.

**Correct placement:** `runtime/drivers/free.rs`, wired in `AppState::new()`.

---

### ❌ Domain logic in a driver

```rust
// WRONG: business rule in a driver
impl AgentDriver for ClaudeDriver {
    async fn complete(&self, agent: &Agent, prompt: &str) -> Result<String> {
        if agent.stats.gold_earned < 100 {
            return Err(FfError::Other("agent too poor to use Claude".into()));
        }
        // ...
    }
}
```

Why it's wrong: the poverty check is a domain rule that belongs in
`domain/agents`. A driver must be a pure I/O adapter — it never makes
game-semantic decisions.

---

### ❌ I/O in a plugin tick

```rust
// WRONG: HTTP call inside a plugin tick
fn tick(t: u64) -> i32 {
    reqwest::blocking::get("https://api.example.com/quest-ideas"); // never do this
    0
}
```

Why it's wrong: tick is called on every world tick inside the main loop.
I/O in a tick blocks the simulation and breaks determinism.
If a plugin needs external data it must fetch it in `init` and cache it,
or delegate through a `foundation/contracts` trait that a driver satisfies.

---

## Decision flowchart

```
New code to add
│
├─ Does it perform I/O (HTTP / subprocess / dlopen / filesystem)?
│    yes → runtime/  (driver or sandbox or server)
│
├─ Does it change game behaviour (agents / world / economy / quests)?
│    yes → plugins/  (if hot-swappable) or domain/ (if always-on)
│
├─ Is it a pure type, trait, error, or event definition?
│    yes → foundation/
│
└─ Is it deterministic business logic with no I/O?
     yes → domain/
```

---

## Adding a new LLM provider (reference procedure)

1. Add a variant to `FreeProvider` in `foundation/types/src/agent.rs`.
2. Add a factory function in `runtime/drivers/free.rs`.
3. Add the env-var check in `load_free_drivers()`.
4. Add the `kind` string mapping in `runtime/server/src/handlers.rs`.
5. Update `FreeProvider::fmt` so it returns the same string as `AgentDriver::name()`.
6. **No plugin, no new crate** — three files, one PR.

---

## Adding a new game-mode extension (reference procedure)

1. Create `plugins/plugin-<name>/` with a `Cargo.toml` and `Plugin.toml`.
2. Implement `ff_plugin_init`, `ff_plugin_tick`, `ff_plugin_shutdown`.
3. Declare `provides` and `requires` in `Plugin.toml`.
4. Add to `[workspace] members` in the root `Cargo.toml`.
5. **No driver wiring, no HTTP** — pure domain logic only.
