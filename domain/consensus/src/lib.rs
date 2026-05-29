//! # consensus
//!
//! **BKG-Layer:** `domain`  
//! **Konzept:** BLAKE3-basiertes Weltzustand-Konsens-Protokoll.
pub mod coordinator;
pub use coordinator::ConsensusStore;
