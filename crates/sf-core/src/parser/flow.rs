//! Finds the states *and the transitions between them* in source code.
//!
//! The per-language parsers this replaces found the list of states and almost
//! never a transition: Swift, Kotlin and Go produced diagrams without a single
//! arrow, XState produced nothing, and the TypeScript reducer attached every
//! arrow to whichever state a HashMap happened to return first.
//!
//! The approach is the same for every language:
//! 1. find the declaration of the state type (enum, sealed class, union, iota block);
//! 2. walk the code in segments, tracking the enclosing function, the state the
//!    code is in (`case .idle:`, `is Open ->`, `State::A =>`, `if state == X`)
//!    and the event that triggered it (`ev == Dial`, `case 'PAY':`, the function name);
//! 3. every place that sets a new state (`state = .playing`, `status: 'paid'`,
//!    the result of a `when`/`match` arm) becomes a transition from the current
//!    state(s). Code that sets a state without checking the current one allows
//!    the change from anywhere, so it gets an arrow from every other state.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeSet;

use crate::models::Language;

#[derive(Debug, Clone, PartialEq)]
pub struct Decl {
    pub type_name: String,
    pub states: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub event: Option<String>,
}

#[derive(Debug, Default)]
pub struct Flow {
    pub edges: Vec<Edge>,
    pub initial: Option<String>,
}

// ── Declarations ────────────────────────────────────────────────────────────

fn prefers_state_name(name: &str) -> bool {
    let n = name.to_lowercase();
    ["state", "status", "phase", "mode", "step"].iter().any(|w| n.contains(w))
}

/// Among several candidate types, the one named like a state wins; otherwise the first.
fn pick(mut found: Vec<Decl>) -> Option<Decl> {
    found.retain(|d| d.states.len() >= 2);
    let idx = found.iter().position(|d| prefers_state_name(&d.type_name)).unwrap_or(0);
    (!found.is_empty()).then(|| found.swap_remove(idx))
}

/// The text between the brace that opens at or after `from` and its partner.
fn braced(src: &str, from: usize) -> Option<(usize, usize)> {
    let open = from + src[from..].find('{')?;
    let mut depth = 0usize;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open + 1, open + i));
                }
            }
            _ => {}
        }
    }
    None
}

fn identifiers(body: &str, item: &Regex) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for cap in item.captures_iter(body) {
        let name = cap[1].to_string();
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

pub fn declaration(lang: Language, src: &str) -> Option<Decl> {
    match lang {
        Language::Swift => swift_decl(src),
        Language::Kotlin => kotlin_decl(src),
        Language::Go => go_decl(src),
        Language::TypeScript => ts_decl(src),
        Language::Rust => rust_decl(src),
        Language::Generic => None,
    }
}

fn swift_decl(src: &str) -> Option<Decl> {
    static ENUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"\benum\s+(\w+)").unwrap());
    static CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bcase\s+([^\n{}]+)").unwrap());
    static NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|,)\s*(\w+)").unwrap());
    let found = ENUM.captures_iter(src).filter_map(|cap| {
        let (a, b) = braced(src, cap.get(0)?.end())?;
        let mut states = Vec::new();
        for line in CASE.captures_iter(&src[a..b]) {
            // `case playing(track: Track), paused` → playing, paused
            let flat = strip_parens(&line[1]);
            states.extend(identifiers(&flat, &NAME));
        }
        Some(Decl { type_name: cap[1].to_string(), states })
    }).collect();
    pick(found)
}

fn kotlin_decl(src: &str) -> Option<Decl> {
    static ENUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"\benum\s+class\s+(\w+)").unwrap());
    static ITEM: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|,)\s*([A-Za-z_]\w*)").unwrap());
    static SEALED: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bsealed\s+(?:class|interface)\s+(\w+)").unwrap());
    let mut found: Vec<Decl> = ENUM.captures_iter(src).filter_map(|cap| {
        let (a, b) = braced(src, cap.get(0)?.end())?;
        let body = src[a..b].split(';').next().unwrap_or("");
        Some(Decl { type_name: cap[1].to_string(), states: identifiers(&strip_parens(body), &ITEM) })
    }).collect();
    for cap in SEALED.captures_iter(src) {
        let name = &cap[1];
        let sub = Regex::new(&format!(
            r"(?:object|class)\s+(\w+)\s*(?:\([^)]*\))?\s*:\s*{}\b", regex::escape(name)
        )).ok()?;
        found.push(Decl { type_name: name.to_string(), states: identifiers(src, &sub) });
    }
    pick(found)
}

