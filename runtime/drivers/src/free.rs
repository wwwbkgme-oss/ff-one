//! Free-tier LLM provider factories.
//!
//! **BKG-Layer:** `runtime`
//!
//! All providers use OpenAI-compatible `/v1/chat/completions` endpoints and
//! are backed by [`OpenAiDriver`]. No credit card is required for any of them.
//!
//! # Provider overview
//!
//! | Provider   | Env var              | Base URL                               | Default model                           |
//! |------------|----------------------|----------------------------------------|-----------------------------------------|
//! | Groq       | `GROQ_API_KEY`       | `https://api.groq.com/openai/v1`       | `llama-3.1-8b-instant`                  |
//! | SambaNova  | `SAMBANOVA_API_KEY`  | `https://api.sambanova.ai/v1`          | `Meta-Llama-3.3-70B-Instruct`           |
//! | Ollama     | —                    | `OLLAMA_BASE_URL` or localhost:11434   | `OLLAMA_MODEL` or `llama3.2`            |
//! | OpenRouter | `OPENROUTER_API_KEY` | `https://openrouter.ai/api/v1`         | `meta-llama/llama-3.1-8b-instruct:free` |
//! | Cerebras   | `CEREBRAS_API_KEY`   | `https://api.cerebras.ai/v1`           | `llama3.1-8b`                           |

use crate::openai::OpenAiDriver;
use contracts::traits::AgentDriver;
use std::sync::Arc;
use tracing::info;

// ── Groq ─────────────────────────────────────────────────────────────────────

/// Groq Cloud — free API key at <https://console.groq.com>.
/// Default model: `llama-3.1-8b-instant` (fast, free tier).
pub fn groq(api_key: impl Into<String>) -> OpenAiDriver {
    OpenAiDriver::new("Groq", "https://api.groq.com/openai/v1", api_key, "llama-3.1-8b-instant")
}

// ── SambaNova ────────────────────────────────────────────────────────────────

/// SambaNova Cloud — free tier, no credit card.
/// Sign up at <https://cloud.sambanova.ai>.
/// Default model: `Meta-Llama-3.3-70B-Instruct`.
pub fn sambanova(api_key: impl Into<String>) -> OpenAiDriver {
    OpenAiDriver::new("SambaNova", "https://api.sambanova.ai/v1", api_key, "Meta-Llama-3.3-70B-Instruct")
}

// ── Ollama (local) ───────────────────────────────────────────────────────────

/// Local Ollama — entirely free, runs on your own hardware.
/// Install Ollama at <https://ollama.com>, then `ollama pull <model>`.
/// Override URL with `OLLAMA_BASE_URL`, model with `OLLAMA_MODEL`.
pub fn ollama_default() -> OpenAiDriver {
    let base  = std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434/v1".into());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".into());
    // Ollama's OpenAI-compat endpoint ignores the bearer token but requires a non-empty value.
    OpenAiDriver::new("Ollama", base, "ollama", model)
}

// ── OpenRouter ───────────────────────────────────────────────────────────────

/// OpenRouter — free account at <https://openrouter.ai>.
/// Models with `:free` suffix cost $0. Default: `meta-llama/llama-3.1-8b-instruct:free`.
pub fn openrouter(api_key: impl Into<String>) -> OpenAiDriver {
    OpenAiDriver::new("OpenRouter", "https://openrouter.ai/api/v1", api_key, "meta-llama/llama-3.1-8b-instruct:free")
}

// ── Cerebras ─────────────────────────────────────────────────────────────────

/// Cerebras inference — free tier, no credit card.
/// Sign up at <https://cloud.cerebras.ai>.
/// Default model: `llama3.1-8b`.
pub fn cerebras(api_key: impl Into<String>) -> OpenAiDriver {
    OpenAiDriver::new("Cerebras", "https://api.cerebras.ai/v1", api_key, "llama3.1-8b")
}

// ── Auto-loader ──────────────────────────────────────────────────────────────

/// Reads free-provider keys from environment variables and returns one
/// [`Arc<dyn AgentDriver>`] per discovered provider.
///
/// Discovery order (fastest / most capable first):
/// 1. Groq          — `GROQ_API_KEY`
/// 2. Cerebras      — `CEREBRAS_API_KEY`
/// 3. SambaNova     — `SAMBANOVA_API_KEY`
/// 4. OpenRouter    — `OPENROUTER_API_KEY`
/// 5. Ollama        — always included (localhost fallback)
pub fn load_free_drivers() -> Vec<Arc<dyn AgentDriver>> {
    let mut out: Vec<Arc<dyn AgentDriver>> = Vec::new();

    if let Ok(k) = std::env::var("GROQ_API_KEY") {
        info!(provider = "groq", "registering free driver");
        out.push(Arc::new(groq(k)));
    }
    if let Ok(k) = std::env::var("CEREBRAS_API_KEY") {
        info!(provider = "cerebras", "registering free driver");
        out.push(Arc::new(cerebras(k)));
    }
    if let Ok(k) = std::env::var("SAMBANOVA_API_KEY") {
        info!(provider = "sambanova", "registering free driver");
        out.push(Arc::new(sambanova(k)));
    }
    if let Ok(k) = std::env::var("OPENROUTER_API_KEY") {
        info!(provider = "openrouter", "registering free driver");
        out.push(Arc::new(openrouter(k)));
    }

    info!(provider = "ollama", "registering free driver (local)");
    out.push(Arc::new(ollama_default()));

    out
}
