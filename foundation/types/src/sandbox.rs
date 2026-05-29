//! Sandbox configuration and execution result types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Language / runtime of submitted code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeLanguage {
    Python,
    JavaScript,
    Rust,
    Bash,
    Lua,
}

impl std::fmt::Display for CodeLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Resource limits enforced by the sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub timeout_ms: u64,
    pub max_memory_bytes: u64,
    pub max_cpu_ms: u64,
    pub allow_network: bool,
    pub allow_filesystem_write: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            timeout_ms: 5_000,
            max_memory_bytes: 128 * 1024 * 1024, // 128 MiB
            max_cpu_ms: 4_000,
            allow_network: false,
            allow_filesystem_write: false,
        }
    }
}

/// Configuration for one sandbox instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub language: CodeLanguage,
    pub limits: ResourceLimits,
    pub env_vars: std::collections::HashMap<String, String>,
}

impl SandboxConfig {
    pub fn new(agent_id: Uuid, language: CodeLanguage) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            language,
            limits: Default::default(),
            env_vars: Default::default(),
        }
    }
}

/// How a sandboxed process exited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitStatus {
    Success,
    NonZero(i32),
    Timeout,
    MemoryLimitExceeded,
    Killed { signal: i32 },
    SecurityViolation(String),
}

/// Result of one code-execution run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub id: Uuid,
    pub sandbox_id: Uuid,
    pub exit_status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub memory_peak_bytes: u64,
    pub executed_at: DateTime<Utc>,
}

impl ExecutionResult {
    pub fn success(sandbox_id: Uuid, stdout: String, duration_ms: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            sandbox_id,
            exit_status: ExitStatus::Success,
            stdout,
            stderr: String::new(),
            duration_ms,
            memory_peak_bytes: 0,
            executed_at: Utc::now(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.exit_status == ExitStatus::Success
    }
}

/// Lifecycle state of a sandbox container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxState {
    Ready,
    Running,
    Snapshotting,
    Stopped,
    Error(String),
}
