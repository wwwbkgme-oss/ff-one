//! Anthropic Claude HTTP-Client.
use async_trait::async_trait;
use contracts::{
    error::{FfError, Result},
    traits::AgentDriver,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use types::agent::{Agent, AgentCapability};

#[derive(Serialize)]
struct Req<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Msg<'a>>,
}
#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: &'a str,
}
#[derive(Deserialize)]
struct Resp {
    content: Vec<Content>,
}
#[derive(Deserialize)]
struct Content {
    text: String,
}

pub struct ClaudeDriver {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl ClaudeDriver {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            model: "claude-opus-4-5".into(),
        }
    }
    pub fn with_model(mut self, m: impl Into<String>) -> Self {
        self.model = m.into();
        self
    }
}

#[async_trait]
impl AgentDriver for ClaudeDriver {
    fn name(&self) -> &str {
        "Claude"
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::BuildAndMine,
            AgentCapability::ExecuteCode,
            AgentCapability::Trade,
            AgentCapability::Witness,
            AgentCapability::QuestGiver,
        ]
    }

    async fn complete(&self, agent: &Agent, prompt: &str) -> Result<String> {
        let sys = agents::prompt::system_prompt(agent);
        let body = Req {
            model: &self.model,
            max_tokens: 1024,
            system: &sys,
            messages: vec![Msg {
                role: "user",
                content: prompt,
            }],
        };
        debug!(agent = %agent.name, "Claude request");
        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| FfError::Other(e.to_string()))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(FfError::Other(format!("Claude {s}: {b}")));
        }
        let data: Resp = resp
            .json()
            .await
            .map_err(|e| FfError::Other(e.to_string()))?;
        Ok(data
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join(""))
    }

    async fn generate_code(&self, agent: &Agent, task: &str, lang: &str) -> Result<String> {
        let p = agents::prompt::code_prompt(task, lang);
        self.complete(agent, &p).await
    }

    async fn is_available(&self) -> bool {
        self.client
            .get("https://api.anthropic.com")
            .send()
            .await
            .map(|r| r.status().as_u16() < 500)
            .unwrap_or(false)
    }
}
