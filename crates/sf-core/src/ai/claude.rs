use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::Value;
use crate::models::{StateMachine, State, Transition, StateKind, AnalysisSource, Language};
use super::{AiAnalyzer, prompts};
use chrono::Utc;
use uuid::Uuid;

const MODEL: &str = "claude-haiku-4-5-20251001";
const API: &str = "https://api.anthropic.com";

pub struct ClaudeAnalyzer {
    api_key: String,
    client: Client,
}

impl ClaudeAnalyzer {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), client: Client::new() }
    }

    async fn call(&self, prompt: &str) -> Result<Value> {
        let body = serde_json::json!({
            "model": MODEL,
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": prompt}]
        });
        let resp = self.client
            .post(format!("{}/v1/messages", API))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Claude error: {}", resp.status()));
        }
        Ok(resp.json().await?)
    }

    fn extract_json(resp: &Value) -> Result<Value> {
        let text = resp["content"][0]["text"].as_str().ok_or_else(|| anyhow!("No text"))?;
        let start = text.find('{').ok_or_else(|| anyhow!("No JSON"))?;
        let end = text.rfind('}').ok_or_else(|| anyhow!("No JSON end"))?;
        Ok(serde_json::from_str(&text[start..=end])?)
    }
}

#[async_trait]
impl AiAnalyzer for ClaudeAnalyzer {
    fn provider_name(&self) -> &str { "claude" }

    async fn enhance(&self, sm: &mut StateMachine) -> Result<()> {
        let prompt = prompts::enhance_prompt(sm);
        let resp = self.call(&prompt).await?;
        let j = Self::extract_json(&resp)?;

        sm.ai_summary = j["summary"].as_str().map(str::to_string);

        // Apply state descriptions
        if let Some(descs) = j["state_descriptions"].as_object() {
            for state in &mut sm.states {
                if let Some(desc) = descs.get(&state.name).and_then(|d| d.as_str()) {
                    state.description = Some(desc.to_string());
                }
            }
        }

        // Mark error states
        if let Some(error_paths) = j["error_paths"].as_array() {
            for ep in error_paths {
                if let Some(name) = ep.as_str() {
                    if let Some(s) = sm.states.iter_mut().find(|s| s.name == name) {
                        s.kind = StateKind::Error;
                    }
                }
            }
        }

        Ok(())
    }

    async fn extract_from_description(&self, description: &str) -> Result<StateMachine> {
        let prompt = prompts::extract_from_description_prompt(description);
        let resp = self.call(&prompt).await?;
        let j = Self::extract_json(&resp)?;

        let name = j["name"].as_str().unwrap_or("DescriptionMachine");
        let mut sm = StateMachine::new(name, AnalysisSource::Manual);

        let mut state_id_map = std::collections::HashMap::new();

        if let Some(states) = j["states"].as_array() {
            for sv in states {
                let sname = sv["name"].as_str().unwrap_or("Unknown");
                let kind = match sv["kind"].as_str().unwrap_or("normal") {
                    "initial" => StateKind::Initial,
                    "final"   => StateKind::Final,
                    "error"   => StateKind::Error,
                    _         => crate::parser::helpers::state_kind_from_name(sname),
                };
                let mut s = State::new(sname, kind);
                s.description = sv["description"].as_str().map(str::to_string);
                state_id_map.insert(sname.to_string(), s.id.clone());
                sm.add_state(s);
            }
        }

        if let Some(transitions) = j["transitions"].as_array() {
            for tv in transitions {
                let from = tv["from"].as_str().unwrap_or("");
                let to   = tv["to"].as_str().unwrap_or("");
                let event = tv["event"].as_str().map(str::to_string);

                if let (Some(from_id), Some(to_id)) = (state_id_map.get(from), state_id_map.get(to)) {
                    let mut t = Transition::new(from_id, to_id, event);
                    t.guard = tv["guard"].as_str().map(str::to_string);
                    t.actions = tv["actions"].as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                        .unwrap_or_default();
                    sm.add_transition(t);
                }
            }
        }

        Ok(sm)
    }

    async fn is_available(&self) -> bool {
        self.client.get(format!("{}/v1/models", API))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
