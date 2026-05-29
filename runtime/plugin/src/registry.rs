use std::collections::HashMap;
use types::plugin::{PluginCapability, PluginRecord, PluginState};
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, PluginRecord>,
}
impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, r: PluginRecord) {
        self.plugins.insert(r.manifest.id.clone(), r);
    }
    pub fn remove(&mut self, id: &str) -> Option<PluginRecord> {
        self.plugins.remove(id)
    }
    pub fn get(&self, id: &str) -> Option<&PluginRecord> {
        self.plugins.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &PluginRecord> {
        self.plugins.values()
    }
    pub fn available_capabilities(&self) -> Vec<PluginCapability> {
        let mut caps = Vec::new();
        for r in self.plugins.values() {
            if matches!(r.state, PluginState::Initialised | PluginState::Running) {
                caps.extend(r.manifest.provides.clone());
            }
        }
        caps.sort_by_key(|c| c.to_string());
        caps.dedup();
        caps
    }
}
