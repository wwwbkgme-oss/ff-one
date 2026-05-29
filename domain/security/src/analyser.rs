
use async_trait::async_trait;
use contracts::{error::Result, traits::SecurityAnalyser};
use regex::Regex;
use std::time::Instant;
use types::security::{
    FindingCategory, SafetyDecision, SecurityAssessment, SecurityFinding, Severity,
};
use uuid::Uuid;

struct Rule {
    pattern:  Regex,
    category: FindingCategory,
    severity: Severity,
    title:    &'static str,
    desc:     &'static str,
}

/// Deterministischer Regex-basierter Code-Analyser.
///
/// Kein Netzwerk, kein Wallclock-Zustand — gleicher Code → gleiche Findings.
pub struct StaticAnalyser {
    rules:     Vec<Rule>,
    threshold: u8,
}

impl StaticAnalyser {
    pub fn new() -> Self {
        let rules = vec![
            Rule { pattern: Regex::new(r"(?i)(os\.system|subprocess\.|exec\(|eval\()").unwrap(),      category: FindingCategory::DangerousSyscall, severity: Severity::High,     title: "OS-Befehlsausführung",    desc: "Möglicher Shell-Aufruf." },
            Rule { pattern: Regex::new(r"(?i)(socket\.|urllib|requests\.|http\.client|fetch\()").unwrap(), category: FindingCategory::NetworkAccess,   severity: Severity::Medium,   title: "Netzwerkzugriff",         desc: "Verbindungsversuch erkannt." },
            Rule { pattern: Regex::new(r"(?i)(open\s*\(|pathlib|shutil\.|os\.path)").unwrap(),         category: FindingCategory::FilesystemAccess, severity: Severity::Low,      title: "Dateisystemzugriff",      desc: "Lese- oder Schreiboperation." },
            Rule { pattern: Regex::new(r"(?i)(import\s+ctypes|cffi|mmap\.)").unwrap(),                 category: FindingCategory::DangerousSyscall, severity: Severity::Critical, title: "Nativer Speicherzugriff", desc: "ctypes/cffi verwendet." },
            Rule { pattern: Regex::new(r"while\s+True|while\s+1").unwrap(),                            category: FindingCategory::ResourceAbuse,    severity: Severity::Medium,   title: "Mögliche Endlosschleife", desc: "CPU-Erschöpfung möglich." },
            Rule { pattern: Regex::new(r"(?i)rm\s+-rf|shutil\.rmtree").unwrap(),                       category: FindingCategory::FilesystemAccess, severity: Severity::High,     title: "Rekursives Löschen",      desc: "Dateien werden rekursiv gelöscht." },
        ];
        Self { rules, threshold: 50 }
    }

    fn decide_inner(&self, findings: &[SecurityFinding]) -> SafetyDecision {
        if findings.iter().any(|f| f.severity == Severity::Critical) {
            return SafetyDecision::Reject { reason: "Kritischer Fund".into() };
        }
        let score: u8 = findings.iter()
            .map(|f| match f.severity { Severity::Info => 1u8, Severity::Low => 5, Severity::Medium => 15, Severity::High => 30, Severity::Critical => 50 })
            .fold(0u8, |a, b| a.saturating_add(b));
        if score >= self.threshold { SafetyDecision::Reject  { reason: format!("Risiko-Score {score}") } }
        else if score > 0          { SafetyDecision::ExecuteRestricted { reason: format!("Risiko-Score {score}") } }
        else                       { SafetyDecision::Safe }
    }
}

impl Default for StaticAnalyser {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl SecurityAnalyser for StaticAnalyser {
    async fn assess(&self, code: &str, _language: &str, agent_id: Uuid) -> Result<SecurityAssessment> {
        let t0 = Instant::now();
        let mut findings = Vec::new();
        for (line_no, line) in code.lines().enumerate() {
            for r in &self.rules {
                if r.pattern.is_match(line) {
                    let mut f = SecurityFinding::new(r.category.clone(), r.severity, r.title, r.desc);
                    f.code_snippet = Some(line.trim().to_string());
                    f.line_number  = Some(line_no as u32 + 1);
                    findings.push(f);
                }
            }
        }
        let decision = self.decide_inner(&findings);
        let mut a = SecurityAssessment::new(agent_id, findings, decision);
        a.analysis_duration_ms = t0.elapsed().as_millis() as u64;
        Ok(a)
    }

    fn decide(&self, a: &SecurityAssessment) -> SafetyDecision {
        self.decide_inner(&a.findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn clean_code_is_safe() {
        let a = StaticAnalyser::new();
        let r = a.assess("x = 1 + 2\nprint(x)\n", "python", Uuid::new_v4()).await.unwrap();
        assert_eq!(r.decision, SafetyDecision::Safe);
    }

    #[tokio::test]
    async fn syscall_produces_finding() {
        let a = StaticAnalyser::new();
        let r = a.assess("import os\nos.system('ls')\n", "python", Uuid::new_v4()).await.unwrap();
        assert!(!r.findings.is_empty());
    }

    #[tokio::test]
    async fn ctypes_is_critical() {
        let a = StaticAnalyser::new();
        let r = a.assess("import ctypes\n", "python", Uuid::new_v4()).await.unwrap();
        assert!(matches!(r.decision, SafetyDecision::Reject { .. }));
    }
}
