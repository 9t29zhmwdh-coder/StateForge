use crate::models::analysis::{EventSequence, FlowGroup};
use uuid::Uuid;

pub struct FlowGrouper;

impl FlowGrouper {
    pub fn new() -> Self { Self }

    pub fn group(&self, sequences: &[EventSequence]) -> Vec<FlowGroup> {
        if sequences.is_empty() { return Vec::new(); }

        let mut groups: Vec<FlowGroup> = Vec::new();

        'outer: for seq in sequences {
            let template = make_template(seq);
            for group in &mut groups {
                if group.template == template {
                    group.sequences.push(seq.clone());
                    group.frequency += 1;
                    if seq.is_error_path {
                        let errors = group.sequences.iter().filter(|s| s.is_error_path).count();
                        group.error_rate = errors as f32 / group.sequences.len() as f32;
                    }
                    continue 'outer;
                }
            }

            // New group
            groups.push(FlowGroup {
                id: Uuid::new_v4().to_string(),
                name: make_group_name(seq),
                sequences: vec![seq.clone()],
                template,
                frequency: 1,
                avg_duration_ms: seq.duration_ms.map(|d| d as f64),
                error_rate: if seq.is_error_path { 1.0 } else { 0.0 },
            });
        }

        groups.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        groups
    }
}

fn make_template(seq: &EventSequence) -> String {
    seq.events.iter()
        .map(|e| e.name.as_str())
        .collect::<Vec<_>>()
        .join(" → ")
}

fn make_group_name(seq: &EventSequence) -> String {
    let events = &seq.events;
    if events.is_empty() {
        return "empty flow".to_string();
    }
    let first = &events[0].name;
    let last = &events[events.len() - 1].name;
    if events.len() == 1 {
        first.clone()
    } else {
        format!("{} → {}", first, last)
    }
}
