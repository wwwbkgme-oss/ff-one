//! Security analysis types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Criticality of a security finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Category of a detected issue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingCategory {
    DangerousSyscall,
    NetworkAccess,
    FilesystemAccess,
    CodeInjection,
    ResourceAbuse,
    InfoDisclosure,
    WeakCrypto,
    Custom(String),
}

/// One security issue found in agent-submitted code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id:           Uuid,
    pub category:     FindingCategory,
    pub severity:     Severity,
    pub title:        String,
    pub description:  String,
    pub code_snippet: Option<String>,
    pub line_number:  Option<u32>,
    pub detected_at:  DateTime<Utc>,
}

impl SecurityFinding {
    pub fn new(
        category: FindingCategory,
        severity: Severity,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            category,
            severity,
            title: title.into(),
            description: description.into(),
            code_snippet: None,
            line_number: None,
            detected_at: Utc::now(),
        }
    }
}

/// Final safety verdict for a code submission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyDecision {
    Safe,
    ExecuteRestricted { reason: String },
    Reject { reason: String },
}

/// Aggregate assessment for one submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAssessment {
    pub id:                   Uuid,
    pub submission_id:        Uuid,
    pub findings:             Vec<SecurityFinding>,
    pub decision:             SafetyDecision,
    pub risk_score:           u8,
    pub assessed_at:          DateTime<Utc>,
    pub analysis_duration_ms: u64,
}

impl SecurityAssessment {
    pub fn new(
        submission_id: Uuid,
        findings: Vec<SecurityFinding>,
        decision: SafetyDecision,
    ) -> Self {
        let risk_score = findings
            .iter()
            .map(|f| match f.severity {
                Severity::Info => 1u8,
                Severity::Low => 5,
                Severity::Medium => 15,
                Severity::High => 30,
                Severity::Critical => 50,
            })
            .fold(0u8, |a, b| a.saturating_add(b));

        Self {
            id: Uuid::new_v4(),
            submission_id,
            findings,
            decision,
            risk_score,
            assessed_at: Utc::now(),
            analysis_duration_ms: 0,
        }
    }

    pub fn is_safe(&self) -> bool {
        matches!(
            self.decision,
            SafetyDecision::Safe | SafetyDecision::ExecuteRestricted { .. }
        )
    }
}
