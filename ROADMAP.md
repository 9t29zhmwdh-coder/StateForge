# Roadmap, StateForge

## v0.1.0, Initial Release (2026-06-12) ✅

- FSM extraction from source code (Rust, TypeScript, Python: state enums, match arms, transitions)
- FSM extraction from log files (pattern matching, sequence mining)
- FSM extraction from natural language descriptions via Claude
- Normalized `FsmModel` (states, transitions, guards, actions, initial/final states)
- Interactive React Flow diagram canvas (drag, zoom, pan)
- Mermaid `stateDiagram-v2` export
- GraphViz DOT export
- SVG export (standalone file)
- `sf-core` Rust crate: `extractor/`, `fsm/`, `exporter/`
- `sf-cli` binary for headless extraction and export
- Tauri v2 desktop shell (macOS, Windows, Linux)

## v0.2.0, Edit & Annotate

- Wire up the Ollama option in Settings to an actual `OllamaAnalyzer`; the
  UI already lets you pick "ollama" and set a host/model, but the backend's
  `get_analyzer()` always returns `ClaudeAnalyzer` regardless of the setting
- Wire up the "Auto AI Enhance" toggle in Settings; `auto_ai_enhance` is
  stored in `AppSettings` but never read anywhere, so toggling it has no effect
- In-canvas FSM editing (add/remove states and transitions via GUI)
- Transition guard and action annotation editor
- Import previously exported FSM (Mermaid or DOT to `FsmModel`)
- FSM diff view (compare two versions of the same machine)
- Project save/load (`.stateforge` project format, JSON-based)

## v0.3.0, Extended Extraction & Analysis

- API sequence extraction (HTTP log / OpenAPI spec to FSM)
- FSM validation (reachability, deadlock detection, nondeterminism warnings)
- Simulation mode: step through FSM states interactively with test inputs
- Multiple FSMs per project (tabbed view, cross-reference)
- Configurable extraction heuristics (TOML-based rules)

## v1.0.0, Stable Release

- Stable public API for `sf-core` (semver)
- Full test coverage (unit + integration + snapshot tests for export formats)
- Packaged installers (`.dmg`, `.msi`, `.AppImage`)
- Comprehensive documentation site

## Out of Scope

- UML tool integration (Enterprise Architect, Lucidchart import/export)
- Cloud storage or collaborative real-time editing
- Runtime FSM execution engine (this is a visualization/documentation tool)
- Mobile platforms (iOS, Android)

## Dual-Licensing Readiness

Assessed 2026-07-11: Community-only, not a Dual-Licensing candidate. StateForge's own roadmap already rules out cloud storage, collaborative real-time editing and UML enterprise tool integration by design, which are exactly the features enterprise diagram tools (Lucidchart, Miro) monetize around. It is a single-developer documentation/visualization tool with no team dimension. Revisit only if the project's scope intentionally changes toward collaborative editing.
