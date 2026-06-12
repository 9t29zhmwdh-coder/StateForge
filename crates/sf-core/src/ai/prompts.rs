use crate::models::StateMachine;

pub fn enhance_prompt(sm: &StateMachine) -> String {
    let state_names: Vec<_> = sm.states.iter().map(|s| s.name.as_str()).collect();
    let transitions: Vec<_> = sm.transitions.iter().take(20).map(|t| {
        let from = sm.state_by_id(&t.from_state).map(|s| s.name.as_str()).unwrap_or("?");
        let to = sm.state_by_id(&t.to_state).map(|s| s.name.as_str()).unwrap_or("?");
        let event = t.event.as_deref().unwrap_or("auto");
        format!("{} --[{}]--> {}", from, event, to)
    }).collect();

    format!(
        r#"You are a state machine expert. Analyze this state machine and improve it.

Name: {}
States: {}
Transitions:
{}

Respond ONLY with JSON:
{{
  "summary": "One paragraph describing what this state machine models",
  "state_descriptions": {{
    "StateName": "What this state represents"
  }},
  "missing_states": ["state1", "state2"],
  "missing_transitions": [
    {{"from": "State", "to": "State", "event": "EventName", "reason": "why needed"}}
  ],
  "error_paths": ["StateName that should be marked as error"],
  "issues": ["issue description"],
  "confidence": 0.85
}}"#,
        sm.name,
        state_names.join(", "),
        transitions.join("\n")
    )
}

pub fn extract_from_description_prompt(description: &str) -> String {
    format!(
        r#"You are a state machine expert. Extract a complete state machine from this description.

Description:
{}

Respond ONLY with JSON:
{{
  "name": "MachineName",
  "states": [
    {{"name": "StateName", "kind": "initial|normal|final|error", "description": "what it means"}}
  ],
  "transitions": [
    {{"from": "State", "to": "State", "event": "EventName", "guard": "optional condition", "actions": []}}
  ],
  "initial_state": "StateName",
  "context_type": "optional data context description"
}}"#,
        description
    )
}
