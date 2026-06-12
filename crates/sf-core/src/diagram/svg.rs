use crate::models::{StateMachine, StateKind, TransitionKind};
use crate::models::diagram::DiagramConfig;
use super::layout::compute_layout;
use anyhow::Result;

const NODE_W: f32 = 120.0;
const NODE_H: f32 = 44.0;
const H_GAP: f32 = 60.0;
const V_GAP: f32 = 80.0;
const FONT: &str = "JetBrains Mono, Menlo, monospace";

pub fn render(sm: &StateMachine, _config: &DiagramConfig) -> Result<String> {
    let positions = compute_layout(sm);

    let (max_x, max_y) = positions.values()
        .fold((0.0f32, 0.0f32), |(mx, my), &(x, y)| (mx.max(x), my.max(y)));

    let width = max_x + NODE_W + H_GAP;
    let height = max_y + NODE_H + V_GAP;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}">
<defs>
  <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto">
    <path d="M 0 0 L 10 5 L 0 10 z" fill="#555"/>
  </marker>
  <marker id="arrow-error" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto">
    <path d="M 0 0 L 10 5 L 0 10 z" fill="#e74c3c"/>
  </marker>
</defs>
<rect width="{w}" height="{h}" fill="#0d1117"/>
"#,
        w = width as u32, h = height as u32
    );

    // Draw edges
    for t in &sm.transitions {
        let Some(from) = sm.state_by_id(&t.from_state) else { continue };
        let Some(to)   = sm.state_by_id(&t.to_state)   else { continue };
        let Some(&(fx, fy)) = positions.get(&from.id) else { continue };
        let Some(&(tx, ty)) = positions.get(&to.id)   else { continue };

        let x1 = fx + NODE_W / 2.0;
        let y1 = fy + NODE_H;
        let x2 = tx + NODE_W / 2.0;
        let y2 = ty;
        let cx = (x1 + x2) / 2.0;
        let cy = (y1 + y2) / 2.0 - 20.0;

        let (stroke, marker) = if t.kind == TransitionKind::Error {
            ("#e74c3c", "url(#arrow-error)")
        } else {
            ("#58a6ff", "url(#arrow)")
        };

        svg.push_str(&format!(
            r#"<path d="M{x1},{y1} Q{cx},{cy} {x2},{y2}" fill="none" stroke="{stroke}" stroke-width="1.5" marker-end="{marker}"/>"#,
        ));

        if let Some(ref event) = t.event {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" fill="#8b949e" font-family="{FONT}" font-size="10" text-anchor="middle">{}</text>"#,
                cx, cy - 4.0, escape_xml(event)
            ));
        }
    }

    // Draw nodes
    for state in &sm.states {
        let Some(&(x, y)) = positions.get(&state.id) else { continue };

        let (fill, stroke, text_fill) = match state.kind {
            StateKind::Initial  => ("#1f6feb", "#58a6ff", "#ffffff"),
            StateKind::Final    => ("#196227", "#3fb950", "#ffffff"),
            StateKind::Error    => ("#67060c", "#f85149", "#ffffff"),
            StateKind::Parallel => ("#3d2b00", "#e3b341", "#e3b341"),
            _                   => ("#161b22", "#30363d", "#e6edf3"),
        };

        let rx = if state.kind == StateKind::Initial || state.kind == StateKind::Final { "22" } else { "8" };

        svg.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{NODE_W}" height="{NODE_H}" fill="{fill}" stroke="{stroke}" stroke-width="1.5" rx="{rx}"/>"#,
        ));
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" fill="{text_fill}" font-family="{FONT}" font-size="12" font-weight="500" text-anchor="middle" dominant-baseline="middle">{}</text>"#,
            x + NODE_W / 2.0, y + NODE_H / 2.0, escape_xml(&state.name)
        ));
    }

    svg.push_str("</svg>");
    Ok(svg)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
