# Architecture — StateForge

## Overview

StateForge is a Rust workspace with a Tauri v2 desktop shell and a React/TypeScript frontend. The core library (`sf-core`) handles FSM extraction from diverse input sources and export to multiple diagram formats. `sf-cli` enables headless/batch operation. AI-assisted extraction from natural language and ambiguous inputs runs locally via Ollama.

```
StateForge/
├── sf-core/              # Core library crate
│   └── src/
│       ├── extractor/    # Input-specific extraction pipelines
│       │   ├── code/     # Static analysis of Rust/TypeScript/Python source
│       │   ├── log/      # Pattern-based FSM inference from log files
│       │   └── nl/       # Natural language → FSM via Ollama
│       ├── fsm/          # State machine model (states, transitions, guards, actions)
│       └── exporter/     # Output renderers
│           ├── mermaid/  # Mermaid stateDiagram-v2
│           ├── dot/      # GraphViz DOT
│           └── svg/      # Direct SVG generation
├── sf-cli/               # CLI binary (headless extraction + export)
├── src-tauri/            # Tauri v2 backend
│   └── src/
│       ├── main.rs
│       ├── error.rs
│       ├── state.rs
│       └── commands/     # IPC handlers (extract, visualize, export, save)
└── frontend/             # React + TypeScript + Vite + React Flow
    └── src/
        ├── stores/       # Zustand state (FSM model, view mode, export settings)
        └── components/   # UI components (FlowCanvas, StateNode, TransitionEdge, ExportPanel)
```

## Component Diagram

```
┌────────────────────────────────────────────────────────────────┐
│                     StateForge Desktop                         │
│                                                                │
│  ┌──────────────────┐   Tauri IPC   ┌───────────────────────┐  │
│  │    Frontend      │◄─────────────►│      src-tauri        │  │
│  │  React Flow / TS │               │    commands/*.rs      │  │
│  └──────────────────┘               └──────────┬────────────┘  │
│                                                │               │
│                                     ┌──────────▼────────────┐  │
│                                     │       sf-core         │  │
│                                     │                       │  │
│                                     │  extractor/           │  │
│                                     │    code/ ─────────────┼──┼── source files
│                                     │    log/  ─────────────┼──┼── log files
│                                     │    nl/   ─────────────┼──┼──► Ollama
│                                     │                       │  │    localhost:11434
│                                     │  fsm/ (model)         │  │
│                                     │                       │  │
│                                     │  exporter/            │  │
│                                     │    mermaid/           │  │
│                                     │    dot/               │  │
│                                     │    svg/               │  │
│                                     └───────────────────────┘  │
│                                                                │
│  sf-cli ──────────────────────────────────────► sf-core        │
└────────────────────────────────────────────────────────────────┘
```

## Data Flow

1. **Input** — user provides source code files, log files, API sequence descriptions, or natural language text describing a workflow.
2. **Extract** — the appropriate extractor pipeline parses the input: `code/` uses static analysis heuristics (state enums, match arms, transition patterns); `log/` applies pattern matching and sequence mining; `nl/` sends a structured prompt to Ollama and receives a state/transition list.
3. **Model** — all extractors produce a normalized `FsmModel` (states, transitions, guards, actions, initial/final states) defined in `fsm/`.
4. **Visualize** — the Tauri backend sends the `FsmModel` to the frontend via IPC; React Flow renders it as an interactive diagram (drag nodes, inspect transitions, zoom/pan).
5. **Export** — the user triggers export; `mermaid/` renders `stateDiagram-v2` syntax, `dot/` renders GraphViz DOT, `svg/` generates a standalone SVG file.

## External Dependencies

| Dependency | Purpose | Network |
|------------|---------|---------|
| Ollama (localhost:11434) | NL-to-FSM extraction + AI assistance | localhost only |
| React Flow | Interactive FSM diagram canvas | none |
| serde / serde_json | Serialization of FSM model | none |
| Tauri v2 | Desktop shell + IPC | none |
| React + Vite | Frontend | none (build-time only) |
