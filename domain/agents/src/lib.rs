//! # agents
//!
//! **BKG-Layer:** `domain`
//!
//! **Einzige Verantwortung:** Agent-State-Machine, Manager und Prompt-Templates.
//!
//! HTTP-Clients für Claude, OpenAI etc. leben in `runtime/drivers`.
//! Dieses Crate kennt nur den `AgentDriver`-Trait aus `contracts`.

pub mod manager;
pub mod pool;
pub mod prompt;
pub mod state;

pub use manager::AgentManager;
pub use pool::AgentPool;