fn go_decl(src: &str) -> Option<Decl> {
    static TYPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\btype\s+(\w+)\s+(?:u?int\d*|string|byte)\b").unwrap());
    static CONST_BLOCK: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)\bconst\s*\((.*?)\)").unwrap());
    let found = TYPE.captures_iter(src).map(|cap| {
        let ty = cap[1].to_string();
        let typed = Regex::new(&format!(r"^\s*(\w+)\s+{}\b", regex::escape(&ty))).unwrap();
        let bare = Regex::new(r"^\s*([A-Za-z_]\w*)\s*$").unwrap();
        let mut states = Vec::new();
        for block in CONST_BLOCK.captures_iter(src) {
            // After `A State = iota`, bare names in the same block share the type.
            let mut in_type = false;
            for line in block[1].lines() {
                if let Some(c) = typed.captures(line) {
                    in_type = true;
                    states.push(c[1].to_string());
                } else if in_type {
                    match bare.captures(line) {
                        Some(c) => states.push(c[1].to_string()),
                        None if line.trim().is_empty() || line.trim().starts_with("//") => {}
                        None => in_type = false,
                    }
                }
            }
        }
        Decl { type_name: ty, states }
    }).collect();
    pick(found)
}

fn ts_decl(src: &str) -> Option<Decl> {
    static UNION: Lazy<Regex> = Lazy::new(|| Regex::new(r"\btype\s+(\w+)\s*=\s*((?:\s*\|?\s*['\x22]\w+['\x22])+)").unwrap());
    static MEMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r#"['"](\w+)['"]"#).unwrap());
    static ENUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"\benum\s+(\w+)").unwrap());
    static ITEM: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|,)\s*(\w+)").unwrap());
    let mut found: Vec<Decl> = UNION.captures_iter(src)
        .map(|cap| Decl { type_name: cap[1].to_string(), states: identifiers(&cap[2], &MEMBER) })
        .collect();
    for cap in ENUM.captures_iter(src) {
        let Some((a, b)) = cap.get(0).and_then(|m| braced(src, m.end())) else { continue };
        // `Idle = 'idle'` keeps the member name
        let body: String = src[a..b].split(',').map(|p| p.split('=').next().unwrap_or("")).collect::<Vec<_>>().join(",");
        found.push(Decl { type_name: cap[1].to_string(), states: identifiers(&body, &ITEM) });
    }
    pick(found)
}

fn rust_decl(src: &str) -> Option<Decl> {
    static ENUM: Lazy<Regex> = Lazy::new(|| Regex::new(r"\benum\s+(\w+)").unwrap());
    static ITEM: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:^|,)\s*(?:#\[[^\]]*\]\s*)*([A-Z]\w*)").unwrap());
    let found = ENUM.captures_iter(src).filter_map(|cap| {
        let (a, b) = braced(src, cap.get(0)?.end())?;
        let body = strip_braces(&strip_parens(&src[a..b]));
        let body: String = body.lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
        Some(Decl { type_name: cap[1].to_string(), states: identifiers(&body, &ITEM) })
    }).collect();
    pick(found)
}

fn strip_nested(text: &str, open: char, close: char) -> String {
    let mut depth = 0usize;
    text.chars().filter(|&c| {
        if c == open { depth += 1; return false; }
        if c == close { depth = depth.saturating_sub(1); return false; }
        depth == 0
    }).collect()
}
fn strip_parens(text: &str) -> String { strip_nested(text, '(', ')') }
fn strip_braces(text: &str) -> String { strip_nested(text, '{', '}') }

// ── Transitions ─────────────────────────────────────────────────────────────

struct Syntax {
    /// References to a state that name it unambiguously in this language.
    reference: Regex,
    /// A state being set: `state = X`, `status: 'x'`, `setState(X)`.
    assignment: Regex,
    /// A comparison naming the current state: `state == X`, `is X`.
    state_check: Regex,
    initializer: Regex,
}

