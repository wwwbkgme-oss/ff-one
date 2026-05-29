use async_trait::async_trait;
use chrono::Utc;
use contracts::{
    error::{FfError, Result},
    traits::SandboxExecutor,
};
use std::{collections::HashMap, sync::Arc};
use tempfile::TempDir;
use tokio::{process::Command, sync::RwLock, time};
use tracing::info;
use types::sandbox::{CodeLanguage, ExecutionResult, ExitStatus, SandboxConfig, SandboxState};
use uuid::Uuid;

struct Inst {
    config: SandboxConfig,
    state:  SandboxState,
    wd:     TempDir,
    snaps:  HashMap<String, Vec<u8>>,
}

/// Prozessbasierter Sandbox-Executor.
///
/// Jede Code-Ausführung wird in einem isolierten Kind-Prozess ausgeführt.
/// Ressourcen-Limits werden über Timeouts und env-clearing enforced.
pub struct ProcessSandbox {
    instances: Arc<RwLock<HashMap<Uuid, Inst>>>,
}

impl ProcessSandbox {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn interpreter(lang: &CodeLanguage) -> &'static str {
        match lang {
            CodeLanguage::Python => "python3",
            CodeLanguage::JavaScript => "node",
            CodeLanguage::Rust => "rustc",
            CodeLanguage::Bash => "bash",
            CodeLanguage::Lua => "lua",
        }
    }

    fn extension(lang: &CodeLanguage) -> &'static str {
        match lang {
            CodeLanguage::Python => "py",
            CodeLanguage::JavaScript => "js",
            CodeLanguage::Rust => "rs",
            CodeLanguage::Bash => "sh",
            CodeLanguage::Lua => "lua",
        }
    }
}

impl Default for ProcessSandbox {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SandboxExecutor for ProcessSandbox {
    async fn create(&self, config: SandboxConfig) -> Result<Uuid> {
        let id = config.id;
        let wd = tempfile::tempdir().map_err(FfError::Io)?;
        self.instances.write().await.insert(
            id,
            Inst {
                config,
                state: SandboxState::Ready,
                wd,
                snaps: HashMap::new(),
            },
        );
        info!(sandbox = %id, "sandbox created");
        Ok(id)
    }

    async fn execute(&self, id: Uuid, code: &str) -> Result<ExecutionResult> {
        // Konfiguration lesen, bevor der mutable Borrow beginnt.
        let (lang, timeout_ms, env_vars) = {
            let ins = self.instances.read().await;
            let i = ins
                .get(&id)
                .ok_or_else(|| FfError::SandboxNotFound(id.to_string()))?;
            (
                i.config.language.clone(),
                i.config.limits.timeout_ms,
                i.config.env_vars.clone(),
            )
        };

        {
            // Zustand → Running
            if let Some(i) = self.instances.write().await.get_mut(&id) {
                i.state = SandboxState::Running;
            }
        }

        // Code in temporäre Datei schreiben.
        let code_path = std::env::temp_dir().join(format!("{}.{}", id, Self::extension(&lang)));
        tokio::fs::write(&code_path, code)
            .await
            .map_err(FfError::Io)?;

        // Prozess spawnen und auf Timeout achten.
        let start = std::time::Instant::now();
        let interp = Self::interpreter(&lang);
        let mut cmd = Command::new(interp);
        cmd.arg(&code_path).env_clear().envs(&env_vars);

        let outcome =
            time::timeout(std::time::Duration::from_millis(timeout_ms), cmd.output()).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        let _ = tokio::fs::remove_file(&code_path).await;

        let result = match outcome {
            Err(_) => ExecutionResult {
                id: Uuid::new_v4(),
                sandbox_id: id,
                exit_status: ExitStatus::Timeout,
                stdout: String::new(),
                stderr: format!("Timeout nach {timeout_ms} ms"),
                duration_ms,
                memory_peak_bytes: 0,
                executed_at: Utc::now(),
            },
            Ok(Err(e)) => return Err(FfError::ExecutionFailed(e.to_string())),
            Ok(Ok(out)) => ExecutionResult {
                id: Uuid::new_v4(),
                sandbox_id: id,
                exit_status: if out.status.success() {
                    ExitStatus::Success
                } else {
                    ExitStatus::NonZero(out.status.code().unwrap_or(-1))
                },
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
                duration_ms,
                memory_peak_bytes: 0,
                executed_at: Utc::now(),
            },
        };

        {
            // Zustand → Ready
            if let Some(i) = self.instances.write().await.get_mut(&id) {
                i.state = SandboxState::Ready;
            }
        }
        Ok(result)
    }

