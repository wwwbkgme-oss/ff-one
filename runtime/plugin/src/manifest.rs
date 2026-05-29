use serde::Deserialize;
use std::path::Path;
use types::plugin::{PluginCapability, PluginManifest};
#[derive(Debug, Deserialize)]
pub struct ManifestFile {
    pub plugin: PS,
    pub capabilities: CS,
    pub entry: ES,
}
#[derive(Debug, Deserialize)]
pub struct PS {
    pub id: String,
    pub version: String,
    pub name: String,
    pub description: String,
}
#[derive(Debug, Deserialize)]
pub struct CS {
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub requires: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct ES {
    pub lib: String,
}
pub fn parse_manifest(path: impl AsRef<Path>) -> anyhow::Result<PluginManifest> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let f: ManifestFile = toml::from_str(&content)?;
    let dir = path.as_ref().parent().unwrap_or(Path::new("."));
    Ok(PluginManifest {
        id: f.plugin.id,
        version: f.plugin.version,
        name: f.plugin.name,
        description: f.plugin.description,
        provides: f.capabilities.provides.iter().map(|s| cap(s)).collect(),
        requires: f.capabilities.requires.iter().map(|s| cap(s)).collect(),
        lib: dir.join(&f.entry.lib).to_string_lossy().into_owned(),
    })
}
fn cap(s: &str) -> PluginCapability {
    match s {
        "agent" | "agents" => PluginCapability::Agent,
        "sandbox" => PluginCapability::Sandbox,
        "security" => PluginCapability::Security,
        "game-mode" | "gamemode" => PluginCapability::GameMode,
        "physics" => PluginCapability::Physics,
        "economy" => PluginCapability::Economy,
        "render" => PluginCapability::Render,
        "ui" => PluginCapability::Ui,
        "consensus" => PluginCapability::Consensus,
        o => PluginCapability::Custom(o.to_string()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn parse_ok() {
        let toml = "[plugin]\nid=\"x\"\nversion=\"1\"\nname=\"X\"\ndescription=\"d\"\n\
                  [capabilities]\nprovides=[\"agent\"]\nrequires=[]\n[entry]\nlib=\"x.so\"\n";
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("Plugin.toml");
        std::fs::File::create(&p)
            .unwrap()
            .write_all(toml.as_bytes())
            .unwrap();
        let m = parse_manifest(&p).unwrap();
        assert_eq!(m.id, "x");
    }
}
