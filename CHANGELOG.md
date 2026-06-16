# Changelog — StateForge

All notable changes to this project will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) — [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [0.1.0] — 2026-06-12

### Added

- FSM extraction from source code files (Rust, TypeScript, Python): state enums, match arms, transition patterns via static analysis
- FSM extraction from log files: pattern matching and sequence mining
- FSM extraction from natural language descriptions via Ollama (localhost:11434)
- Normalized `FsmModel` data structure: states, transitions, guards, actions, initial/final state annotations
- Interactive React Flow diagram canvas with drag, zoom, and pan
- Mermaid `stateDiagram-v2` export
- GraphViz DOT export
- Standalone SVG export
- `sf-core` Rust crate: `extractor/` (code/, log/, nl/), `fsm/`, `exporter/` (mermaid/, dot/, svg/)
- `sf-cli` binary for headless extraction and batch export
- Tauri v2 desktop shell for macOS, Windows, and Linux
- React/TypeScript frontend with Zustand state management

[0.1.0]: https://github.com/9t29zhmwdh-coder/StateForge/releases/tag/v0.1.0
