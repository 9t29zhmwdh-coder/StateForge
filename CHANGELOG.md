# Changelog, StateForge

All notable changes to this project will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [0.2.7] - 2026-07-17

### Changed
- CI: added an explicit `permissions: contents: read` block to the workflow(s) that were missing one (CodeQL `actions/missing-workflow-permissions`), narrowing the default GITHUB_TOKEN scope.

## [0.2.6] - 2026-07-12

### Fixed

- Removed 16 em-dashes from `ARCHITECTURE.md`, `CONTRIBUTING.md`, `SKELETON.md`, and a source comment in `crates/sf-core/src/parser/typescript.rs`. Swiss German orthography rule: no em-dash/en-dash anywhere in the repo.

## [0.2.5] - 2026-07-12

### Added

- Release workflow (`release.yml`) producing installable cross-platform artifacts (dmg, exe, msi, deb, rpm, AppImage, plus stable-named copies for the README download links) on every `v*` tag push.
- README download section (macOS DMG, Windows installer, Linux AppImage links) in both English and German.

### Fixed

- Pinned all GitHub Actions in `ci.yml` to a commit SHA instead of a mutable tag, per the portfolio's supply-chain integrity standard.
- Bumped `vite`/`@vitejs/plugin-react` to major versions 8/6 to resolve a moderate/high-severity esbuild dev-server request-forwarding vulnerability (npm audit).

## [0.2.4] - 2026-07-11

### Fixed

- SemVer correction: v0.1.1 added a genuine new feature (English/German UI translations with a language toggle) but was versioned as a patch. Renumbered v0.1.1 through v0.1.4 to v0.2.0 through v0.2.3 (same commits, tags and releases recreated at identical SHAs), per the portfolio's SemVer discipline (patch = fix, minor = feature, major = finished product).

## [0.2.3] - 2026-07-11

### Added

- Documented Dual-Licensing assessment (Community-only) in ROADMAP.md.

### Fixed

- Removed an em-dash from the SECURITY.md heading.

## [0.2.2] - 2026-07-11

### Fixed

- Updated actions/setup-node to its latest major version in CI, since GitHub is deprecating the Node.js 20 runtime and the previous version was being forced onto Node 24 and crashing during post-run cleanup.

## [0.2.1] - 2026-07-10

### Fixed

- Removed a duplicate "New here? -> beginners guide" callout from README.md (was shown twice)

### Added

- Added the "New here?" beginner guide callout to README.de.md (was missing)

## [0.2.0] - 2026-07-08

### Fixed

- Fixed a Tokio runtime panic pattern in `setup()` (`tokio::runtime::Handle::current()` calls with no reactor running) that could crash the app on launch; all runtime calls now go through Tauri's own `async_runtime::block_on`/`spawn`
- Fixed `beforeDevCommand`/`beforeBuildCommand` in `tauri.conf.json` pointing at the wrong frontend path (`cd frontend` instead of `cd ../frontend`)
- Fixed a type-alias shadowing bug in `error.rs` that broke the `Serialize` implementation for `SfError`
- Fixed `check_ai_available` to satisfy Tauri's async-command-with-reference-must-return-Result rule
- Added missing `sqlx` and `chrono` dependencies to `src-tauri/Cargo.toml` (previously only compiled because of a stale build cache)
- Removed unused `tauri-plugin-shell`, `tauri-plugin-dialog`, and `tauri-plugin-fs` plugin registrations; none were ever invoked from the frontend
- Added `src-tauri/capabilities/default.json`; without it, Tauri v2 grants close to no permissions by default
- Fixed a bug in `DiagramEditor.tsx` where nodes and edges were stored in a `useRef` disguised as `useState`, so the diagram canvas never re-rendered after loading or editing a machine; replaced with real `useState`
- Fixed a CSS `@import` ordering warning in `index.css`
- Removed dead code: unused `TransitionKind`/`NodeKind` imports, an unused `state` parameter, an unnecessary `mut`, and an unused `diagramFormat` destructure

### Changed

- Generated app icons via `cargo tauri icon`
- CI now builds and checks the `src-tauri` and `frontend` crates instead of excluding them

### Added

- English (default) and German UI translations with a language toggle

## [0.1.0] - 2026-06-12

### Added

- FSM extraction from source code files (Rust, TypeScript, Python): state enums, match arms, transition patterns via static analysis
- FSM extraction from log files: pattern matching and sequence mining
- FSM extraction from natural language descriptions via Claude (Anthropic API)
- Normalized `FsmModel` data structure: states, transitions, guards, actions, initial/final state annotations
- Interactive React Flow diagram canvas with drag, zoom, and pan
- Mermaid `stateDiagram-v2` export
- GraphViz DOT export
- Standalone SVG export
- `sf-core` Rust crate: `extractor/` (code/, log/, nl/), `fsm/`, `exporter/` (mermaid/, dot/, svg/)
- `sf-cli` binary for headless extraction and batch export
- Tauri v2 desktop shell for macOS, Windows, and Linux
- React/TypeScript frontend with Zustand state management
