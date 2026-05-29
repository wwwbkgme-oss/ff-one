//! # types
//!
//! **BKG-Layer:** `foundation`
//!
//! **Einzige Verantwortung:** reine Domänentypen — kein Trait, keine Logik, kein I/O.
//!
//! Jedes andere Crate, das einen Domänentyp benötigt, importiert ihn aus hier.
//! Es gibt genau eine kanonische Definition jedes Konzepts (Single Source of Truth).
//!
//! ## Abhängigkeiten
//! - Keine internen Crates.
//! - Nur externe: `serde`, `uuid`, `blake3`, `chrono`.

pub mod agent;
pub mod block;
pub mod consensus;
pub mod economy;
pub mod plugin;
pub mod position;
pub mod quest;
pub mod sandbox;
pub mod security;
pub mod world;

pub use agent::*;
pub use block::*;
pub use consensus::*;
pub use economy::*;
pub use plugin::*;
pub use position::*;
pub use quest::*;
pub use sandbox::*;
pub use security::*;
pub use world::*;