fn syntax(lang: Language, decl: &Decl) -> Syntax {
    let ty = regex::escape(&decl.type_name);
    let names = decl.states.iter().map(|s| regex::escape(s)).collect::<Vec<_>>().join("|");
    // How one state is written in code.
    let reference = match lang {
        Language::Swift => format!(r"(?:\b{ty})?\.({names})\b"),
        // Bare names count only when not qualified by something else: `Event.Open` is no state.
        Language::Kotlin => format!(r"(?:\b{ty}\.|(?:^|[^.\w]))({names})\b"),
        Language::Go => format!(r"(?:^|[^.\w])({names})\b"),
        Language::Rust => format!(r"\b(?:{ty}|Self)::({names})\b"),
        Language::TypeScript => format!(r#"(?:['"]({names})['"]|\b{ty}\.({names})\b)"#),
        Language::Generic => format!(r"\b({names})\b"),
    };
    // `state`, `c.state`, `self.status`, `_uiState.value`, `currentPhase`
    let state_var = r"(?:\b(?:self\.|this\.|\w+\.)?_?\w*(?:state|status|phase|mode)\w*(?:\.value)?\b|\bcurrent\w*\b)";
    let assignment = format!(r"(?i:{state_var})\s*(?::\s*\w+\s*)?=\s*(?:{reference})|(?i:\b(?:status|state|phase|mode)\s*:\s*)(?:{reference})|\bset(?:State|Status|Phase|Mode)\s*\(\s*(?:{reference})|\btransition(?:To)?\s*\(\s*(?:{reference})");
    let state_check = format!(r"(?i:{state_var})\s*(?:==|===)\s*(?:{reference})|\bis\s+(?:{reference})|(?:{reference})\s*(?:==|===)\s*(?i:{state_var})");
    // Holders created with a start value: where the machine begins, not a transition.
    let initializer = format!(r"\b(?:MutableStateFlow|mutableStateOf|useState|useReducer|BehaviorSubject|signal|ref)\s*(?:<[^>]*>)?\s*\(\s*(?:\w+\s*,\s*)?(?:{reference})");
    Syntax {
        initializer: Regex::new(&initializer).unwrap(),
        reference: Regex::new(&reference).unwrap(),
        assignment: Regex::new(&assignment).unwrap(),
        state_check: Regex::new(&state_check).unwrap(),
    }
}

/// All state names in the captures of `re` over `text`.
fn states_in(re: &Regex, text: &str) -> Vec<String> {
    re.captures_iter(text)
        .flat_map(|c| c.iter().skip(1).flatten().map(|m| m.as_str().to_string()).collect::<Vec<_>>())
        .collect()
}

static FUNCTION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:func|fun|function|fn)\s+(?:\([^)]*\)\s*)?(\w+)|^\s*(?:(?:public|private|protected|static|async|override)\s+)*(\w+)\s*\([^)]*\)\s*(?::\s*[^{=]+)?\{").unwrap()
});
static EVENT_CHECK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i:\b(?:ev|event|evt|action|msg|message|cmd|command|input|trigger)\w*(?:\.type)?)\s*(?:==|===)\s*(?:\w+(?:\.|::))*['\x22]?(\w+)").unwrap()
});
static CASE_HEAD: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\s*case\s+([^:]+):"#).unwrap());
static CASE_LABEL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?:\w+(?:\.|::))*['"]?\.?(\w+)['"]?"#).unwrap());
const KEYWORDS: &[&str] = &["if", "for", "while", "switch", "catch", "when", "match", "else", "return"];

struct Guard {
    depth: usize,
    states: Vec<String>,
    event: Option<String>,
}

pub fn transitions(lang: Language, src: &str, decl: &Decl) -> Flow {
    let syn = syntax(lang, decl);
    let arrow = match lang {
        Language::Kotlin => Some("->"),
        Language::Rust => Some("=>"),
        _ => None,
    };
    let mut flow = Flow::default();
    let mut edges = BTreeSet::new();
    let mut depth = 0usize;
    let mut function: Option<(String, usize)> = None;
    let mut branch: Option<Guard> = None; // case X: / is X -> / X =>
    let mut guards: Vec<Guard> = Vec::new(); // if state == X { ... }
    let mut last_event_guard: Option<String> = None;
    // The state set last in this function: `setStatus('uploading')` then
    // `setStatus('done')` is uploading → done, not "done from anywhere".
    let mut last_set: Option<Vec<String>> = None;
    let mut try_marks: Vec<Option<Vec<String>>> = Vec::new();

    for segment in segments(src) {
        let text = segment.trim();
        let before = depth;
        let opens = text.matches('{').count();
        let closes = text.matches('}').count();

        if let Some(cap) = FUNCTION.captures(text) {
            let name = cap.get(1).or(cap.get(2)).map(|m| m.as_str()).unwrap_or("");
            if !name.is_empty() && !KEYWORDS.contains(&name) {
                function = Some((name.to_string(), before));
                branch = None;
                last_set = None;
                try_marks.clear();
            }
        }

        // Where are we: the branch head and the if-guards give the current state(s) and event.
        let (head, body) = match arrow.and_then(|a| text.find(a).map(|i| (i, a.len()))) {
            Some((i, len)) => (&text[..i], &text[i + len..]),
            None => ("", text),
        };
        if let Some(cap) = CASE_HEAD.captures(text) {
            let label = &cap[1];
            let states: Vec<String> = states_in(&syn.reference, label);
            let states = if states.is_empty() && lang != Language::TypeScript {
                // Go writes bare constants in `case X:`; Swift `case .x:`
                CASE_LABEL.captures_iter(label).map(|c| c[1].to_string()).filter(|s| decl.states.contains(s)).collect()
            } else { states };
            // `case 'PAY':` names an event; in `case (.downloading, .complete):` the
            // part that is not a state is the event.
            let event = CASE_LABEL.captures_iter(label).map(|c| c[1].to_string())
                .find(|e| !decl.states.contains(e) && !["default", "let", "var", "_"].contains(&e.as_str()));
            branch = Some(Guard { depth: before, states, event });
            last_set = None;
        } else if !head.is_empty() && head.trim() != "else" && !head.trim_start().starts_with("//") {
            let states = states_in(&syn.reference, head);
            // `(Light::Red, Tick::Timer) =>`: the other qualified name is the event.
            let event = EVENT_CHECK.captures(head).map(|c| c[1].to_string())
                .or_else(|| qualified_other(head, decl));
            match (&mut branch, states.is_empty()) {
                // `Event.Open -> ...` inside a state arm refines the event only.
                (Some(b), true) if event.is_some() && b.depth < before + 1 && !b.states.is_empty() => b.event = event,
                _ => {
                    branch = Some(Guard { depth: before, states, event });
                    last_set = None;
                }
            }
        }

        if text.starts_with("try") {
            try_marks.push(last_set.clone());
        } else if text.starts_with("catch") || text.starts_with("} catch") || text.starts_with("except") {
            // A catch block runs after the state set before the try, not after the try body.
            if let Some(mark) = try_marks.pop() {
                last_set = mark;
            }
        }
        if let Some(first) = states_in(&syn.initializer, body).first() {
            if flow.initial.is_none() {
                flow.initial = Some(first.clone());
            }
            continue_after(&mut depth, opens, closes);
            continue;
        }
        if text.starts_with("else") || text.starts_with("} else") {
            last_set = None;
        }

        let checked = states_in(&syn.state_check, body);
        let event_here = EVENT_CHECK.captures(body).map(|c| c[1].to_string());
        let is_if = text.starts_with("if ") || text.starts_with("if(") || text.contains(" if ") || text.contains(" if(") || text.starts_with("} else if") || text.starts_with("else if");
        if is_if && (!checked.is_empty() || event_here.is_some()) {
            guards.push(Guard { depth: before, states: checked.clone(), event: event_here.clone() });
        }
        let else_event = if (text.starts_with("else") || text.starts_with("} else")) && !is_if {
            last_event_guard.as_ref().map(|e| format!("not {e}"))
        } else { None };
        if let Some(e) = else_event.as_ref().filter(|_| opens > 0) {
            // `else { c.state = Failed }`: the block runs when the event was not the one checked.
            guards.push(Guard { depth: before, states: Vec::new(), event: Some(e.clone()) });
        }

        // What is set here.
        let mut targets = states_in(&syn.assignment, body);
        if arrow.is_some() && !head.is_empty() {
            // The result of a `when`/`match` arm is the next state, unless it is compared.
            let result = syn.state_check.replace_all(body, "");
            let result = EVENT_CHECK.replace_all(&result, "");
            targets.extend(states_in(&syn.reference, &result));
        } else if lang == Language::Kotlin || lang == Language::Rust {
            if let Some(rest) = body.trim_start().strip_prefix("else") {
                targets.extend(states_in(&syn.reference, rest));
            }
        }
        if (lang == Language::Rust || lang == Language::Kotlin || lang == Language::TypeScript) && body.trim_start().starts_with("return ") {
            targets.extend(states_in(&syn.reference, body));
        }
        targets.dedup();

        if function.is_none() && !targets.is_empty() && flow.initial.is_none() {
            // `var state: PlayerState = .idle` outside any function: where it starts.
            flow.initial = targets.first().cloned();
            targets.clear();
        }

        let froms: Vec<String> = guards.iter().rev().find(|g| !g.states.is_empty()).map(|g| g.states.clone())
            .or_else(|| branch.as_ref().filter(|b| !b.states.is_empty()).map(|b| b.states.clone()))
            .or_else(|| last_set.clone())
            .unwrap_or_default();
        let event = else_event
            .or_else(|| guards.iter().rev().find_map(|g| g.event.clone()))
            .or_else(|| branch.as_ref().and_then(|b| b.event.clone()))
            .or_else(|| function.as_ref().map(|(f, _)| f.clone()));

        for to in &targets {
            let sources: Vec<&String> = if froms.is_empty() {
                // Set without checking the current state: allowed from any other state.
                decl.states.iter().filter(|s| *s != to).collect()
            } else {
                froms.iter().filter(|s| *s != to).collect()
            };
            for from in sources {
                edges.insert(Edge { from: from.clone(), to: to.clone(), event: event.clone() });
            }
        }

        if !targets.is_empty() {
            last_set = Some(targets.clone());
        }

        // Leave blocks.
        depth = (depth + opens).saturating_sub(closes);
        while let Some(g) = guards.last() {
            if depth <= g.depth {
                last_event_guard = g.event.clone();
                guards.pop();
            } else {
                break;
            }
        }
        if !(text.starts_with("else") || text.starts_with("} else")) && guards.is_empty() && closes == 0 && opens == 0 && !is_if {
            last_event_guard = None;
        }
        if branch.as_ref().is_some_and(|b| depth < b.depth) {
            branch = None;
        }
        if function.as_ref().is_some_and(|(_, d)| depth <= *d && closes > 0) {
            function = None;
            branch = None;
        }
    }

    flow.edges = edges.into_iter().collect();
    flow
}

fn continue_after(depth: &mut usize, opens: usize, closes: usize) {
    *depth = (*depth + opens).saturating_sub(closes);
}

/// `Event.Close` / `Event::Close` in an arm head names the event.
fn qualified_other(head: &str, decl: &Decl) -> Option<String> {
    static QUALIFIED: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\w+)(?:\.|::)(\w+)\b").unwrap());
    QUALIFIED.captures_iter(head)
        .find(|c| c[1] != decl.type_name && c[1] != *"Self")
        .map(|c| c[2].to_string())
}

