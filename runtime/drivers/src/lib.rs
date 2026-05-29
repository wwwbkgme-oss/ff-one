//! # drivers
//!
//! **BKG-Layer:** `runtime`
//!
//! **Einzige Verantwortung:** AgentDriver-Implementierungen (HTTP-Clients).
//!
//! Die Prompts werden von `domain/agents` deterministisch aufgebaut.
//! Dieses Crate führt ausschließlich die Netzwerk-I/O durch.
//!
//! ## Free-tier providers
//!
//! [`free::load_free_drivers`] auto-discovers free providers from env vars
//! and returns ready-to-register `Arc<dyn AgentDriver>` instances.
pub mod claude;
pub mod free;
pub mod mock;
pub mod openai;

pub use claude::ClaudeDriver;
pub use free::load_free_drivers;
pub use mock::MockDriver;
pub use openai::OpenAiDriver;
