pub mod claude;
pub mod prompts;

use async_trait::async_trait;
use anyhow::Result;
use crate::models::StateMachine;

#[async_trait]
pub trait AiAnalyzer: Send + Sync {
    fn provider_name(&self) -> &str;
    async fn enhance(&self, sm: &mut StateMachine) -> Result<()>;
    async fn extract_from_description(&self, description: &str) -> Result<StateMachine>;
    async fn is_available(&self) -> bool;
}
