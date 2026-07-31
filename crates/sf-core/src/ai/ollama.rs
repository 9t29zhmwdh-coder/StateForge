//! Lokales Backend gegen eine Ollama-Instanz.
//!
//! Bis 1.0.12 bot die Oberflaeche eine Ollama-Auswahl an, ohne dass es dieses
//! Modul gab: `get_analyzer` baute unabhaengig von der Einstellung den
//! Claude-Client. Wer "lokal" waehlte, schickte seine Zustandsmaschine
//! trotzdem an Anthropic.

use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use crate::models::StateMachine;
use super::{AiAnalyzer, prompts, json_from_text, apply_enhancement, state_machine_from_json};

pub struct OllamaAnalyzer {
    base_url: String,
    model: String,
    client: Client,
}

impl OllamaAnalyzer {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            client: Client::new(),
        }
    }

    fn base(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    async fn call(&self, prompt: &str) -> Result<String> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "format": "json"
        });

        let resp = self.client
            .post(format!("{}/api/generate", self.base()))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Ollama error: {}", resp.status()));
        }

        let json: serde_json::Value = resp.json().await?;
        Ok(json["response"].as_str().unwrap_or_default().to_string())
    }
}

#[async_trait]
impl AiAnalyzer for OllamaAnalyzer {
    fn provider_name(&self) -> &str { "ollama" }

    async fn enhance(&self, sm: &mut StateMachine) -> Result<()> {
        let text = self.call(&prompts::enhance_prompt(sm)).await?;
        apply_enhancement(sm, &json_from_text(&text)?);
        Ok(())
    }

    async fn extract_from_description(&self, description: &str) -> Result<StateMachine> {
        let text = self.call(&prompts::extract_from_description_prompt(description)).await?;
        Ok(state_machine_from_json(&json_from_text(&text)?))
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base()))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_trailing_slash_in_the_url_does_not_produce_a_double_slash() {
        let a = OllamaAnalyzer::new("http://localhost:11434/", "llama3");
        assert_eq!(a.base(), "http://localhost:11434");
        let b = OllamaAnalyzer::new("http://localhost:11434", "llama3");
        assert_eq!(b.base(), "http://localhost:11434");
    }

    #[tokio::test]
    async fn an_unreachable_instance_reports_unavailable_rather_than_hanging() {
        // Port 1 nimmt nichts entgegen; der Aufruf muss zurueckkommen und
        // false liefern, nicht bis zum Standard-Timeout blockieren.
        let a = OllamaAnalyzer::new("http://127.0.0.1:1", "llama3");
        assert!(!a.is_available().await);
    }
}
