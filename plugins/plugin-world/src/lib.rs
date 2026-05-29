//! `plugin-world` — Voxel world simulation.
use plugin::{abi::FfPluginCtx, export_plugin};
use tracing::{debug, info};

fn init(_: *const FfPluginCtx) -> i32 {
    info!(plugin = "plugin-world", "initialised");
    0
}
fn tick(t: u64) -> i32 {
    if t % 200 == 0 {
        debug!(plugin = "plugin-world", tick = t, "tick");
    }
    0
}
fn shutdown() -> i32 {
    info!(plugin = "plugin-world", "shutdown");
    0
}

export_plugin!(
    id:       "plugin-world",
    version:  "0.1.0",
    name:     "Reality",
    init:     init,
    tick:     tick,
    shutdown: shutdown,
);
