//! # core
//!
//! **BKG-Layer:** `foundation`
//!
//! **Einzige Verantwortung:** Fehler-Enum, Trait-Verträge, World-Events.
//!
//! Abhängigkeiten nach oben: **keine** — nur `foundation/types` und externe Crates.
//!
//! ## Inhalte
//! - [`error`] — [`FfError`] + [`Result<T>`] alias  
//! - [`traits`] — alle Trait-Verträge (AgentDriver, WorldSimulator, …)  
//! - [`events`] — [`WorldEvent`]-Enum (Events sind Wahrheit)

pub mod error;
pub mod events;
pub mod traits;

pub use error::{FfError, Result};
