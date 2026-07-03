<div align="center">
  <img src="RayStudio.png" alt="RayStudio Logo" width="120"/>

  <h1>StateForge</h1>
</div>

[🇬🇧 English Version](README.md)

**Automatische State-Machine-Generierung aus Code, Logs und UI-Flows, entwickelt mit Rust und Tauri.**

StateForge extrahiert automatisch State Machines aus Quellcode, Log-Dateien, API-Sequenzen oder natürlichsprachigen Beschreibungen und stellt diese als interaktive Diagramme dar. Es hilft, komplexe Abläufe zu verstehen, automatisch zu dokumentieren und sauberen State-Machine-Code in der Zielsprache zu regenerieren.

[![CI](https://github.com/9t29zhmwdh-coder/StateForge/actions/workflows/ci.yml/badge.svg)](https://github.com/9t29zhmwdh-coder/StateForge/actions) ![Platform](https://img.shields.io/badge/Platform-macOS_%7C_Windows_%7C_Ubuntu-lightgrey) ![Rust](https://img.shields.io/badge/Rust-CE422B?logo=rust&logoColor=white) ![Tauri](https://img.shields.io/badge/Tauri-24C8D8?logo=tauri&logoColor=white) ![AI | Claude Code](https://img.shields.io/badge/AI-Claude_Code-black?logo=anthropic&logoColor=white) ![AI | Copilot](https://img.shields.io/badge/AI-Copilot-black?logo=github&logoColor=white) ![AI | Ollama](https://img.shields.io/badge/AI-Ollama-black?logo=ollama&logoColor=white)
![Plattform](https://img.shields.io/badge/Plattform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![Lizenz](https://img.shields.io/badge/Lizenz-MIT-green)

---

## Funktionen

| Funktion | Beschreibung |
|---|---|
| **Code-Parser** | Extrahiert State Machines aus Swift, Kotlin, TypeScript, Go, Rust |
| **Log-Analyzer** | Rekonstruiert Zustandsflüsse aus Log-Dateien (JSON, Plaintext, Nginx, Syslog) |
| **Diagramm-Engine** | Rendert Mermaid, GraphViz DOT, SVG, interaktives React Flow |
| **Code-Generator** | Generiert idiomatischen State-Machine-Code in 5 Sprachen |
| **KI-Integration** | Claude / Ollama: Maschinen anreichern oder aus Beschreibung erstellen |
| **Plugin-System** | Erweiterbar mit eigenen Parsern via Rust-Trait |

---

## Voraussetzungen

- [Rust](https://rustup.rs/) 1.77+
- [Node.js](https://nodejs.org/) 20+
- [Tauri CLI v2](https://tauri.app/): `cargo install tauri-cli`
- macOS / Windows / Linux

---

## Schnellstart

```bash
git clone https://github.com/9t29zhmwdh-coder/StateForge
cd StateForge

cd frontend && npm install && cd ..
cargo tauri dev
```

### Verwendung

1. **Importieren**; Quellcode einfügen, eine Log-Datei laden oder den Ablauf in natürlicher Sprache beschreiben
2. **Analysieren**; StateForge extrahiert Zustände, Transitionen, Events und Guards automatisch
3. **Visualisieren**; Drag-and-Drop-Diagrammeditor mit Live-Sync zum extrahierten Modell
4. **Generieren**; Sauberen State-Machine-Code in Swift, Kotlin, TypeScript, Go oder Rust exportieren

---

## Unterstützte Eingaben

| Eingabe | Formate |
|---|---|
| **Swift** | Enums, TCA Reducer, `@Observable` / `@Published` |
| **Kotlin** | Sealed Classes, `when`-Ausdrücke, ViewModel-State |
| **TypeScript** | XState `createMachine`, Union Types, Redux Reducer |
| **Go** | iota-Konstanten, Switch-FSMs, `SetState()` / `Transition()` |
| **Logs** | key=value, JSON, Nginx, Docker, Syslog |

---

## Diagrammformate

| Format | Anwendungsfall |
|---|---|
| Interaktiv (React Flow) | Drag-and-Drop-Bearbeitung, Live-Sync |
| Mermaid stateDiagram-v2 | Markdown-Docs, GitHub |
| GraphViz DOT | Erweitertes Layout, CI-Pipelines |
| SVG | Eigenständiger Export, Präsentationen |

---

**Autor:** [Rafael Yilmaz](https://github.com/9t29zhmwdh-coder) · **Status:** Active · ![version](https://img.shields.io/github/v/release/9t29zhmwdh-coder/StateForge?label=\&color=6b7280\&style=flat-square) · **Lizenz:** MIT
