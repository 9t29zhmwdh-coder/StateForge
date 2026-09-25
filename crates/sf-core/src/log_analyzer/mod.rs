//! Rebuilds a state machine from log files.
//!
//! The analyzer this replaces mixed up events and states ("pending --> shipped
//! : paid"), read nothing from JSON lines and nothing from access logs. It now
//! reads each line for an explicit change of state, keeps the last state per
//! entity (`order=17`) for lines that only name the new one, and turns web
//! server access logs into the sequence of API calls per client.

use std::collections::{BTreeSet, HashMap};

use anyhow::{bail, Result};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::models::{AnalysisSource, StateKind, StateMachine};
use crate::parser::flow::{Decl, Edge, Flow};

pub struct LogAnalyzer;

#[derive(Debug, PartialEq)]
struct Step {
    entity: String,
    from: Option<String>,
    to: String,
    event: Option<String>,
    error: bool,
}

static FROM_TO: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\bfrom\s+['"]?([\w-]+)['"]?\s+to\s+['"]?([\w-]+)"#).unwrap());
static ARROW: Lazy<Regex> = Lazy::new(|| Regex::new(r#"['"]?([A-Za-z_][\w-]*)['"]?\s*(?:->|=>|→)\s*['"]?([A-Za-z_][\w-]*)"#).unwrap());
static TO_ONLY: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\b(?:(?:state|status|phase)\s*(?:changed|set|is|now|updated)?\s*(?:to|=|:)\s*|entered\s+|entering\s+|transition(?:ed|ing)?\s+to\s+)['"]?([A-Za-z_][\w-]*)"#).unwrap());
static EVENT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\b(?:event|action|trigger|command)\s*[=:]?\s*['"]?([\w-]+)|\(([\w-]+)\)\s*$"#).unwrap());
static ENTITY: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\b((?:\w+_)?id|order|user|session|request|req|job|task|conn(?:ection)?|device|account)\s*[=:#]?\s*['"]?([\w-]*\d[\w-]*)"#).unwrap());
static ERROR_LEVEL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)\b(?:error|fatal|critical|panic)\b"#).unwrap());
static ACCESS: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\S+) \S+ \S+ \[[^\]]+\] "([A-Z]+) (\S+)[^"]*" (\d{3})"#).unwrap());
static ID_SEGMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:\d+|[0-9a-fA-F-]{8,})$").unwrap());

const NOT_STATES: &[&str] = &["from", "to", "state", "status", "event", "the", "a", "an"];

fn entity_of(line: &str) -> String {
    ENTITY.captures(line).map(|c| format!("{}={}", c[1].to_lowercase(), &c[2])).unwrap_or_default()
}

fn event_of(line: &str) -> Option<String> {
    EVENT.captures(line).and_then(|c| c.get(1).or(c.get(2))).map(|m| m.as_str().to_string())
}

fn is_state_word(word: &str) -> bool {
    !NOT_STATES.contains(&word.to_lowercase().as_str()) && !word.chars().all(|c| c.is_ascii_digit())
}

/// One JSON object per line: `{"order":17,"event":"PAY","from":"pending","to":"paid"}`.
fn read_json(line: &str) -> Option<Step> {
    let value: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
    let obj = value.as_object()?;
    let text = |keys: &[&str]| keys.iter().find_map(|k| obj.get(*k)).and_then(|v| match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    });
    let to = text(&["to", "to_state", "new_state", "next", "state", "status"])?;
    let from = text(&["from", "from_state", "old_state", "prev", "previous"]);
    let entity = ["id", "order", "order_id", "user", "user_id", "session", "session_id", "request_id", "job", "job_id", "entity"]
        .iter()
        .find_map(|k| text(&[k]).map(|v| format!("{k}={v}")))
        .unwrap_or_default();
    let level = text(&["level", "severity", "lvl"]).unwrap_or_default();
    Some(Step { entity, from, to, event: text(&["event", "action", "type", "trigger"]), error: ERROR_LEVEL.is_match(&level) })
}