    async fn state(&self, id: Uuid) -> Result<SandboxState> {
        self.instances
            .read()
            .await
            .get(&id)
            .map(|i| i.state.clone())
            .ok_or_else(|| FfError::SandboxNotFound(id.to_string()))
    }

    async fn destroy(&self, id: Uuid) -> Result<()> {
        self.instances
            .write()
            .await
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| FfError::SandboxNotFound(id.to_string()))
    }

    /// Archiviert das Sandbox-Arbeitsverzeichnis als tar.gz in-memory.
    /// Phase 1.3: ROADMAP.md — echter Filesystem-Snapshot.
    async fn snapshot(&self, id: Uuid) -> Result<String> {
        let snap_id = Uuid::new_v4().to_string();
        let wd_path = {
            let ins = self.instances.read().await;
            let i = ins.get(&id)
                .ok_or_else(|| FfError::SandboxNotFound(id.to_string()))?;
            i.wd.path().to_path_buf()
        };

        // tar -czf - <dir> → in-memory bytes
        let output = tokio::process::Command::new("tar")
            .args(["-C", wd_path.parent().unwrap_or(&wd_path).to_str().unwrap_or("/tmp"),
                   "-czf", "-",
                   wd_path.file_name().unwrap_or_default().to_str().unwrap_or(".")])
            .output()
            .await
            .map_err(FfError::Io)?;

        let bytes = if output.status.success() {
            output.stdout
        } else {
            // Leeres Archiv wenn kein Inhalt
            vec![]
        };

        let mut ins = self.instances.write().await;
        let i = ins.get_mut(&id)
            .ok_or_else(|| FfError::SandboxNotFound(id.to_string()))?;
        i.snaps.insert(snap_id.clone(), bytes);

        info!(sandbox = %id, snap_id = %snap_id, "Snapshot erstellt");
        Ok(snap_id)
    }

    /// Stellt einen gespeicherten Snapshot wieder her.
    /// Überschreibt den aktuellen Inhalt des Arbeitsverzeichnisses.
    async fn restore(&self, id: Uuid, snap: &str) -> Result<()> {
        let (wd_path, bytes) = {
            let ins = self.instances.read().await;
            let i = ins.get(&id)
                .ok_or_else(|| FfError::SandboxNotFound(id.to_string()))?;
            let b = i.snaps.get(snap)
                .ok_or_else(|| FfError::Other(format!("Snapshot {snap} nicht gefunden")))?
                .clone();
            (i.wd.path().to_path_buf(), b)
        };

        if bytes.is_empty() {
            info!(sandbox = %id, snap = snap, "restore: leerer Snapshot — kein Inhalt");
            return Ok(());
        }

        // Aktuelles Verzeichnis leeren
        if let Ok(mut entries) = tokio::fs::read_dir(&wd_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let _ = tokio::fs::remove_file(entry.path()).await;
            }
        }

        // tar-Archiv entpacken
        let mut child = tokio::process::Command::new("tar")
            .args(["-C", wd_path.to_str().unwrap_or("/tmp"), "-xzf", "-"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(FfError::Io)?;

        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(&bytes).await;
        }

        child.wait().await.map_err(FfError::Io)?;
        info!(sandbox = %id, snap = snap, "restore: abgeschlossen");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn make_sandbox(lang: CodeLanguage) -> (ProcessSandbox, Uuid) {
        let sb = ProcessSandbox::new();
        let cfg = SandboxConfig::new(Uuid::new_v4(), lang);
        let id = sb.create(cfg).await.unwrap();
        (sb, id)
    }

    #[tokio::test]
    async fn state_cycle() {
        let (sb, id) = make_sandbox(CodeLanguage::Python).await;
        assert_eq!(sb.state(id).await.unwrap(), SandboxState::Ready);
        sb.destroy(id).await.unwrap();
        assert!(sb.state(id).await.is_err());
    }

    #[tokio::test]
    async fn snapshot_roundtrip() {
        let (sb, id) = make_sandbox(CodeLanguage::Bash).await;
        let snap = sb.snapshot(id).await.unwrap();
        sb.restore(id, &snap).await.unwrap();
    }
}
