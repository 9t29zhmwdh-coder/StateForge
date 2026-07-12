# StateForge: Professional Repo Skeleton

**Generated:** 2026-06-16 | **Earliest commit:** 2026-06-12 | **Release:** v0.1.0

## Canonical File Tree

```
StateForge/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
├── sf-core/
│   └── src/
│       ├── extractor/
│       │   ├── code/
│       │   ├── log/
│       │   └── nl/
│       ├── fsm/
│       └── exporter/
│           ├── mermaid/
│           ├── dot/
│           └── svg/
├── sf-cli/
├── src-tauri/
│   └── src/
│       ├── main.rs
│       ├── error.rs
│       ├── state.rs
│       └── commands/
├── frontend/
│   └── src/
│       ├── stores/
│       └── components/
├── ARCHITECTURE.md
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── LICENSE
├── PRIVACY.md
├── README.md
├── README.de.md
├── ROADMAP.md
├── SECURITY.md
└── SKELETON.md
```

## Migration Checklist

- ARCHITECTURE.md ✅
- PRIVACY.md ✅
- ROADMAP.md ✅
- CODE_OF_CONDUCT.md ✅
- SECURITY.md ✅
- CHANGELOG.md ✅
- .github/ISSUE_TEMPLATE/ ✅
- .github/PULL_REQUEST_TEMPLATE.md ✅
- .github/workflows/ci.yml ⚠️: requires `workflows` OAuth scope (run: gh auth refresh -s workflows)
- GitHub Release v0.1.0 ✅

## CI Workflow (push manually after: gh auth refresh -s workflows)

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check & Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: cargo check
        run: cargo check --workspace
      - name: cargo test
        run: cargo test --workspace
      - name: cargo clippy
        run: cargo clippy --workspace -- -D warnings
      - name: cargo fmt
        run: cargo fmt --all -- --check

  build:
    name: Build (release)
    runs-on: ubuntu-latest
    needs: check
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: cargo build --release
        run: cargo build --workspace --release
```

## Reusable from this repo

- `sf-core/src/fsm/`: normalized `FsmModel` struct (states, transitions, guards, actions) reusable as a state machine primitive in other RayStudio tools that need FSM representation
- `sf-core/src/exporter/mermaid/`: generic Mermaid `stateDiagram-v2` renderer, reusable wherever diagram output is needed
- `CODE_OF_CONDUCT.md`: identical across all RayStudio repos, copy as-is

---

*StateForge, RayStudio · Rafael Yilmaz · MIT License · 2026*
