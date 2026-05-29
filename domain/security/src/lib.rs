//! # security
//!
//! **BKG-Layer:** `domain`  
//! **Konzept:** deterministische statische Code-Analyse.  
//! Kein I/O — reine Muster-Erkennung via Regex.
pub mod analyser;
pub use analyser::StaticAnalyser;
