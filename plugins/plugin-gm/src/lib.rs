//! `plugin-gm` — Quest generation.
use plugin::{abi::FfPluginCtx, export_plugin};
use tracing::{debug, info};

fn init(_: *const FfPluginCtx) -> i32 {
    info!(plugin = "plugin-gm", "initialised");
    0
}
fn tick(t: u64) -> i32 {
    if t % 200 == 0 { debug!(plugin = "plugin-gm", tick = t, "tick"); }
    0
}
fn shutdown() -> i32 {
    info!(plugin = "plugin-gm", "shutdown");
    0
}

export_plugin!(
    id:       "plugin-gm",
    version:  "0.1.0",
    name:     "GameMaster",
    init:     init,
    tick:     tick,
    shutdown: shutdown,
);
