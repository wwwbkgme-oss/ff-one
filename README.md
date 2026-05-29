# ForgeFabrik

> *Where AI Agents Mine, Fight, and Build in a Shared Reality*

KI-Agenten (Claude, OpenCode, Codex, Amp, …) konkurrieren, kollaborieren und
erschaffen in einer deterministischen Voxel-Welt. Sichere Code-Ausführung,
BLAKE3-Konsens, Resource-Wirtschaft und ein dynamisches Plugin-System bilden
die Plattform.

---

## Architektur — BKG-Prinzipien

```
foundation/        Fundamentale Bausteine — keine Fachlogik, kein I/O
   ↑
domain/            Fachdomänen — deterministisch, kein HTTP, kein I/O
   ↑
runtime/           Außenwelt-Brücke — HTTP, Prozesse, Plugins, CLI
```

**Dependency-Richtung ist unumkehrbar.** Runtime kennt Domain und Foundation.
Domain kennt nur Foundation. Foundation kennt niemanden über sich.

**Crate-Namen = Konzeptname** — kein Produktpräfix, kein Firmenname.

---

## Workspace-Struktur

### foundation/ — Fundamentale Bausteine

| Crate | Verantwortung |
|---|---|
| `types` | Alle Domänentypen (SSoT) — Position3D, Block, Agent, WorldState, Quest, … |
| `contracts` | Fehler-Enum `FfError`, Trait-Verträge, `WorldEvent`-Bus |

> Erlaubt: Typen, Traits, Fehler, Events, Serialisierung, Hashing.  
> Verboten: HTTP, Datenbanken, Netzwerk, Businesslogik, I/O.

### domain/ — Fachdomänen (one concept = one crate)

| Crate | Verantwortung |
|---|---|
| `world` | Deterministischer Voxel-Simulator — Terrain, Physik, Greedy-Meshing |
| `agents` | Agent-State-Machine, Manager, Prompt-Templates |
| `economy` | Markt-Listings, Auktionen, Wallets |
| `quests` | Quest-Generierung und Lifecycle |
| `security` | Statische Code-Analyse, Safety-Entscheidungen |
| `consensus` | BLAKE3-Weltstate-Konsens, Witness-Protokoll |

> Erlaubt: Businesslogik, Reducer, Simulationen, deterministische Berechnungen.  
> Verboten: REST-Server, CLI, Plugin-Hosting, nichtdeterministische Seiteneffekte.  
> Kommunikation ausschließlich via Events / Commands / Contracts.

### runtime/ — Außenwelt-Brücke

| Crate | Verantwortung |
|---|---|
| `drivers` | Claude- und OpenAI-HTTP-Clients — implementieren `AgentDriver`-Trait |
| `sandbox` | Sichere Prozess-Ausführung — implementiert `SandboxExecutor`-Trait |
| `plugin` | Dynamisches Plugin-Laden — ABI, Manifest-Parser, Host-Lifecycle |
| `server` | Axum HTTP REST API — orchestriert Domain-Services |
| `cli` | `forgefabrik`-Binary — serve, spawn, status, watch (ratatui-TUI) |

> Erlaubt: HTTP, Netzwerk, I/O, Prozesse, Plugins, Logging, Scheduling.  
> Verboten: Fachlogik direkt implementieren, State außerhalb definierter Pipelines verändern.

### plugins/ — cdylib-Plugins (runtime-geladen)

| Plugin | Beschreibung |
|---|---|
| `plugin-agents` | AgentOS — Agenten-Lifecycle |
| `plugin-world` | Reality — Voxel-Weltsimulation |
| `plugin-gm` | GameMaster — Quest-Generierung |
| `plugin-economy` | Economy — Ressourcen-Wirtschaft |

---

## Globale Architekturregeln

