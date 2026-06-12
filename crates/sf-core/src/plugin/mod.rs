use crate::models::{StateMachine, Language};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

pub trait ParserPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn language(&self) -> Language;
    fn can_parse(&self, content: &str, path: Option<&str>) -> bool;
    fn parse(&self, content: &str, path: Option<&str>) -> Result<StateMachine>;
}

pub struct PluginRegistry {
    parsers: HashMap<String, Arc<dyn ParserPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { parsers: HashMap::new() }
    }

    pub fn register(&mut self, plugin: Arc<dyn ParserPlugin>) {
        self.parsers.insert(plugin.id().to_string(), plugin);
    }

    pub fn detect_and_parse(&self, content: &str, path: Option<&str>) -> Option<Result<StateMachine>> {
        for plugin in self.parsers.values() {
            if plugin.can_parse(content, path) {
                return Some(plugin.parse(content, path));
            }
        }
        None
    }
}

impl Default for PluginRegistry {
    fn default() -> Self { Self::new() }
}
