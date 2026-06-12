<p align="center">
  <img src="RayStudio.png" alt="RayStudio" width="120" />
</p>

<h1 align="center">StateForge</h1>
<p align="center"><strong>Automatische State-Machine-Generierung aus Code, Logs und UI-Flows</strong></p>
<p align="center">
  <a href="README.md">English</a> ·
  <a href="https://github.com/9t29zhmwdh-coder/StateForge">GitHub</a> ·
  <a href="LICENSE">MIT-Lizenz</a>
</p>

---

## Was ist StateForge?

StateForge ist ein Entwicklerwerkzeug, das **automatisch State Machines** aus Quellcode, Log-Dateien, API-Sequenzen oder natürlichsprachigen Beschreibungen extrahiert — und diese als interaktive Diagramme darstellt.

Es hilft dabei, komplexe Abläufe zu verstehen, automatisch zu dokumentieren und sauberen State-Machine-Code in der Zielsprache zu regenerieren.

## Funktionen

| Modul | Beschreibung |
|---|---|
| **Code-Parser** | Extrahiert State Machines aus Swift, Kotlin, TypeScript, Go, Rust |
| **Log-Analyzer** | Rekonstruiert Zustandsflüsse aus Log-Dateien |
| **Diagramm-Engine** | Rendert Mermaid, GraphViz DOT, SVG, interaktives React Flow |
| **Code-Generator** | Generiert idiomatischen State-Machine-Code in 5 Sprachen |
| **KI-Integration** | Claude / Ollama — Maschinen anreichern oder aus Beschreibung erstellen |
| **Plugin-System** | Erweiterbar mit eigenen Parsern |

## Technologie

- **Core** — Rust (async, petgraph, sqlx, regex)
- **Desktop** — Tauri v2
- **Frontend** — React, TypeScript, Tailwind CSS, @xyflow/react, Mermaid

## Schnellstart

```bash
# Voraussetzungen: Rust, Node.js 18+, npm
git clone https://github.com/9t29zhmwdh-coder/StateForge
cd StateForge

npm --prefix frontend install
cargo tauri dev
```

## Verwendung

1. **Importieren** — Quellcode einfügen, eine Log-Datei laden oder den Ablauf in natürlicher Sprache beschreiben
2. **Analysieren** — StateForge extrahiert Zustände, Transitionen, Events und Guards automatisch
3. **Visualisieren** — Drag-and-Drop-Diagrammeditor mit Live-Sync zum extrahierten Modell
4. **Generieren** — Sauberen State-Machine-Code in Swift, Kotlin, TypeScript, Go oder Rust exportieren

## Unterstützte Eingabeformate

- **Swift** — Enums, TCA Reducer, `@Observable` / `@Published`
- **Kotlin** — Sealed Classes, `when`-Ausdrücke, ViewModel-State
- **TypeScript** — XState `createMachine`, Union Types, Redux Reducer
- **Go** — iota-Konstanten, Switch-FSMs, `SetState()` / `Transition()`-Patterns
- **Logs** — key=value, JSON, Nginx, Docker, Syslog

## Diagrammformate

| Format | Anwendungsfall |
|---|---|
| Interaktiv (React Flow) | Drag-and-Drop-Bearbeitung, Live-Sync |
| Mermaid stateDiagram-v2 | Markdown-Docs, GitHub |
| GraphViz DOT | Erweitertes Layout, CI-Pipelines |
| SVG | Eigenständiger Export, Präsentationen |

---

<p align="right">
  <sub>von <a href="https://github.com/9t29zhmwdh-coder">RayStudio</a> &nbsp;·&nbsp; MIT-Lizenz</sub>
  &nbsp;
  <img src="RayStudio.png" alt="" width="70" align="right" />
</p>
