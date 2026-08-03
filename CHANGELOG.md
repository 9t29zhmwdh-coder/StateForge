# Changelog, StateForge

All notable changes to this project will be documented in this file.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

---

## [1.2.1] - 2026-08-03

### Fixed

- Corrects a claim in the 1.2.0 entry. It said that leaving `rounded` in place would have halved every corner radius, because version 4 shifted the scale. That is wrong. Measured directly under Tailwind 4.3.3: `rounded` is still 0.25rem and is kept as an alias. The scale did shift, but under the name `rounded-sm`, which now means 0.25rem where it meant 0.125rem before. The dangerous case is source that already used `rounded-sm`, and this repository never did, so the rename changed nothing visually. The migration itself was correct; the reason given for it was not.

---

## [1.2.0] - 2026-08-03

### Changed

- Tailwind CSS 3 to 4. `tailwind.config.ts` is gone; the six `sf` colours and the mono stack are theme variables in the stylesheet now, and PostCSS uses `@tailwindcss/postcss`.
- Three utility renames went through seven components. `rounded` became `rounded-sm` because version 4 shifted the radius scale by one step and the old name now means half as much, `outline-none` became `outline-hidden`, and `flex-shrink-0` became `shrink-0`.
- autoprefixer is no longer a dependency. Version 4 handles prefixing itself, and the generated stylesheet comes out as the same file with and without it, down to the content hash.

---

## [1.1.6] - 2026-08-02

### Changed

- React 18 to 19, together with `react-dom` and both type packages. Dependabot had split these across separate pull requests and neither could be merged alone: `@types/react-dom` 18 requires `@types/react` 18, so raising either one left npm unable to resolve the peer dependency. All four move together here.
- No code changes were needed, checked against the list of things React 19 removes rather than assumed: `createRoot` is already in use, and there are no string refs, no `propTypes`, no argument-less `useRef`, no `forwardRef`, no `defaultProps` and no callback refs. Typecheck and production build both clean.

---

## [1.1.5] - 2026-08-02

### Changed

- `sqlx` 0.8.6 to 0.9.0, merged since 1.1.4 and carried by this version. Unlike LogLens, this project compiled against 0.9 without changes: the new version rejects SQL that is not a `&'static str`, and every statement here already is one. That refusal is what exposed an injection in LogLens; here it confirmed there was nothing to expose.

---

## [1.1.4] - 2026-08-02

### Security

- `keyring` switches from `crypto-openssl` to `crypto-rust`, which removes OpenSSL from the Linux build. The Secret Service protocol encrypts the session between the application and the keyring daemon, and until now that encryption came from the OpenSSL C library, pulled in through `keyring` and `secret-service`. `crypto-rust` implements the same algorithms the specification prescribes, AES-CBC with SHA-2 and HKDF, using the RustCrypto crates instead. The wire format is defined by the specification, so an existing keyring stays readable.
- What this buys: OpenSSL leaves the dependency tree entirely, `cargo tree -i openssl --target x86_64-unknown-linux-gnu` no longer finds a package. With it goes a C library with a long CVE history and the requirement to have its development headers present when building for Linux. macOS and Windows are unaffected either way, since both use their native keychain and never compiled this path.

---

## [1.1.3] - 2026-08-01

### Changed

- `reqwest` 0.12.28 to 0.13.4. This replaces the TLS stack rather than merely raising a number: `native-tls` and OpenSSL leave, `rustls-platform-verifier` and `aws-lc-rs` arrive. Certificate verification matters here, because the AI enhancement talks to `https://api.anthropic.com`, so the handshake was tested against that host under both versions before merging. Both reach it and return `401 Unauthorized` without a key, which is the proof that the handshake and the certificate check completed.
- OpenSSL does not leave this tree entirely. It still arrives through `keyring` 3 and `secret-service`, and only in the Linux build. `reqwest` cannot remove that one.

---

## [1.1.2] - 2026-08-01

### Changed

- Dependency updates merged since 1.1.1, carried by this version rather than one release each: `thiserror` 1.0.69 to 2.0.19 and `dirs` 5.0.1 to 6.0.0. The `dirs` bump was checked rather than assumed, because `data_local_dir()` decides where the database lives: both versions were built and their paths compared, and the source diff between `dirs-sys` 0.4.1 and 0.5.0 is a single Windows FFI line, `HANDLE::default()` replaced by `null_mut()`. No path logic changed on any platform.

### Removed

- Five declared dependencies that no code references: petgraph, tracing, dashmap, glob, indexmap. They were compiled on every build, shipped their own transitive tree, counted toward the supply-chain surface, and produced Dependabot pull requests proposing upgrades to code nobody calls. Verified by removing them and running `cargo check`, `cargo clippy` with `-D warnings` and the full test suite, all clean.

---

