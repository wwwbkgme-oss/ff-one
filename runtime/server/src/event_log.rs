//! Persistenter Eventlog — Phase 1.2 aus docs/ROADMAP.md.
//!
//! Alle `WorldEvent`-Werte werden als JSON Lines in eine append-only Datei
//! geschrieben. Beim Start kann der gesamte Log replayed werden, um den
//! `WorldState` ohne DB-Seed zu rekonstruieren.
//!
//! ## Format
//! Eine Zeile pro Event, JSON-kodiert:
//! ```json
//! {"tick":42,"hash":"0xdeadbeef","event":{...},"written_at":"2026-..."}
//! ```
//!
//! ## Pfad
//! Standard: `FORGEFABRIK_EVENT_LOG` ENV → `./forgefabrik-events.jsonl`

use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::Mutex,
};

use chrono::Utc;
use contracts::error::{FfError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use types::world::WorldEvent;

// ── Envelope ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Tick bei dem das Event emittiert wurde.
    pub tick:       u64,
    /// BLAKE3 Weltstate-Hash nach diesem Event (hex).
    pub hash:       String,
    /// Das Event selbst.
    pub event:      WorldEvent,
    /// Wall-Clock-Zeit (nur für Logging — beeinflusst keinen State).
    pub written_at: chrono::DateTime<Utc>,
}

// ── EventLog ──────────────────────────────────────────────────────────────────

/// Append-only JSON-Lines-Eventlog.
///
/// Thread-sicher via Mutex. Schreibt jedes Event sofort auf Disk (`flush`).
pub struct EventLog {
    file: Mutex<File>,
    path: PathBuf,
}

impl std::fmt::Debug for EventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventLog").field("path", &self.path).finish()
    }
}

impl EventLog {
    /// Öffnet oder erstellt den Eventlog.
    /// Pfad: `FORGEFABRIK_EVENT_LOG` ENV oder `./forgefabrik-events.jsonl`.
    pub fn open() -> Result<Self> {
        let path = std::env::var("FORGEFABRIK_EVENT_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("forgefabrik-events.jsonl"));

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| FfError::Io(format!("EventLog open {}: {e}", path.display())))?;

        info!(path = %path.display(), "EventLog: geöffnet");
        Ok(Self { file: Mutex::new(file), path })
    }

    /// Event in den Log schreiben (append + flush).
    pub fn append(&self, tick: u64, hash: u64, event: WorldEvent) -> Result<()> {
        let entry = LogEntry {
            tick,
            hash: format!("{hash:#x}"),
            event,
            written_at: Utc::now(),
        };

        let mut line = serde_json::to_string(&entry)
            .map_err(|e| FfError::Serialise(e.to_string()))?;
        line.push('\n');

        let mut file = self.file.lock()
            .map_err(|e| FfError::Internal(format!("EventLog mutex: {e}")))?;

        file.write_all(line.as_bytes())
            .and_then(|_| file.flush())
            .map_err(|e| FfError::Io(format!("EventLog write: {e}")))?;

        debug!(tick, "EventLog: event gespeichert");
        Ok(())
    }

    /// Alle gespeicherten Events laden (für Replay beim Start).
    ///
    /// Fehlerhafte Zeilen werden übersprungen (geloggt als warn).
    pub fn load_all(&self) -> Result<Vec<LogEntry>> {
        let file = File::open(&self.path)
            .map_err(|e| FfError::Io(format!("EventLog open für read: {e}")))?;

        let reader  = BufReader::new(file);
        let mut out = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| FfError::Io(e.to_string()))?;
            if line.trim().is_empty() { continue; }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => out.push(entry),
                Err(e) => warn!(line = i + 1, error = %e, "EventLog: fehlerhafte Zeile übersprungen"),
            }
        }

        info!(entries = out.len(), path = %self.path.display(), "EventLog: geladen");
        Ok(out)
    }

    /// Gibt den Pfad des Logs zurück.
    pub fn path(&self) -> &PathBuf { &self.path }
}
