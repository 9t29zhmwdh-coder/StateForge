use once_cell::sync::Lazy;
use regex::Regex;
use crate::models::analysis::{EventSequence, LogEvent};
use uuid::Uuid;
use chrono::Utc;

// Matches common log patterns: [timestamp] [level] [service] event/state
static LOG_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?x)
        (?:(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}[^\s]*)\s+)?  # timestamp
        (?:\[?(?:INFO|DEBUG|WARN|ERROR|FATAL)\]?\s+)?               # level
        (?:\[([^\]]+)\]\s+)?                                         # [service]
        (.+)                                                         # message
        "
    ).unwrap()
});

// Detects state transitions in log messages
static TRANSITION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        (?:
            (?:state|status|phase|mode)\s+(?:changed?|transition(?:ed|ing)?|moved?|updated?)\s+(?:from\s+)?['\"]?(\w+)['\"]?\s+(?:to|->|=>)\s+['\"]?(\w+)['\"]?  |
            ['\"]?(\w+)['\"]?\s+(?:->|=>)\s+['\"]?(\w+)['\"]?   |
            (?:enter(?:ing|ed)?|transition(?:ing|ed)?(?:\s+to)?)\s+['\"]?(\w+)['\"]?  |
            (?:leaving?|exit(?:ing|ed)?)\s+['\"]?(\w+)['\"]?
        )"
    ).unwrap()
});

static EVENT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?ix)
        (?:
            (?:event|action|trigger|dispatch(?:ing|ed)?)\s+['\"]?(\w+)['\"]?  |
            received?\s+(?:event|message|command|request)\s+['\"]?(\w+)['\"]?  |
            (?:handle|process(?:ing|ed)?)\s+(?:event\s+)?['\"]?(\w+)['\"]?
        )"
    ).unwrap()
});

pub struct LogSequenceExtractor;

impl LogSequenceExtractor {
    pub fn new() -> Self { Self }

    pub fn extract(&self, content: &str) -> Vec<EventSequence> {
        let mut sequences = Vec::new();
        let mut current_events: Vec<LogEvent> = Vec::new();
        let mut session_marker = 0usize;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }

            // Session boundary detection
            if is_session_boundary(line) && !current_events.is_empty() {
                if current_events.len() >= 2 {
                    let is_error = current_events.iter().any(|e| is_error_event(&e.name));
                    sequences.push(EventSequence {
                        id: Uuid::new_v4().to_string(),
                        events: std::mem::take(&mut current_events),
                        duration_ms: None,
                        final_state: None,
                        is_error_path: is_error,
                        occurrence_count: 1,
                    });
                } else {
                    current_events.clear();
                }
                session_marker += 1;
                continue;
            }

            if let Some(event) = extract_event(line) {
                current_events.push(event);
            }
        }

        if current_events.len() >= 2 {
            let is_error = current_events.iter().any(|e| is_error_event(&e.name));
            sequences.push(EventSequence {
                id: Uuid::new_v4().to_string(),
                events: current_events,
                duration_ms: None,
                final_state: None,
                is_error_path: is_error,
                occurrence_count: 1,
            });
        }

        sequences
    }
}

fn extract_event(line: &str) -> Option<LogEvent> {
    // Try transition pattern first
    if let Some(cap) = TRANSITION_PATTERN.captures(line) {
        let from_state = cap.get(1).or(cap.get(3)).or(cap.get(6))
            .map(|m| m.as_str().to_string());
        let to_state = cap.get(2).or(cap.get(4)).or(cap.get(5))
            .map(|m| m.as_str().to_string());
        let name = to_state.clone().or(from_state.clone()).unwrap_or_else(|| "transition".to_string());

        return Some(LogEvent {
            timestamp: None,
            name,
            from_state,
            to_state,
            payload: None,
            level: detect_level(line),
        });
    }

    // Try event pattern
    if let Some(cap) = EVENT_PATTERN.captures(line) {
        let name = cap.get(1).or(cap.get(2)).or(cap.get(3))
            .map(|m| m.as_str().to_string())?;

        return Some(LogEvent {
            timestamp: None,
            name,
            from_state: None,
            to_state: None,
            payload: None,
            level: detect_level(line),
        });
    }

    None
}

fn detect_level(line: &str) -> Option<String> {
    let upper = line.to_uppercase();
    for level in &["ERROR", "FATAL", "WARN", "INFO", "DEBUG"] {
        if upper.contains(level) {
            return Some(level.to_string());
        }
    }
    None
}

fn is_session_boundary(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("session started") || lower.contains("new request") || lower.contains("--- ") || lower.contains("=== ")
}

fn is_error_event(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("error") || lower.contains("fail") || lower.contains("exception") || lower.contains("fatal")
}