fn read_text(line: &str) -> Option<Step> {
    let (from, to) = if let Some(c) = FROM_TO.captures(line) {
        (Some(c[1].to_string()), c[2].to_string())
    } else if let Some(c) = ARROW.captures(line).filter(|c| is_state_word(&c[1]) && is_state_word(&c[2])) {
        (Some(c[1].to_string()), c[2].to_string())
    } else {
        (None, TO_ONLY.captures(line)?[1].to_string())
    };
    if !is_state_word(&to) {
        return None;
    }
    Some(Step { entity: entity_of(line), from, to, event: event_of(line), error: ERROR_LEVEL.is_match(line) })
}

/// `"POST /orders/17/pay HTTP/1.1" 200` → client, "POST /orders/{id}/pay", status
fn read_access(line: &str) -> Option<(String, String, u16)> {
    let c = ACCESS.captures(line)?;
    let path = c[3].split('?').next().unwrap_or("");
    let normalized: Vec<&str> = path.split('/').map(|seg| if ID_SEGMENT.is_match(seg) { "{id}" } else { seg }).collect();
    Some((c[1].to_string(), format!("{} {}", &c[2], normalized.join("/")), c[4].parse().ok()?))
}

impl LogAnalyzer {
    pub fn analyze(content: &str, source_path: Option<&str>) -> Result<StateMachine> {
        let source = match source_path {
            Some(p) => AnalysisSource::LogFile { path: p.to_string() },
            None => AnalysisSource::LogContent { content: content.chars().take(200).collect() },
        };
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        let access = lines.iter().filter(|l| ACCESS.is_match(l)).count();
        if access > 0 && access * 2 >= lines.len() {
            return Self::api_flow(&lines, source);
        }
        Self::state_flow(&lines, source)
    }

    fn state_flow(lines: &[&str], source: AnalysisSource) -> Result<StateMachine> {
        let mut states: Vec<String> = Vec::new();
        let mut edges = BTreeSet::new();
        let mut errors = BTreeSet::new();
        let mut last: HashMap<String, String> = HashMap::new();
        let mut initial = None;
        let add = |s: &str, states: &mut Vec<String>| if !states.iter().any(|x| x == s) { states.push(s.to_string()) };

        for step in lines.iter().filter_map(|l| read_json(l).or_else(|| read_text(l))) {
            let from = step.from.clone().or_else(|| last.get(&step.entity).cloned());
            if let Some(from) = &from {
                add(from, &mut states);
                if initial.is_none() && !last.contains_key(&step.entity) {
                    initial = Some(from.clone());
                }
            } else if initial.is_none() {
                initial = Some(step.to.clone());
            }
            add(&step.to, &mut states);
            if step.error {
                errors.insert(step.to.clone());
            }
            if let Some(from) = from.filter(|f| *f != step.to) {
                edges.insert(Edge { from, to: step.to.clone(), event: step.event.clone() });
            }
            last.insert(step.entity, step.to);
        }
        if states.len() < 2 {
            bail!("No state changes found. Expected lines such as \"from pending to paid\", \"pending -> paid\", \"state changed to paid\" or JSON with from/to fields.");
        }
        Ok(Self::assemble("LogFlow", states, edges, initial, &errors, source))
    }

    /// Access logs: each normalized endpoint is a step, and consecutive calls of
    /// the same client are the transitions. Status 4xx/5xx marks an error step.
    fn api_flow(lines: &[&str], source: AnalysisSource) -> Result<StateMachine> {
        let mut states: Vec<String> = Vec::new();
        let mut edges = BTreeSet::new();
        let mut errors = BTreeSet::new();
        let mut last: HashMap<String, String> = HashMap::new();
        for (client, call, status) in lines.iter().filter_map(|l| read_access(l)) {
            if !states.contains(&call) {
                states.push(call.clone());
            }
            if status >= 400 {
                errors.insert(call.clone());
            }
            if let Some(prev) = last.insert(client, call.clone()) {
                if prev != call {
                    edges.insert(Edge { from: prev, to: call, event: None });
                }
            }
        }
        if states.len() < 2 {
            bail!("The access log has fewer than two distinct calls.");
        }
        let initial = states.first().cloned();
        let source = match source {
            AnalysisSource::LogFile { path } => AnalysisSource::ApiSequence { content: path },
            other => other,
        };
        Ok(Self::assemble("ApiFlow", states, edges, initial, &errors, source))
    }

