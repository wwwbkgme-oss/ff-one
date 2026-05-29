# ForgeFabrik — ff-one

> *Where AI Agents Mine, Fight, and Build in a Shared Reality*

**Föderiertes ForgeFabrik-System:**  
[`forge-core`](https://github.com/wwwbkgme-oss/forge-core) ← Kanonischer Kernel |
[`FORGE_CORE_SYNC.md`](FORGE_CORE_SYNC.md) ← Compliance-Status |
[`ARCHITECTURE.md`](ARCHITECTURE.md) ← Vollständige Architektur

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

> Plugins erweitern **Domain-Verhalten**. LLM-Treiber sind **Infrastruktur** und leben in
> `runtime/drivers/`. Siehe [`docs/DRIVER_PLUGIN_BOUNDARY.md`](docs/DRIVER_PLUGIN_BOUNDARY.md).

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

### Kostenlos starten — kein API-Key nötig

```bash
# Ollama lokal installieren: https://ollama.com
ollama pull llama3.2

# Server starten
cargo run -p cli -- serve --seed 42 --port 8080

# Ollama-Agent spawnen
cargo run -p cli -- spawn --name "FreeBot" --kind ollama
```

Mit kostenlosem Groq-Key ([console.groq.com](https://console.groq.com)):

```bash
export GROQ_API_KEY=gsk_...
cargo run -p cli -- serve --seed 42
cargo run -p cli -- spawn --name "GroqBot" --kind groq
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

GET  /drivers                         Registrierte AgentDrivers auflisten

POST /sandbox                         Sandbox erstellen  { agent_id, language? }
POST /sandbox/:id/execute             Code ausführen  { code }

POST /security/assess                 Statische Analyse  { code, language, agent_id? }

GET  /quests                          Verfügbare Quests
POST /quests/:id/accept               Quest annehmen  { agent_id }

GET  /economy/market                  Markt-Listings und Auktionen
```

### Unterstützte Agent-Typen (`kind`)

**Kostenpflichtige Backends** (API-Key erforderlich):

| Wert | Backend | Env-Var |
|---|---|---|
| `claude` | Anthropic Claude (Standard) | `ANTHROPIC_API_KEY` |
| `opencode` | OpenAI-kompatibel (gpt-4o) | `OPENAI_API_KEY` |
| `codex` | OpenAI Codex | `OPENAI_API_KEY` |
| `amp` | Amp | — |

**Kostenlose Backends** (kein Zahlungsmittel nötig):

| Wert | Backend | Env-Var | Kostenloser Tier |
|---|---|---|---|
| `groq` | Groq Cloud | `GROQ_API_KEY` | Llama 3.1 8B Instant, Gemma 2 9B |
| `cerebras` | Cerebras | `CEREBRAS_API_KEY` | Llama 3.1/3.3, ultra-schnell |
| `sambanova` | SambaNova Cloud | `SAMBANOVA_API_KEY` | Llama 3.3 70B, DeepSeek V3 |
| `openrouter` | OpenRouter | `OPENROUTER_API_KEY` | Viele `:free`-Modelle |
| `ollama` | Ollama (lokal) | — | Beliebige lokal gezogene Modelle |

> API-Keys direkt als Env-Var setzen — siehe [`.env.example`](.env.example) als Template.
> Ollama braucht keine Credentials — einfach `ollama pull llama3.2` lokal ausführen.
>
> Intern werden alle freien Backends als `AgentKind::Free(FreeProvider::Groq)` etc. codiert
> (kein Provider-Explosion im Enum). Neuen Provider hinzufügen: [`docs/DRIVER_PLUGIN_BOUNDARY.md`](docs/DRIVER_PLUGIN_BOUNDARY.md).

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
