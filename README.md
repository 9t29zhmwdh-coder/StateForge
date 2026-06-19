<div align="center">
  <img src="RayStudio.png" alt="RayStudio Logo" width="120"/>

  <h1>StateForge</h1>
</div>

[🇩🇪 Deutsche Version](README.de.md)

**Automatic state machine generation from code, logs and UI flows, built with Rust and Tauri.**

StateForge automatically extracts state machines from source code, log files, API sequences, or natural language descriptions and visualizes them as interactive diagrams. It helps you understand complex flows, document them automatically, and regenerate clean state machine code in your target language.

[![CI](https://github.com/9t29zhmwdh-coder/StateForge/actions/workflows/ci.yml/badge.svg)](https://github.com/9t29zhmwdh-coder/StateForge/actions) ![Platform](https://img.shields.io/badge/Platform-macOS_%7C_Windows_%7C_Ubuntu-lightgrey) ![Rust](https://img.shields.io/badge/Rust-CE422B?logo=rust&logoColor=white) ![AI | Claude Code](https://img.shields.io/badge/AI-Claude_Code-black?logo=anthropic&logoColor=white) ![AI | Copilot](https://img.shields.io/badge/AI-Copilot-black?logo=github&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)

---

## Features

| Feature | Description |
|---|---|
| **Code Parser** | Extracts state machines from Swift, Kotlin, TypeScript, Go, Rust |
| **Log Analyzer** | Reconstructs state flows from log files (JSON, plaintext, nginx, syslog) |
| **Diagram Engine** | Renders Mermaid, GraphViz DOT, SVG, interactive React Flow |
| **Code Generator** | Generates idiomatic state machine code in 5 languages |
| **AI Integration** | Local AI (Ollama): enhance machines or create from natural language |
| **Plugin System** | Extend with custom parsers via Rust trait |

---

## Requirements

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 20+
- [Tauri CLI v2](https://tauri.app/): `cargo install tauri-cli`
- macOS / Windows / Linux

---

## Quick Start

```bash
git clone https://github.com/9t29zhmwdh-coder/StateForge
cd StateForge

cd frontend && npm install && cd ..
cargo tauri dev
```

### Usage

1. **Import**; paste source code, a log file, or describe your flow in natural language
2. **Analyze**; StateForge extracts states, transitions, events, and guards automatically
3. **Visualize**; drag-and-drop diagram editor with live sync to the extracted model
4. **Generate**; export clean state machine code in Swift, Kotlin, TypeScript, Go, or Rust

---

## Supported Inputs

| Input | Formats |
|---|---|
| **Swift** | Enums, TCA Reducers, `@Observable` / `@Published` |
| **Kotlin** | Sealed classes, `when` expressions, ViewModel state |
| **TypeScript** | XState `createMachine`, union types, Redux reducers |
| **Go** | iota constants, switch FSMs, `SetState()` / `Transition()` |
| **Logs** | key=value, JSON, Nginx, Docker, Syslog |

---

## Diagram Formats

| Format | Use Case |
|---|---|
| Interactive (React Flow) | Drag-and-drop editing, live sync |
| Mermaid stateDiagram-v2 | Markdown docs, GitHub |
| GraphViz DOT | Advanced layout, CI pipelines |
| SVG | Self-contained export, presentations |

---

**Author:** [Rafael Yilmaz](https://github.com/9t29zhmwdh-coder) · **Status:** Active · v0.1.0 · **License:** MIT
