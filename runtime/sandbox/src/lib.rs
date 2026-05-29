//! # sandbox
//!
//! **BKG-Layer:** `runtime`
//!
//! **Einzige Verantwortung:** sichere Code-Ausführung in isolierten Prozessen.
//!
//! Spawnt Kindprozesse, erzwingt Timeouts, gibt ExecutionResult zurück.
//! Keine Businesslogik — nur I/O-Infrastruktur.
pub mod executor;
pub use executor::ProcessSandbox;