## [1.1.1] - 2026-08-01

### Changed

- Dependabot no longer retries the `glib` update it cannot perform. GHSA-wrw7-89jp-8q8g is fixed in 0.20, and this project cannot reach it: `tauri` 2.x pins `gtk ^0.18`, `gtk` 0.18 requires `glib ^0.18`, and no patched 0.18.x exists, so cargo rejects the upgrade rather than resolving it. Three attempts had already failed identically, each one a red run on `main` that carried no information. Only the unreachable versions are ignored, so a backported 0.18.x fix would still arrive, and the advisory itself stays visible in the Security tab. The block goes away when Tauri moves to gtk-rs 0.20, the condition already recorded in `SECURITY.md`.

---

## [1.1.0] - 2026-07-31

### Added

- **A local AI backend that actually exists.** `OllamaAnalyzer` implements the same `AiAnalyzer` contract as the Claude client, running enhancement and extraction against your own Ollama instance. Nothing leaves the device with it selected.

### Fixed

- **The Ollama setting now decides where data goes.** Until this release the settings screen offered a backend choice and `get_analyzer` ignored it, always building the Claude client. Anyone who picked the local option still sent their state machine to Anthropic. The setting is read and honoured.
- A model response with a closing brace before the opening one, for example `} see above {`, panicked instead of erroring. `find` and `rfind` can cross, and the resulting slice had a start greater than its end. The order is checked before slicing.

### Changed

- Ollama is the default backend for new installations, matching the rest of the portfolio. Stored settings are untouched.
- The JSON-to-state-machine mapping moved from `claude.rs` into `ai/mod.rs`, where both backends share it. Duplicating roughly eighty lines into the new analyzer would have let the two providers interpret the same model output differently over time. `claude.rs` drops from 139 lines to 68 as a result.

---

## [1.0.12] - 2026-07-31

### Fixed

- `SECURITY.md` claimed "no external network calls except localhost:11434 (Ollama)". The opposite is true: AI enhancement always goes to Anthropic, because `get_analyzer` builds the Claude client regardless of the backend setting and no local analyzer exists in the codebase. The README has said as much since the Ollama option was added; the security policy contradicted it. It now states where data actually goes, that the Ollama setting does not change that, and that nothing is sent without a stored key.
- The supported-versions table still listed `0.1.x`, a line that no longer exists.

### Added

- `SECURITY.md` records GHSA-wrw7-89jp-8q8g against `glib` 0.18.5, which cannot be fixed from this repository because Tauri 2.11.5 pins `gtk ^0.18` and no patched 0.18.x exists.

---

## [1.0.11] - 2026-07-31

### Changed

- Both READMEs now open with a state machine everyone recognises, the order status field that exists only as scattered `if` branches and a `status` column, rather than with the extraction the app performs. A short paragraph says that designing a new state machine is faster done by writing the Mermaid by hand.

---

## [1.0.10] - 2026-07-30

### Added

- `Cargo.lock` is committed. It was listed in `.gitignore`, so every build resolved dependencies afresh and no two builds were guaranteed to use the same versions. For an application rather than a library the lock file belongs in the repository: it is what makes a release reproducible and what lets a security advisory be checked against what actually shipped.

---

## [1.0.9] - 2026-07-30

### Changed

- The `Check` job runs on Linux, macOS and Windows instead of macOS alone. The release builds artifacts for all three, so a fault that only shows on one of the other two reached a release before anything noticed. The keychain backend fixed in 1.0.8 is exactly that kind of fault: it can be absent per platform, independently of the others.
- The Linux leg installs the GTK and WebKit packages that Tauri builds against. The runner does not ship them, so `cargo check` failed at `gobject-2.0` before reaching any code. The release workflow already installed the same packages, which is why releases worked while this job did not.
- The ruleset now requires `Check (ubuntu-latest)`, `Check (macos-latest)` and `Check (windows-latest)` in place of the single `Check`. Adding a matrix renames the job, so leaving the old context required would have left a check that can never report again, which is how dependency pull requests were blocked here earlier this month.

---

## [1.0.8] - 2026-07-30

### Security

- `keyring` now names a platform backend for every target, so the Claude API key actually reaches the credential store of the operating system. It was declared without any platform feature, which compiles and raises no error but falls back to a store held in process memory. The key was gone after every restart, and `settings.rs` reads it with `.ok().and_then(...)`, so the loss was silent. This change reached `main` in 1.0.7 without a changelog entry of its own; recording it here rather than leaving it undocumented.

### Added

