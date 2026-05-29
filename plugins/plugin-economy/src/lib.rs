//! `plugin-economy` — Resource economy.
use plugin::{abi::FfPluginCtx, export_plugin};
use tracing::{debug, info};

fn init(_: *const FfPluginCtx) -> i32 {
    info!(plugin = "plugin-economy", "initialised");
    0
}
fn tick(t: u64) -> i32 {
    if t % 200 == 0 { debug!(plugin = "plugin-economy", tick = t, "tick"); }
    0
}
fn shutdown() -> i32 {
    info!(plugin = "plugin-economy", "shutdown");
    0
}

export_plugin!(
    id:       "plugin-economy",
    version:  "0.1.0",
    name:     "Economy",
    init:     init,
    tick:     tick,
    shutdown: shutdown,
);