    fn assemble(name: &str, states: Vec<String>, edges: BTreeSet<Edge>, initial: Option<String>, errors: &BTreeSet<String>, source: AnalysisSource) -> StateMachine {
        let decl = Decl { type_name: name.into(), states };
        let flow = Flow { edges: edges.into_iter().collect(), initial };
        let mut sm = crate::parser::assemble(&decl, flow, &BTreeSet::new(), source);
        for s in sm.states.iter_mut().filter(|s| errors.contains(&s.name) && s.kind != StateKind::Initial) {
            s.kind = StateKind::Error;
        }
        sm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(sm: &StateMachine) -> Vec<(String, String, String)> {
        let name = |id: &str| sm.state_by_id(id).unwrap().name.clone();
        let mut out: Vec<_> = sm.transitions.iter()
            .map(|t| (name(&t.from_state), name(&t.to_state), t.event.clone().unwrap_or_default()))
            .collect();
        out.sort();
        out
    }

    fn e(from: &str, to: &str, ev: &str) -> (String, String, String) {
        (from.into(), to.into(), ev.into())
    }

    #[test]
    fn plain_text_with_events_and_entities() {
        let log = "2026-09-25T08:00:01Z INFO [orders] order=17 event PAY state changed from pending to paid\n\
                   2026-09-25T08:00:05Z INFO [orders] order=17 event SHIP state changed from paid to shipped\n\
                   2026-09-25T08:01:09Z ERROR [orders] order=18 event CANCEL state changed from paid to cancelled\n";
        let sm = LogAnalyzer::analyze(log, None).unwrap();
        assert_eq!(edges(&sm), [e("paid", "cancelled", "CANCEL"), e("paid", "shipped", "SHIP"), e("pending", "paid", "PAY")]);
        assert_eq!(sm.state_by_name("cancelled").unwrap().kind, StateKind::Error);
        assert_eq!(sm.state_by_name("pending").unwrap().kind, StateKind::Initial);
    }

    #[test]
    fn json_lines() {
        let log = r#"{"order":17,"event":"PAY","from":"pending","to":"paid"}
{"order":17,"event":"SHIP","status":"shipped"}"#;
        let sm = LogAnalyzer::analyze(log, None).unwrap();
        assert_eq!(edges(&sm), [e("paid", "shipped", "SHIP"), e("pending", "paid", "PAY")]);
    }

    #[test]
    fn syslog_arrows_with_event_in_parentheses() {
        let log = "Sep 25 08:00:01 host orderd[412]: order 17: pending -> paid (PAY)\n\
                   Sep 25 08:00:05 host orderd[412]: order 17: paid -> shipped (SHIP)\n";
        let sm = LogAnalyzer::analyze(log, None).unwrap();
        assert_eq!(edges(&sm), [e("paid", "shipped", "SHIP"), e("pending", "paid", "PAY")]);
    }

    #[test]
    fn target_only_lines_continue_from_the_entity_state() {
        let log = "job=7 state set to queued\njob=8 state set to queued\njob=7 state set to running\njob=7 state set to done\n";
        let sm = LogAnalyzer::analyze(log, None).unwrap();
        assert_eq!(edges(&sm), [e("queued", "running", ""), e("running", "done", "")]);
    }

    #[test]
    fn access_log_becomes_the_api_call_flow() {
        let log = r#"10.0.0.5 - - [25/Sep/2026:08:00:01 +0000] "POST /orders HTTP/1.1" 201 512 "-" "curl/8"
10.0.0.5 - - [25/Sep/2026:08:00:02 +0000] "POST /orders/17/pay HTTP/1.1" 200 64 "-" "curl/8"
10.0.0.9 - - [25/Sep/2026:08:00:03 +0000] "GET /health HTTP/1.1" 200 2 "-" "probe"
10.0.0.5 - - [25/Sep/2026:08:00:05 +0000] "POST /orders/17/ship HTTP/1.1" 409 64 "-" "curl/8"
"#;
        let sm = LogAnalyzer::analyze(log, None).unwrap();
        assert_eq!(edges(&sm), [e("POST /orders", "POST /orders/{id}/pay", ""), e("POST /orders/{id}/pay", "POST /orders/{id}/ship", "")]);
        assert_eq!(sm.state_by_name("POST /orders/{id}/ship").unwrap().kind, StateKind::Error);
    }

    #[test]
    fn nothing_recognisable_is_an_error_not_an_empty_diagram() {
        assert!(LogAnalyzer::analyze("hello\nworld\n", None).is_err());
    }
}
