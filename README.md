<p align="center">
  <img src="RayStudio.png" alt="RayStudio" width="120" />
</p>

<h1 align="center">StateForge</h1>
<p align="center"><strong>Automatic State Machine generation from code, logs and UI flows</strong></p>
<p align="center">
  <a href="README.de.md">Deutsch</a> ·
  <a href="https://github.com/9t29zhmwdh-coder/StateForge">GitHub</a> ·
  <a href="LICENSE">MIT License</a>
</p>

---

## What is StateForge?

StateForge is a developer tool that **automatically extracts state machines** from source code, log files, API sequences or natural language descriptions — and visualizes them as interactive diagrams.

It helps you understand complex flows, automatically document them, and regenerate clean state machine code in your target language.

## Features

| Module | Description |
|---|---|
| **Code Parser** | Extracts state machines from Swift, Kotlin, TypeScript, Go, Rust |
| **Log Analyzer** | Reconstructs state flows from log files |
| **Diagram Engine** | Renders Mermaid, GraphViz DOT, SVG, interactive React Flow |
| **Code Generator** | Generates idiomatic state machine code in 5 languages |
| **AI Integration** | Claude / Ollama — enhance machines or create from description |
| **Plugin System** | Extend with custom parsers |

## Tech Stack

- **Core** — Rust (async, petgraph, sqlx, regex)
- **Desktop** — Tauri v2
- **Frontend** — React, TypeScript, Tailwind CSS, @xyflow/react, Mermaid

## Getting Started

```bash
# Prerequisites: Rust, Node.js 18+, npm
git clone https://github.com/9t29zhmwdh-coder/StateForge
cd StateForge

npm --prefix frontend install
cargo tauri dev
```

## Usage

1. **Import** — paste source code, a log file, or describe your flow in natural language
2. **Analyze** — StateForge extracts states, transitions, events and guards automatically
3. **Visualize** — drag-and-drop diagram editor with live sync to the extracted model
4. **Generate** — export clean state machine code in Swift, Kotlin, TypeScript, Go or Rust

## Supported Input Formats

- **Swift** — enums, TCA Reducers, `@Observable` / `@Published`
- **Kotlin** — sealed classes, `when` expressions, ViewModel state
- **TypeScript** — XState `createMachine`, union types, Redux reducers
- **Go** — iota constants, switch FSMs, `SetState()` / `Transition()` patterns
- **Logs** — key=value, JSON, Nginx, Docker, Syslog

## Diagram Formats

| Format | Use case |
|---|---|
| Interactive (React Flow) | Drag-and-drop editing, live sync |
| Mermaid stateDiagram-v2 | Markdown docs, GitHub |
| GraphViz DOT | Advanced layout, CI pipelines |
| SVG | Self-contained export, presentations |

---

<p align="right">
  <sub>by <a href="https://github.com/9t29zhmwdh-coder">RayStudio</a> &nbsp;·&nbsp; MIT License</sub>
  &nbsp;
  <img src="RayStudio.png" alt="" width="70" align="right" />
</p>
