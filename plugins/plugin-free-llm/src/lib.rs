//! # plugin-free-llm
//!
//! **ForgeFabrik plugin — free-tier LLM provider registry.**
//!
//! Declares the `free-llm` capability and logs which providers are reachable
//! when the plugin is loaded.  The actual [`AgentDriver`] instances are wired
//! in `runtime/server`'s [`AppState::new`] via [`drivers::load_free_drivers`];
//! this plugin is the declarative anchor in the plugin system.
//!
//! ## Providers
//!
//! | Provider   | Env var              | Notes                                    |
//! |------------|----------------------|------------------------------------------|
//! | Groq       | `GROQ_API_KEY`       | Fast Llama 3 / Gemma 2 inference         |
//! | Cerebras   | `CEREBRAS_API_KEY`   | Ultra-fast RDU inference                 |
//! | SambaNova  | `SAMBANOVA_API_KEY`  | Llama 3.3 / DeepSeek V3                  |
//! | OpenRouter | `OPENROUTER_API_KEY` | Free models have `:free` suffix           |
//! | Ollama     | —                    | Local; override via `OLLAMA_BASE_URL`    |
//!
//! ## Quick start
//!
//! ```bash
//! # Spawn an Ollama-backed agent (no key needed — runs locally)
//! cargo run -p cli -- spawn --name "LocalBot" --kind ollama
//!
//! # Spawn a Groq-backed agent (set GROQ_API_KEY first)
//! GROQ_API_KEY=gsk_… cargo run -p cli -- serve --seed 42
//! cargo run -p cli -- spawn --name "GroqBot" --kind groq
//! ```

use plugin::{abi::FfPluginCtx, export_plugin};
use tracing::{info, warn};

const KEYED_PROVIDERS: &[(&str, &str)] = &[
    ("Groq",       "GROQ_API_KEY"),
    ("Cerebras",   "CEREBRAS_API_KEY"),
    ("SambaNova",  "SAMBANOVA_API_KEY"),
    ("OpenRouter", "OPENROUTER_API_KEY"),
];

fn init(_: *const FfPluginCtx) -> i32 {
    let mut available: Vec<&str> = Vec::new();
    let mut missing:   Vec<&str> = Vec::new();

    for &(name, var) in KEYED_PROVIDERS {
        if std::env::var(var).is_ok() {
            available.push(name);
        } else {
            missing.push(name);
        }
    }
    available.push("Ollama"); // always present (localhost fallback)

    info!(plugin = "plugin-free-llm", providers = ?available, "free-LLM plugin ready");

    if !missing.is_empty() {
        warn!(
            plugin  = "plugin-free-llm",
            missing = ?missing,
            "set the corresponding env vars to enable these providers"
        );
    }
    0
}

fn tick(_: u64) -> i32 { 0 }

fn shutdown() -> i32 {
    info!(plugin = "plugin-free-llm", "shutdown");
    0
}

export_plugin!(
    id:       "plugin-free-llm",
    version:  "0.1.0",
    name:     "FreeLLM",
    init:     init,
    tick:     tick,
    shutdown: shutdown,
);
