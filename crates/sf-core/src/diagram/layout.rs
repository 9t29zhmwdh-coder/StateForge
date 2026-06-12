use std::collections::HashMap;
use crate::models::StateMachine;

const NODE_W: f32 = 120.0;
const NODE_H: f32 = 44.0;
const H_GAP: f32 = 60.0;
const V_GAP: f32 = 80.0;
const PADDING: f32 = 40.0;

/// Layered layout (Sugiyama-inspired): assign states to ranks, then position.
pub fn compute_layout(sm: &StateMachine) -> HashMap<String, (f32, f32)> {
    let mut positions = HashMap::new();
    if sm.states.is_empty() { return positions; }

    let ranks = assign_ranks(sm);
    let max_rank = *ranks.values().max().unwrap_or(&0);

    // Group states by rank
    let mut by_rank: HashMap<usize, Vec<String>> = HashMap::new();
    for (id, rank) in &ranks {
        by_rank.entry(*rank).or_default().push(id.clone());
    }

    for rank in 0..=max_rank {
        let states = match by_rank.get(&rank) {
            Some(s) => s,
            None => continue,
        };
        let count = states.len();
        for (i, sid) in states.iter().enumerate() {
            let x = PADDING + i as f32 * (NODE_W + H_GAP);
            let y = PADDING + rank as f32 * (NODE_H + V_GAP);
            positions.insert(sid.clone(), (x, y));
        }
    }

    positions
}

fn assign_ranks(sm: &StateMachine) -> HashMap<String, usize> {
    let mut ranks: HashMap<String, usize> = HashMap::new();

    // BFS from initial state
    let start = match &sm.initial_state {
        Some(id) => id.clone(),
        None => sm.states.first().map(|s| s.id.clone()).unwrap_or_default(),
    };

    let mut queue = std::collections::VecDeque::new();
    queue.push_back((start.clone(), 0usize));
    ranks.insert(start, 0);

    while let Some((sid, rank)) = queue.pop_front() {
        for t in sm.outgoing(&sid) {
            if !ranks.contains_key(&t.to_state) {
                ranks.insert(t.to_state.clone(), rank + 1);
                queue.push_back((t.to_state.clone(), rank + 1));
            }
        }
    }

    // Assign remaining states (unreachable) to extra ranks
    let max_rank = ranks.values().max().copied().unwrap_or(0);
    for (i, state) in sm.states.iter().enumerate() {
        ranks.entry(state.id.clone()).or_insert(max_rank + 1 + i);
    }

    ranks
}
