//! Prompt-Templates — deterministisch, rein funktional.
//!
//! Wandelt Domänen-Typen in Strings für den KI-Backend-Aufruf um.
//! Kein I/O, keine Zufälligkeit, kein Wallclock-Zeit-Aufruf.

use types::{agent::Agent, world::WorldState};

/// System-Prompt für einen Agent-Kontext.
pub fn system_prompt(agent: &Agent) -> String {
    format!(
        "Du bist {name}, ein {kind}-Agent in ForgeFabrik bei {pos}. \
         Level {level}, XP {xp}. Fähigkeiten: {caps:?}.\n\
         ForgeFabrik ist eine deterministische Voxel-Welt. \
         Du kannst Blöcke abbauen, bauen, Code im Sandbox ausführen, handeln und Quests absolvieren.\n\
         Sei präzise. Bei Code: nur den Code ausgeben, kein Kommentar.",
        name  = agent.name,
        kind  = agent.kind,
        pos   = agent.position,
        level = agent.level(),
        xp    = agent.stats.xp,
        caps  = agent.capabilities,
    )
}

/// Aufgaben-Prompt für einen konkreten Befehl.
pub fn task_prompt(task: &str, world: &WorldState) -> String {
    format!(
        "Tick: {} | Epoch: {} | Seed: {}\nAufgabe: {}",
        world.tick, world.epoch.number, world.seed, task
    )
}

/// Code-Generierungs-Prompt.
pub fn code_prompt(task: &str, language: &str) -> String {
    format!(
        "Generiere {language}-Code für eine ForgeFabrik-Sandbox.\n\
         Gib NUR den Code aus — keine Erklärungen, keine Markdown-Fence.\n\
         Aufgabe: {task}"
    )
}
