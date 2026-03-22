<h1 align="center">fumi</h1>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-cba6f7?style=for-the-badge&colorA=1e1e2e&logo=rust&logoColor=cba6f7">
  <img src="https://img.shields.io/badge/TUI-a6e3a1?style=for-the-badge&colorA=1e1e2e&logo=windowsterminal&logoColor=a6e3a1">
  <img src="https://img.shields.io/badge/GitHub_API-89b4fa?style=for-the-badge&colorA=1e1e2e&logo=github&logoColor=89b4fa">
</p>

<p align="center">
  A minimal TUI for browsing and selectively downloading files from GitHub repositories — without cloning.
</p>

---

## About

**fumi** is a terminal-based GitHub repository browser written in Rust. It lets you navigate any public GitHub repository, preview files inline, and download only the files you actually need — no `git clone`, no bloat.

Built with [`ratatui`](https://github.com/ratatui-org/ratatui) and the GitHub Contents API, fumi is designed to be fast, keyboard-driven, and distraction-free.

---

## Features

| # | Feature | Description |
|---|---------|-------------|
| `01` | **Selective Download** | Browse a repository tree and cherry-pick individual files — nothing more, nothing less. |
| `02` | **Inline Preview** | Preview file contents directly in the TUI before deciding to download. Scroll through the preview panel independently. |
| `03` | **Async & Concurrent** | Downloads run concurrently via a Tokio worker pool with configurable concurrency limits. |
| `04` | **Keyboard-Driven** | Vim-style keybinds (`j/k/h/l`) for fluid, mouse-free navigation. |
| `05` | **Lazy Tree Loading** | Repository tree is fetched on-demand as you navigate — no upfront API cost. |
| `06` | **Config File** | TOML-based config for auth token, output directory, and concurrency. Auto-generated on first run. |
| `07` | **Flexible Input** | Accepts `owner/repo`, `owner/repo@branch`, or full GitHub URLs directly. |

---

## Keybinds

### Browse Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `Enter` | Open directory or preview file |
| `h` / `Backspace` | Go back |
| `Space` | Toggle file selection |
| `d` | Download selected files |
| `p` | Preview focused file |
| `r` | Refresh current directory |
| `b` | Open repository input |
| `q` / `Esc` / `Ctrl+C` | Quit |

### Preview Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll preview down |
| `k` / `↑` | Scroll preview up |
| `h` / `q` / `Esc` | Close preview |

### Input Mode

| Key | Action |
|-----|--------|
| `Enter` | Confirm and load repository |
| `Esc` | Cancel (or quit if no repo loaded) |

---

## Installation

```bash
git clone https://github.com/pavelc4/fumi.git
cd fumi
cargo build --release
./target/release/fumi
```

---

## Usage

```bash
# open a repo directly
fumi owner/repo
fumi owner/repo@branch

# full GitHub URLs also work
fumi https://github.com/owner/repo

# or launch and input repo from within the TUI
fumi
```

> When launched without args, fumi opens in input mode. Type a repo and press `Enter` to load it.

---

## Configuration

Config is auto-generated on first run at `~/.config/fumi/config.toml`:

```toml
[auth]
# token = "ghp_your_token_here"  # optional — increases rate limit & enables private repos

[output]
# dir = "~/Downloads/fumi"       # default: ~/Downloads

[download]
concurrency = 4                  # number of parallel download workers
```

> Without a token, the GitHub API allows ~60 unauthenticated requests/hour. With a token, the limit increases to 5,000/hour.

---

## Architecture

```
src/
├── main.rs       # entrypoint, CLI args, terminal setup
├── run.rs        # main event loop
├── config.rs     # TOML config loading & defaults
├── event.rs      # AppCommand / WorkerEvent channel types
├── app/
│   ├── state.rs  # App state (cursor, tree, selections, mode)
│   └── actions.rs# NavAction dispatch logic
├── github/       # GitHub API types & RepoTarget
├── handler/
│   └── key.rs    # keyboard input handler (all modes)
├── ui/           # ratatui rendering (filetree, preview, statusbar)
└── worker/       # async Tokio workers (API fetch, download)
```

---

## License

fumi is open-sourced software licensed under the **MIT License**.  
See the [LICENSE](LICENSE) file for more information.