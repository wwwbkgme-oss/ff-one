//! # ff-types
//!
//! **Single responsibility:** pure domain data types for ForgeFabrik.
//!
//! No business logic, no traits, no I/O.  Every other crate that needs a
//! domain type depends on this one — there is exactly one canonical definition
//! for each concept (Single Source of Truth).

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

// Flat re-exports so callers can write `ff_types::Agent` instead of
// `ff_types::agent::Agent`.
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
