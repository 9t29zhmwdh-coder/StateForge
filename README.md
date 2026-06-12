<div align="center">
  <img src="RayStudio.png" alt="RayStudio Logo" width="120"/>

  <h1>StateForge</h1>
</div>

[🇩🇪 Deutsche Version](README.de.md)

**Automatic state machine generation from code, logs, and UI flows — built with Rust + Tauri.**

StateForge automatically extracts state machines from source code, log files, API sequences, or natural language descriptions and visualizes them as interactive diagrams. It helps you understand complex flows, document them automatically, and regenerate clean state machine code in your target language.

![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/License-MIT-green)

---

## Features

| Feature | Description |
|---|---|
| **Code Parser** | Extracts state machines from Swift, Kotlin, TypeScript, Go, Rust |
| **Log Analyzer** | Reconstructs state flows from log files (JSON, plaintext, nginx, syslog) |
| **Diagram Engine** | Renders Mermaid, GraphViz DOT, SVG, interactive React Flow |
| **Code Generator** | Generates idiomatic state machine code in 5 languages |
| **AI Integration** | Claude / Ollama — enhance machines or create from natural language |
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

1. **Import** — paste source code, a log file, or describe your flow in natural language
2. **Analyze** — StateForge extracts states, transitions, events, and guards automatically
3. **Visualize** — drag-and-drop diagram editor with live sync to the extracted model
4. **Generate** — export clean state machine code in Swift, Kotlin, TypeScript, Go, or Rust

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

<div align="right">
  <sub>by</sub><br/>
  <img src="RayStudio.png" alt="RayStudio" width="70"/>
</div>

**Author:** [Rafael Yilmaz](https://github.com/9t29zhmwdh-coder) · **Status:** Framework Preview · **Last Updated:** Juni 2026
