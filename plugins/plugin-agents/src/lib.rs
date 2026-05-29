//! `plugin-agents` — Agent lifecycle.
use plugin::{abi::FfPluginCtx, export_plugin};
use tracing::{debug, info};

fn init(_: *const FfPluginCtx) -> i32 {
    info!(plugin = "plugin-agents", "initialised");
    0
}
fn tick(t: u64) -> i32 {
    if t.is_multiple_of(200) {
        debug!(plugin = "plugin-agents", tick = t, "tick");
    }
    0
}
fn shutdown() -> i32 {
    info!(plugin = "plugin-agents", "shutdown");
    0
}

export_plugin!(
    id:       "plugin-agents",
    version:  "0.1.0",
    name:     "AgentOS",
    init:     init,
    tick:     tick,
    shutdown: shutdown,
);
