# StateForge

[🇬🇧 English Version](README.md)

**Automatische State-Machine-Generierung aus Code, Logs und UI-Flows — entwickelt mit Rust + Tauri.**

StateForge extrahiert automatisch State Machines aus Quellcode, Log-Dateien, API-Sequenzen oder natürlichsprachigen Beschreibungen und stellt diese als interaktive Diagramme dar. Es hilft, komplexe Abläufe zu verstehen, automatisch zu dokumentieren und sauberen State-Machine-Code in der Zielsprache zu regenerieren.

![Rust](https://img.shields.io/badge/Rust-1.77+-orange?logo=rust)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)
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
| **KI-Integration** | Claude / Ollama — Maschinen anreichern oder aus Beschreibung erstellen |
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

1. **Importieren** — Quellcode einfügen, eine Log-Datei laden oder den Ablauf in natürlicher Sprache beschreiben
2. **Analysieren** — StateForge extrahiert Zustände, Transitionen, Events und Guards automatisch
3. **Visualisieren** — Drag-and-Drop-Diagrammeditor mit Live-Sync zum extrahierten Modell
4. **Generieren** — Sauberen State-Machine-Code in Swift, Kotlin, TypeScript, Go oder Rust exportieren

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

**Author:** [Rafael Yilmaz](https://github.com/9t29zhmwdh-coder) · **Status:** Framework Preview · **Last Updated:** Juni 2026
