//! OpenAI-kompatibler HTTP-Client (OpenCode, Codex, Cursor, etc.).
use async_trait::async_trait;
use contracts::{
    error::{FfError, Result},
    traits::AgentDriver,
};
use serde::{Deserialize, Serialize};
use types::agent::{Agent, AgentCapability};

#[derive(Serialize)]
struct Req<'a> {
    model: &'a str,
    messages: Vec<Msg<'a>>,
    max_tokens: u32,
}
#[derive(Serialize)]
struct Msg<'a> {
    role: &'a str,
    content: &'a str,
}
#[derive(Deserialize)]
struct Resp {
    choices: Vec<Choice>,
}
#[derive(Deserialize)]
struct Choice {
    message: MsgResp,
}
#[derive(Deserialize)]
struct MsgResp {
    content: String,
}

pub struct OpenAiDriver {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
    driver_name: String,
}

impl OpenAiDriver {
    pub fn new(
        name: impl Into<String>,
        url: impl Into<String>,
        key: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            driver_name: name.into(),
            base_url: url.into(),
            api_key: key.into(),
            model: model.into(),
        }
    }
    pub fn opencode(key: impl Into<String>) -> Self {
        Self::new("OpenCode", "https://api.openai.com/v1", key, "gpt-4o")
    }
    pub fn codex(key: impl Into<String>) -> Self {
        Self::new(
            "Codex",
            "https://api.openai.com/v1",
            key,
            "code-davinci-002",
        )
    }
}

#[async_trait]
impl AgentDriver for OpenAiDriver {
    fn name(&self) -> &str {
        &self.driver_name
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::BuildAndMine,
            AgentCapability::ExecuteCode,
            AgentCapability::Trade,
            AgentCapability::Witness,
        ]
    }

    async fn complete(&self, agent: &Agent, prompt: &str) -> Result<String> {
        let sys = agents::prompt::system_prompt(agent);
        let body = Req {
            model: &self.model,
            max_tokens: 1024,
            messages: vec![
                Msg {
                    role: "system",
                    content: &sys,
                },
                Msg {
                    role: "user",
                    content: prompt,
                },
            ],
        };
        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| FfError::Other(e.to_string()))?;
        if !resp.status().is_success() {
            let s = resp.status();
            let b = resp.text().await.unwrap_or_default();
            return Err(FfError::Other(format!("{} {s}: {b}", self.driver_name)));
        }
        let data: Resp = resp
            .json()
            .await
            .map_err(|e| FfError::Other(e.to_string()))?;
        Ok(data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }

    async fn generate_code(&self, agent: &Agent, task: &str, lang: &str) -> Result<String> {
        self.complete(agent, &agents::prompt::code_prompt(task, lang))
            .await
    }

    async fn is_available(&self) -> bool {
        true
    }
}
