use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::Value;
use crate::models::StateMachine;
use super::{AiAnalyzer, prompts, json_from_text, apply_enhancement, state_machine_from_json};

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

    fn text_of(resp: &Value) -> Result<&str> {
        resp["content"][0]["text"].as_str().ok_or_else(|| anyhow!("No text in Claude response"))
    }
}

#[async_trait]
impl AiAnalyzer for ClaudeAnalyzer {
    fn provider_name(&self) -> &str { "claude" }

    async fn enhance(&self, sm: &mut StateMachine) -> Result<()> {
        let resp = self.call(&prompts::enhance_prompt(sm)).await?;
        apply_enhancement(sm, &json_from_text(Self::text_of(&resp)?)?);
        Ok(())
    }

    async fn extract_from_description(&self, description: &str) -> Result<StateMachine> {
        let resp = self.call(&prompts::extract_from_description_prompt(description)).await?;
        Ok(state_machine_from_json(&json_from_text(Self::text_of(&resp)?)?))
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
