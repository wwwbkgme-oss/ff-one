//! # economy
//!
//! **BKG-Layer:** `domain`  
//! **Konzept:** Ressourcen-Wirtschaft — Markt, Auktionen, Wallets.  
//! Quest-Logik lebt in `domain/quests` (eigenes Konzept).
pub mod market;
pub use market::EconomyStore;
