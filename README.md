# DragonIDE

[![CI](https://github.com/cyberworrier8088/DragonIDE/actions/workflows/ci.yml/badge.svg)](https://github.com/cyberworrier8088/DragonIDE/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Status](https://img.shields.io/badge/status-work%20in%20progress-orange)


A lightweight Code editor built with **Rust** and **Tauri 2**, with a plain HTML/CSS/JavaScript front end.
**No npm, no Node packages**: the whole app is RUst + Vanilla web code.

> **Work in progress.** DragonIDE is not ready for daily use yet. Things chnage every day and there are no installers yet. You can build it from source and try it.

![DragonIDE ScreenShot](docs/screenshot.png)

## Contents

- [Features](#features)
- [Keyboard ShortCuts](#keyboard-shortcuts)
- [Build and run](#Build-and-run)
- [project layout](#project-layout)
- [Why Tauri](#why-tauri)
- [License](#licence)

## Features

**Editor**
- Syntax highlighting, line numbers, cursor position
- Undo / redo per tab, auto indent, Tab indentaion
- Bracket matching and auto-closing brackets
- Find & replace (case sensitive, whole word, regex)
- Go to line
- Zoom the editor text
- Auto-save, with a dot on tabs that have unsaved changes

**File**
- File Explorer with new file, new folder, rename, delete and copy path
- Explorer refreshes after terminal commands and keeps open folders open
- Remembersthe last opened folder
- Quick Open: Find any file by name
- Search text across all files in the folder
- Image preview for png, jpg, gif, webp, bmp and ico

**Run and terminal**
- Run Rust, JavaScript and Python files, with a Stop button
- Build-in terminal: live output, `cd`, command history, Ctrl+C to stop

**Layout and look**
- Drag tabsto reorder them
- Resize the explorer and bottom panel; hide or show them
- Themes: Dark, Light, Nord, Monokai and Glass (Glass is still being finished)
- English and Spanish
- Custom Title Barrr

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+S` | Save |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo |
| `Ctrl+P` | Quick Open (file by name) |
| `Ctrl+Shift+F` | Search in all files |
| `Ctrl+F` / `Ctrl+H` | Find / replace |
| `Ctrl+G` | Go to line |
| `Ctrl+Shift+\` | Jump to matching bracket |
| `Ctrl+Alt+R` | Run code (press again to stop) |
| `Ctrl+C` (in terminal) | Stop the running command |
| `Ctrl+B` | Show / hide the explorer |
| `Ctrl+J` | Show / hide the bottom panel |
| `Ctrl+=` / `Ctrl+-` / `Ctrl+0` | Zoom in / out / reset |

On macOS, `Cmd` works in place of `Ctrl` for most shortcuts.

## Build and run

## 1. Install the tools

- [Rust](https://rustup.rs) (stable)
- The Tauri CLI:
```
  cargo install tauri-cli --version "^2" --locked
  ```
- System libraries for Tauri:
  - **Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++"). WebView2 is already part of Windows 10 and 11.
  - **macOS:** Xcode Command Line Tools: `xcode-select --install`
  - **Linux (Debian / Ubuntu):**
    ```
    sudo apt install libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
    ```
  - Other systems: see the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

### 2. Run it

```
git clone https://github.com/cyberworrier8088/DragonIDE.git
cd DragonIDE/DragonFoxxDE
cargo tauri dev
```

The first build taks a few minutes. After that it starts much faster.

To make an installer for your own computer: `cargp tauri Build`

> **Run Code** uses the tools on your computer: 'rustc' for Rust, 'node' for JavaScript and 'Python' / 'python3' for python. Install the ones you need

## Project layout

```
DragonIDE/
├── DragonFoxxDE/        the IDE (Tauri app)
│   ├── src/             Rust backend: files, search, terminal, run code
│   ├── web/             front end: index.html, css/, js/, locales/
│   └── tauri.conf.json
├── FoxAi/               a small code-completion AI model, written in Rust
├── journels/            daily development journal with screenshots
└── .github/workflows/   CI: builds and checks every push
```

**FoxAi** is a small Transformer model for code completion, trained from scartch with [burn](https://burn.dev). It is not good enogh to use in the IDE yet. See [FoxAi/README.md](FoxAi/README.md).

The **[journels](journels/)** folder is a day-by-day diary of how DragonIDE was built.

## Why Tauri

| | Tauri 2 | Electron |
|---|---|---|
| Backend | Rust | Node.js |
| App size | about 2–10 MB | about 80–150 MB |
| Memory use | about 30–50 MB | about 130–400 MB |
| Renders with | the system WebView | a bundled Chromium |

## License

[MIT](LICENCE) © 2026 cyberworrier8088(imu)