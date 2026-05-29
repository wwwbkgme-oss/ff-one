//! # drivers
//!
//! **BKG-Layer:** `runtime`
//!
//! **Einzige Verantwortung:** AgentDriver-Implementierungen (HTTP-Clients).
//!
//! Die Prompts werden von `domain/agents` deterministisch aufgebaut.
//! Dieses Crate führt ausschließlich die Netzwerk-I/O durch.
pub mod claude;
pub mod mock;
pub mod openai;

pub use claude::ClaudeDriver;
pub use mock::MockDriver;
pub use openai::OpenAiDriver;
