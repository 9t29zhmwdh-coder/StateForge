# Getting Started with StateForge

This guide walks you through setting up and running StateForge from scratch, even if you have never used a terminal or built a Rust/Tauri app before. StateForge runs on Windows, Linux, and macOS, so pick the section below that matches your computer.

<!-- TODO: Screenshot -->

---

## Windows

### 1. Open a terminal

Right-click the Start button and choose **Terminal** (or **Windows PowerShell** on older versions of Windows).

### 2. Check prerequisites

Run each of these commands one at a time:

```powershell
rustc --version
cargo --version
node --version
cargo tauri --version
```

If any command prints something like `'rustc' is not recognized as an internal or external command`, that tool is not installed (or not on your PATH) yet:

- **Rust / Cargo missing** → install from [rustup.rs](https://rustup.rs)
- **Node.js missing** → install from [nodejs.org](https://nodejs.org)
- **Tauri CLI missing** → once Rust is installed, run `cargo install tauri-cli`

After installing, close and reopen your terminal so the new PATH entries take effect.

### 3. Get the code

**Easiest way (no git required):**
1. Go to the [StateForge GitHub page](https://github.com/9t29zhmwdh-coder/StateForge)
2. Click the green **Code** button → **Download ZIP**
3. Extract the ZIP file somewhere convenient, e.g. `C:\Projects\StateForge`

**If you already use git:**
```powershell
git clone https://github.com/9t29zhmwdh-coder/StateForge.git
```

### 4. Build & run

In your terminal, navigate into the extracted/cloned folder, then run:

```powershell
cd StateForge
cd frontend
npm install
cd ..
cargo tauri dev
```

The first run will take a few minutes: `npm install` downloads frontend dependencies, and `cargo tauri dev` compiles the Rust backend. Once it finishes, a StateForge window should open automatically.

---

## Linux

### 1. Open a terminal

This depends on your desktop environment: try **Ctrl+Alt+T**, or look for "Terminal" in your application menu (GNOME, KDE, XFCE, etc. all ship one).

### 2. Check prerequisites

```bash
rustc --version
cargo --version
node --version
cargo tauri --version
```

If you see `command not found` for any of these, install the missing piece:

- **Rust / Cargo missing** → install from [rustup.rs](https://rustup.rs)
- **Node.js missing** → install from [nodejs.org](https://nodejs.org)
- **Tauri CLI missing** → once Rust is installed, run `cargo install tauri-cli`

You may need to restart your terminal (or run `source ~/.bashrc` / `source ~/.profile`) after installing Rust so `cargo` is on your PATH.

### 3. Get the code

**Easiest way (no git required):**
1. Go to the [StateForge GitHub page](https://github.com/9t29zhmwdh-coder/StateForge)
2. Click the green **Code** button → **Download ZIP**
3. Extract the ZIP file, e.g. into `~/Projects/StateForge`

**If you already use git:**
```bash
git clone https://github.com/9t29zhmwdh-coder/StateForge.git
```

### 4. Build & run

```bash
cd StateForge
cd frontend && npm install && cd ..
cargo tauri dev
```

The first build compiles the Rust backend and installs frontend dependencies, which can take a few minutes. Tauri also needs some system libraries (see Troubleshooting below) to open its window on Linux. Once everything succeeds, the StateForge window opens.

---

## macOS

### 1. Open a terminal

Press **Cmd+Space** to open Spotlight, type "Terminal", and press Enter.

### 2. Check prerequisites

```bash
rustc --version
cargo --version
node --version
cargo tauri --version
```

If any command says `command not found`, install the missing tool:

- **Rust / Cargo missing** → install from [rustup.rs](https://rustup.rs)
- **Node.js missing** → install from [nodejs.org](https://nodejs.org)
- **Tauri CLI missing** → once Rust is installed, run `cargo install tauri-cli`

### 3. Get the code

**Easiest way (no git required):**
1. Go to the [StateForge GitHub page](https://github.com/9t29zhmwdh-coder/StateForge)
2. Click the green **Code** button → **Download ZIP**
3. Extract the ZIP file, e.g. into `~/Projects/StateForge`

**If you already use git:**
```bash
git clone https://github.com/9t29zhmwdh-coder/StateForge.git
```

### 4. Build & run

```bash
cd StateForge
cd frontend && npm install && cd ..
cargo tauri dev
```

The first run compiles the app, which takes a few minutes. Once it's done, the StateForge window opens and you can paste in source code, a log file, or a natural-language description to see it turned into a state machine diagram.

---

### Troubleshooting

| Issue | Cause | Fix |
|---|---|---|
| `'cargo' is not recognized` / `command not found: cargo` | Rust is not installed, or your terminal was opened before installing it | Install via [rustup.rs](https://rustup.rs), then open a **new** terminal window |
| `'npm' is not recognized` / `command not found: npm` | Node.js is not installed or not on PATH | Install via [nodejs.org](https://nodejs.org), then reopen your terminal |
| PowerShell says a `.ps1` script "cannot be loaded because running scripts is disabled" | Windows execution policy blocks local scripts | Run `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` in an elevated PowerShell, then retry |
| Build fails with linker errors mentioning `link.exe` or MSVC (Windows) | Missing C++ build tools required by Rust on Windows | Install "Desktop development with C++" via the [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) installer |
| `error: failed to run custom build command for glib-sys` or missing `webkit2gtk` (Linux) | Tauri needs WebKitGTK and related system libraries | Install them, e.g. on Debian/Ubuntu: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` |