/// Code split after `{`, `}` and `;`, so one-line blocks and arms are visited one by one.
fn segments(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        let code = strip_line_comment(line);
        let mut current = String::new();
        let mut quote: Option<char> = None;
        for c in code.chars() {
            match quote {
                Some(q) if c == q => quote = None,
                Some(_) => {}
                None if c == '\'' || c == '"' || c == '`' => quote = Some(c),
                None => {}
            }
            if quote.is_none() && c == '}' && !current.trim().is_empty() {
                out.push(std::mem::take(&mut current));
            }
            current.push(c);
            if quote.is_none() && (c == '{' || c == ';' || c == '}') {
                out.push(std::mem::take(&mut current));
            }
        }
        if !current.trim().is_empty() {
            out.push(current);
        }
    }
    out.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

fn strip_line_comment(line: &str) -> &str {
    match line.find("//") {
        Some(i) if !line[..i].ends_with(':') => &line[..i], // keep `https://`
        _ => line,
    }
}

// ── XState ──────────────────────────────────────────────────────────────────

/// `createMachine({ initial, states: { a: { on: { EV: 'b' } } } })`: only the
/// top level of `states`; nested machines are not followed.
pub fn xstate(src: &str) -> Option<(Decl, Flow, BTreeSet<String>)> {
    static STATES_KEY: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bstates\s*:\s*\{").unwrap());
    static INITIAL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\binitial\s*:\s*['"](\w+)['"]"#).unwrap());
    // A key whose value is an object: `idle: {`, `'idle': {`, also mid-line.
    static KEY: Lazy<Regex> = Lazy::new(|| Regex::new(r#"['"]?(\w+)['"]?\s*:\s*\{"#).unwrap());
    static ON: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bon\s*:\s*\{").unwrap());
    static PAIR: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"['"]?([\w.]+)['"]?\s*:\s*(?:['"]\.?(\w+)['"]|\{[^{}]*?target\s*:\s*['"]\.?(\w+)['"][^{}]*\}|\[\s*\{[^\]]*?target\s*:\s*['"]\.?(\w+)['"])"#).unwrap()
    });
    static FINAL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"\btype\s*:\s*['"]final['"]"#).unwrap());

    let start = src.find("createMachine").or_else(|| src.find("setup("))?;
    let states_at = start + STATES_KEY.find(&src[start..])?.start();
    let (a, b) = braced(src, states_at)?;
    let body = &src[a..b];

    static NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?:(?:const|let|var)\s+(\w+)\s*=\s*(?:\w+\.)?createMachine|\bid\s*:\s*['"](\w+)['"])"#).unwrap());
    let type_name = NAME.captures(src).and_then(|c| c.get(1).or(c.get(2))).map(|m| m.as_str().to_string()).unwrap_or_else(|| "State".into());
    let mut decl = Decl { type_name, states: Vec::new() };
    let mut flow = Flow { initial: INITIAL.captures(&src[start..]).map(|c| c[1].to_string()), ..Default::default() };
    let mut finals = BTreeSet::new();
    let mut raw_edges = Vec::new();

    let mut pos = 0;
    while pos < body.len() {
        let Some(m) = KEY.find(&body[pos..]) else { break };
        let name = KEY.captures(m.as_str()).map(|c| c[1].to_string()).unwrap_or_default();
        let Some((sa, sb)) = braced(body, pos + m.start()) else { break };
        let block = &body[sa..sb];
        decl.states.push(name.clone());
        if FINAL.is_match(&strip_braces(block)) {
            finals.insert(name.clone());
        }
        if let Some(on) = ON.find(block) {
            if let Some((oa, ob)) = braced(block, on.start()) {
                for c in PAIR.captures_iter(&block[oa..ob]) {
                    let to = c.get(2).or(c.get(3)).or(c.get(4)).map(|m| m.as_str().to_string()).unwrap_or_default();
                    raw_edges.push(Edge { from: name.clone(), to, event: Some(c[1].to_string()) });
                }
            }
        }
        pos = sb + 1;
    }
    flow.edges = raw_edges.into_iter().filter(|e| decl.states.contains(&e.to)).collect();
    (!decl.states.is_empty()).then_some((decl, flow, finals))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edges(lang: Language, src: &str) -> (Decl, Vec<(String, String, String)>, Option<String>) {
        let decl = declaration(lang, src).expect("declaration");
        let flow = transitions(lang, src, &decl);
        let list = flow.edges.iter().map(|e| (e.from.clone(), e.to.clone(), e.event.clone().unwrap_or_default())).collect();
        (decl, list, flow.initial)
    }

    fn has(list: &[(String, String, String)], from: &str, to: &str, event: &str) -> bool {
        list.iter().any(|(f, t, e)| f == from && t == to && e == event)
    }

    #[test]
    fn swift_guards_and_unguarded_setters() {
        let src = r#"
enum PlayerState {
    case idle
    case playing(track: String), paused
    case stopped
}
final class Player {
    var state: PlayerState = .idle
    func play() { if state == .idle || state == .paused { state = .playing } }
    func pause() { if state == .playing { state = .paused } }
    func stop() { state = .stopped }
}"#;
        let (decl, e, initial) = edges(Language::Swift, src);
        assert_eq!(decl.states, ["idle", "playing", "paused", "stopped"]);
        assert_eq!(initial.as_deref(), Some("idle"));
        assert!(has(&e, "idle", "playing", "play"));
        assert!(has(&e, "paused", "playing", "play"));
        assert!(has(&e, "playing", "paused", "pause"));
        assert!(has(&e, "playing", "stopped", "stop"));
        assert!(!has(&e, "stopped", "playing", "play"), "{e:?}");
    }

    #[test]
    fn kotlin_when_arms_with_nested_events() {
        let src = r#"
sealed class DoorState {
    object Open : DoorState()
    object Closed : DoorState()
    object Locked : DoorState()
}
fun next(state: DoorState, event: Event): DoorState = when (state) {
    is DoorState.Open -> if (event == Event.Close) DoorState.Closed else state
    is DoorState.Closed -> when (event) { Event.Open -> DoorState.Open; Event.Lock -> DoorState.Locked; else -> state }
    is DoorState.Locked -> if (event == Event.Unlock) DoorState.Closed else state
}"#;
        let (decl, e, _) = edges(Language::Kotlin, src);
        assert_eq!(decl.states, ["Open", "Closed", "Locked"]);
        assert!(has(&e, "Open", "Closed", "Close"), "{e:?}");
        assert!(has(&e, "Closed", "Open", "Open"), "{e:?}");
        assert!(has(&e, "Closed", "Locked", "Lock"), "{e:?}");
        assert!(has(&e, "Locked", "Closed", "Unlock"), "{e:?}");
        assert_eq!(e.len(), 4, "{e:?}");
    }

    #[test]
    fn go_iota_and_switch_with_else() {
        let src = "package conn\n\ntype State int\n\nconst (\n\tDisconnected State = iota\n\tConnecting\n\tConnected\n\tFailed\n)\n\nfunc (c *Conn) handle(ev Event) {\n\tswitch c.state {\n\tcase Disconnected:\n\t\tif ev == Dial { c.state = Connecting }\n\tcase Connecting:\n\t\tif ev == Ok { c.state = Connected } else { c.state = Failed }\n\tcase Connected:\n\t\tif ev == Drop { c.state = Disconnected }\n\t}\n}\n";
        let (decl, e, _) = edges(Language::Go, src);
        assert_eq!(decl.states, ["Disconnected", "Connecting", "Connected", "Failed"]);
        assert!(has(&e, "Disconnected", "Connecting", "Dial"), "{e:?}");
        assert!(has(&e, "Connecting", "Connected", "Ok"), "{e:?}");
        assert!(has(&e, "Connecting", "Failed", "not Ok"), "{e:?}");
        assert!(has(&e, "Connected", "Disconnected", "Drop"), "{e:?}");
        assert_eq!(e.len(), 4, "{e:?}");
    }

    #[test]
    fn typescript_reducer_on_action_type() {
        let src = r#"
type OrderState = 'pending' | 'paid' | 'shipped' | 'cancelled';
function reducer(state: Order, action: Action): Order {
  switch (action.type) {
    case 'PAY': if (state.status === 'pending') return { ...state, status: 'paid' }; return state;
    case 'SHIP': if (state.status === 'paid') return { ...state, status: 'shipped' }; return state;
    case 'CANCEL': return { ...state, status: 'cancelled' };
  }
}"#;
        let (_, e, _) = edges(Language::TypeScript, src);
        assert!(has(&e, "pending", "paid", "PAY"), "{e:?}");
        assert!(has(&e, "paid", "shipped", "SHIP"), "{e:?}");
        assert!(has(&e, "shipped", "cancelled", "CANCEL"), "cancel has no guard, so from anywhere: {e:?}");
        assert!(!has(&e, "pending", "shipped", "SHIP"), "{e:?}");
    }

    #[test]
    fn rust_match_on_state_and_event() {
        let src = r#"
enum Light { Red, Green, Yellow(u8) }
enum Tick { Timer }
impl Light {
    fn next(self, ev: Tick) -> Light {
        match (self, ev) {
            (Light::Red, Tick::Timer) => Light::Green,
            (Light::Green, Tick::Timer) => Light::Yellow(3),
            (Light::Yellow(_), Tick::Timer) => Light::Red,
        }
    }
}"#;
        let (decl, e, _) = edges(Language::Rust, src);
        assert_eq!(decl.type_name, "Light");
        assert!(has(&e, "Red", "Green", "Timer"), "{e:?}");
        assert!(has(&e, "Green", "Yellow", "Timer"), "{e:?}");
        assert!(has(&e, "Yellow", "Red", "Timer"), "{e:?}");
    }

    #[test]
    fn kotlin_stateflow_in_sequence_with_catch() {
        let src = r#"
sealed interface LoginUiState {
    data object Idle : LoginUiState
    data object Loading : LoginUiState
    data class Success(val user: User) : LoginUiState
    data class Error(val message: String) : LoginUiState
}
class LoginViewModel : ViewModel() {
    private val _uiState = MutableStateFlow<LoginUiState>(LoginUiState.Idle)
    fun login() {
        _uiState.value = LoginUiState.Loading
        viewModelScope.launch {
            try {
                _uiState.value = LoginUiState.Success(user)
            } catch (e: Exception) {
                _uiState.value = LoginUiState.Error(e.message)
            }
        }
    }
}"#;
        let (decl, e, initial) = edges(Language::Kotlin, src);
        assert_eq!(decl.states, ["Idle", "Loading", "Success", "Error"]);
        assert_eq!(initial.as_deref(), Some("Idle"));
        assert!(has(&e, "Loading", "Success", "login"), "{e:?}");
        assert!(has(&e, "Loading", "Error", "login"), "{e:?}");
        assert!(!has(&e, "Success", "Error", "login"), "catch follows the state before try: {e:?}");
    }

    #[test]
    fn react_use_state_sequence() {
        let src = r#"
type UploadStatus = 'idle' | 'uploading' | 'done' | 'failed';
export function Upload() {
  const [status, setStatus] = useState<UploadStatus>('idle');
  async function start(file: File) {
    setStatus('uploading');
    try {
      await api.upload(file);
      setStatus('done');
    } catch {
      setStatus('failed');
    }
  }
  function retry() {
    if (status === 'failed') setStatus('idle');
  }
}"#;
        let (_, e, initial) = edges(Language::TypeScript, src);
        assert_eq!(initial.as_deref(), Some("idle"));
        assert!(has(&e, "uploading", "done", "start"), "{e:?}");
        assert!(has(&e, "uploading", "failed", "start"), "{e:?}");
        assert!(has(&e, "failed", "idle", "retry"), "{e:?}");
        assert!(!has(&e, "done", "failed", "start"), "{e:?}");
    }

    #[test]
    fn swift_switch_on_state_and_event_tuple() {
        let src = r#"
enum DownloadState: Equatable {
    case notStarted
    case downloading(progress: Double)
    case finished
    case failed(Error)
}
final class Downloader {
    private(set) var state: DownloadState = .notStarted
    func handle(_ event: DownloadEvent) {
        switch (state, event) {
        case (.notStarted, .start):
            state = .downloading(progress: 0)
        case (.downloading, .complete):
            state = .finished
        case (.failed, .retry):
            state = .notStarted
        default:
            break
        }
    }
}"#;
        let (_, e, _) = edges(Language::Swift, src);
        assert!(has(&e, "notStarted", "downloading", "start"), "{e:?}");
        assert!(has(&e, "downloading", "finished", "complete"), "{e:?}");
        assert!(has(&e, "failed", "notStarted", "retry"), "{e:?}");
        assert_eq!(e.len(), 3, "{e:?}");
    }

    #[test]
    fn xstate_machine() {
        let src = r#"
export const fetchMachine = createMachine({
  id: 'fetch',
  initial: 'idle',
  states: {
    idle: { on: { FETCH: 'loading' } },
    loading: { on: { RESOLVE: { target: 'success' }, REJECT: 'failure' } },
    success: { type: 'final' },
    failure: { on: { RETRY: 'loading' } }
  }
});"#;
        let (decl, flow, finals) = xstate(src).unwrap();
        assert_eq!(decl.type_name, "fetchMachine");
        assert_eq!(decl.states, ["idle", "loading", "success", "failure"]);
        assert_eq!(flow.initial.as_deref(), Some("idle"));
        assert!(finals.contains("success"));
        let e: Vec<_> = flow.edges.iter().map(|e| (e.from.as_str(), e.to.as_str(), e.event.as_deref().unwrap())).collect();
        assert_eq!(e, [("idle", "loading", "FETCH"), ("loading", "success", "RESOLVE"), ("loading", "failure", "REJECT"), ("failure", "loading", "RETRY")]);
    }
}