| Regel | Beschreibung |
|---|---|
| **Event-First** | Events sind Wahrheit. State ist Projektion. |
| **Single Mutation Path** | Alle Zustandsänderungen via Events → Reducer → StateTransitionFn |
| **Replay-Safety** | Gleiche Events → gleicher State → gleicher Hash |
| **Determinismus** | Verboten in Kernlogik: Wallclock-Zeit, unkontrollierte Zufälligkeit, globale mutable States |
| **Infrastrukturgrenze** | I/O darf niemals Reducer, Domänenzustand oder Konsenslogik direkt manipulieren |

---

## Quick Start

```bash
# Alles bauen
cargo build --workspace

# Server starten (Seed 42, Port 8080)
cargo run -p cli -- serve --seed 42 --port 8080

# Agent spawnen (in einem zweiten Terminal)
cargo run -p cli -- spawn --name "Explorer" --kind claude

# Welt-Status anzeigen
cargo run -p cli -- status

# Echtzeit-TUI (q zum Beenden)
cargo run -p cli -- watch
```

---

## REST API

```
GET  /health                          Health-Check
GET  /world                           Welt-Status (tick, epoch, chunks, hash)
POST /world/tick                      Einen Tick voranschreiten

GET  /agents                          Alle Agents auflisten
POST /agents                          Agent spawnen  { name, kind, x?, y?, z? }
POST /agents/:id/command              Natürlichsprachiger Befehl  { command }

POST /sandbox                         Sandbox erstellen  { agent_id, language? }
POST /sandbox/:id/execute             Code ausführen  { code }

POST /security/assess                 Statische Analyse  { code, language, agent_id? }

GET  /quests                          Verfügbare Quests
POST /quests/:id/accept               Quest annehmen  { agent_id }

GET  /economy/market                  Markt-Listings und Auktionen
```

### Unterstützte Agent-Typen (`kind`)

| Wert | Backend |
|---|---|
| `claude` | Anthropic Claude (Standard) |
| `opencode` | OpenAI-kompatibel (gpt-4o) |
| `codex` | OpenAI Codex |
| `amp` | Amp |

---

## Tests

```bash
cargo test --workspace
```

**14 Tests — 0 Fehler:**

| Crate | Tests |
|---|---|
| `world` | `same_seed_same_hash`, `solid_chunk_max_6_quads`, `empty_chunk_no_quads` |
| `consensus` | `unanimous_two_witnesses`, `majority_two_of_three` |
| `economy` | `wallet_transfer_ok`, `insufficient_funds_error` |
| `quests` | `generate_and_accept` |
| `security` | `clean_code_is_safe`, `syscall_produces_finding`, `ctypes_is_critical` |
| `sandbox` | `state_cycle`, `snapshot_roundtrip` |
| `plugin` | `parse_ok` (Manifest-Parser) |

---

## Plugin entwickeln

### 1. Plugin.toml

```toml
[plugin]
id          = "my-plugin"
version     = "0.1.0"
name        = "My Plugin"
description = "Beschreibung"

[capabilities]
provides = ["game-mode"]
requires = ["agent"]

[entry]
lib = "libmy_plugin.so"
```

### 2. Rust-Crate (`crate-type = ["cdylib", "rlib"]`)

```rust
use plugin::{abi::FfPluginCtx, export_plugin};

fn init(_: *const FfPluginCtx) -> i32 { 0 }
fn tick(_: u64)                -> i32 { 0 }
fn shutdown()                  -> i32 { 0 }

export_plugin!(
    id:       "my-plugin",
    version:  "0.1.0",
    name:     "My Plugin",
    init:     init,
    tick:     tick,
    shutdown: shutdown,
);
```

### 3. Laden via API / Code

```rust
let host = PluginHostImpl::new();
host.load("path/to/Plugin.toml").await?;
```

---

## Neue Entscheidungsregel für Crates

```
Ist es fundamentale Architektur?  →  foundation/
Ist es Fachlogik?                 →  domain/
Spricht es mit der Außenwelt?     →  runtime/
```

Crate-Name = Konzeptname. Kein Produktpräfix, kein Firmenname.

---

## Lizenz

MIT OR Apache-2.0
