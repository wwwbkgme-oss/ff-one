//! Plugin manifest and runtime-state types.

use serde::{Deserialize, Serialize};

/// Capabilities a plugin can provide or require.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    Agent,
    Sandbox,
    Security,
    GameMode,
    Physics,
    Economy,
    Render,
    Ui,
    Consensus,
    Custom(String),
}

impl std::fmt::Display for PluginCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "custom:{s}"),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// Parsed `Plugin.toml` manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub provides: Vec<PluginCapability>,
    pub requires: Vec<PluginCapability>,
    /// Path to the compiled shared library.
    pub lib: String,
}

/// Runtime lifecycle state of a loaded plugin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    Loaded,
    Initialised,
    Running,
    Paused,
    Stopped,
    Error(String),
}

/// Host-side metadata for one loaded plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRecord {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub lib_path: String,
}

impl PluginRecord {
    pub fn new(manifest: PluginManifest, lib_path: impl Into<String>) -> Self {
        Self {
            manifest,
            state: PluginState::Loaded,
            lib_path: lib_path.into(),
        }
    }
}
