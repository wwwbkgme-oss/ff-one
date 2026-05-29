//! Mock-Treiber für Tests und lokale Entwicklung.
//! Kein Netzwerk — gibt vorkonfigurierte Antworten zurück.
use async_trait::async_trait;
use contracts::{error::Result, traits::AgentDriver};
use types::agent::{Agent, AgentCapability};

pub struct MockDriver {
    pub response: String,
}

impl MockDriver {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

#[async_trait]
impl AgentDriver for MockDriver {
    fn name(&self) -> &str {
        "Mock"
    }
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![AgentCapability::BuildAndMine, AgentCapability::ExecuteCode]
    }
    async fn complete(&self, _agent: &Agent, _prompt: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    async fn generate_code(&self, _: &Agent, task: &str, lang: &str) -> Result<String> {
        Ok(format!(
            "# Mock: {lang} für Aufgabe: {task}\nprint('done')\n"
        ))
    }
    async fn is_available(&self) -> bool {
        true
    }
}