- A test that stores a secret and reads it back from a second process, on all three target platforms rather than macOS alone. StateForge ships `.dmg`, `.msi` and `.deb` artifacts, and each platform has its own backend that can be missing independently.
- The test separates a missing service from a missing backend. The in-memory fallback never fails to write, so a write error proves a real backend is compiled in and only its service is absent, which is the normal state of a Linux CI runner without a D-Bus secret service. A write that succeeds while a second process finds nothing is the defect.
- The second process is the test binary re-run, not `/usr/bin/security`. The keychain grants read access per application, so a different binary asking for an item it did not create raises an authorisation dialog that blocks CI and interrupts whoever is at the keyboard.

---

## [1.0.7] - 2026-07-29

### Security

- The release workflow no longer grants `contents: write` for its whole run. The permission moves to the one job that publishes the release, and everything else runs with `contents: read`. OpenSSF Scorecard scores the Token-Permissions check 0 out of 10 whenever any workflow holds a top-level write permission, regardless of how little of the run needs it, so this single line was what held the check at zero.
### Added

- `frontend/src/vite-env.d.ts`, referencing `vite/client`. Vite has always declared modules for `*.css` and the other asset types it handles, but nothing in this project pulled that declaration in. TypeScript 5 accepts the untyped side-effect import of `index.css` regardless, so the gap stayed invisible; TypeScript 7 rejects it with `TS2882`. The file belongs to Vite's own project scaffold and was simply missing, so this closes an existing hole rather than preparing for a specific upgrade.

---

## [1.0.6] - 2026-07-29

### Changed

Dependency and workflow updates merged since 1.0.5:

- chore(ci): bump the actions group across 1 directory with 3 updates
- chore(deps): bump the npm group across 1 directory with 4 updates

---

## [1.0.5] - 2026-07-28

### Fixed

- The CodeQL job requested `packages: read`, `actions: read` and `contents: read` at job level, repeating grants the workflow level already provides. OpenSSF Scorecard counts that as excessive token permissions and scores `Token-Permissions` at 0 out of 10 for it. The job now requests only `security-events: write`, which is the one grant that genuinely exceeds the workflow default.

## [1.0.4] - 2026-07-28

### Changed

- CodeQL moved from GitHub's default setup to an advanced setup with a committed `.github/workflows/codeql.yml`. The default setup skips pull requests that touch no code of a given language, so a dependency pull request changing only a lock file reported `skipping` on the required `Analyze (...)` checks forever and could never be merged. The workflow runs on every pull request regardless of what changed. It also uses the `security-extended` query suite, which the default setup does not allow choosing. Required checks are unchanged: verified on `BugRadar` that all eight, the generic `CodeQL` check included, turn green under this setup.
- Dependabot now groups only minor and patch updates per ecosystem; majors arrive as individual pull requests. The previous grouping put React 18 to 19, Tailwind 3 to 4 and similar breaking changes into one pull request together with urgently needed security patches, which made the whole batch unreviewable and unmergeable. Actions stay grouped wholesale. Follows `engineering-standards` v0.11.0.

## [1.0.3] - 2026-07-28

### Security

- `postcss` updated to 8.5.24, closing a high-severity path traversal in the source map auto-loading via `sourceMappingURL` that affects all versions up to and including 8.5.17.
- `dompurify` to 3.4.12, closing a low-severity advisory where `CUSTOM_ELEMENT_HANDLING` bypasses `afterSanitizeElements` for allowed elements.

Applied as a normal pull request rather than by merging Dependabot's, because Dependabot pull requests cannot currently pass this repository's required checks: CodeQL runs through GitHub's default setup, which does not trigger on a pull request that only touches a lock file, so its checks report `skipping` and never turn green. Bypassing a required check is not an option per `standards/ci-cd.md` section 7, so the fix takes the route that runs the full pipeline.

## [1.0.2] - 2026-07-28

### Added

- `.github/dependabot.yml`, covering GitHub Actions, the Cargo workspace and the frontend npm packages, with grouped weekly updates. The file was missing, and without it there are no version updates at all: security alerts only fire for disclosed vulnerabilities. Follows `engineering-standards` v0.10.0.

### Fixed

- `frontend/package.json` carried version 0.2.6 while the workspace and `tauri.conf.json` were on 1.0.1, the tagged version. All manifests now agree, so the next bump can touch every file that carries a version, as `release-process.md` section 2 requires.
- `actions/checkout` was pinned to two different SHAs across the workflows. All now use v7.0.1 with the full version in the comment.

## [1.0.1] - 2026-07-20

### Changed

- OpenSSF Scorecard workflow and badge.
- `copilot-instructions.md` for consistent AI-assisted contributions.
- Coverage reporting in CI (cargo-tarpaulin).
- Split the README's security/CI badges onto their own line, separate from the platform/tech/AI badges (they were rendering as a single merged line).

## [1.0.0] - 2026-07-17

First stable release: a real, packaged, installable distribution exists
for end users. Real macOS/Windows/Linux installers (DMG, NSIS, AppImage/deb/rpm).

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
