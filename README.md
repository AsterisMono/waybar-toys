# waybar-toys

A collection of lightweight, high-performance status bar tools and widgets for [Waybar](https://github.com/Alexays/Waybar) (and Polybar), written in Rust.

## Crates

| Crate | Description | Documentation |
| --- | --- | --- |
| [`waycat`](waycat) | Animated CPU load indicator featuring dynamic frame rate and idle sleeping states. | [waycat/README.md](waycat/README.md) |
| [`waycodex`](waycodex) | Monitor OpenAI Codex 5-hour and weekly usage quotas, banked resets, and reset countdowns. | [waycodex/README.md](waycodex/README.md) |
| [`wayherdr`](wayherdr) | Live agent fleet status monitor for [herdr](https://herdr.dev) coding agent servers. | [wayherdr/README.md](wayherdr/README.md) |

---

## Overview

### 🐱 [waycat](waycat)

A lightweight animated CPU usage cat based on [polycat](https://github.com/zzqmt/polycat).
- Animation speed dynamically scales with CPU load (`/proc/stat`).
- Automatically switches to a sleeping animation when the system is idle.
- Highly customizable polling interval, frame glyphs, smoothing, and output formats.

### 🤖 [waycodex](waycodex)

A tool to track OpenAI Codex usage and quota windows.
- Displays 5-hour and 1-week usage percentages, banked usage resets, and countdown to next reset.
- Automatically reads existing OAuth credentials from Codex CLI (`~/.codex/auth.json`), pi (`~/.pi/agent/auth.json`), or environment variables.
- Safe read-only operation: never modifies tokens or redeems banked resets.

### 🐑 [wayherdr](wayherdr)

A real-time status monitor for coding agent workflows in [herdr](https://herdr.dev).
- Connects directly to herdr's newline-delimited JSON UNIX socket API.
- Shows counts for `working`, `blocked`, `done`, and `idle` agents.
- Supports conditional formatting templates (e.g. show `⛔ blocked` only when non-zero).

---

## Building & Installation

### Prerequisites

- Rust toolchain (1.80+) or [devenv](https://devenv.sh) / `nix`
- Linux system with `/proc/stat` support (for `waycat`)

### Build Everything

```sh
# Build release binaries for all workspace crates
cargo build --release --workspace
```

Binaries will be output to `target/release/`:
- `target/release/waycat`
- `target/release/waycodex`
- `target/release/wayherdr`

### Build an Individual Crate

```sh
cargo build --release -p waycat
cargo build --release -p waycodex
cargo build --release -p wayherdr
```

### Install Font (for `waycat`)

`waycat` uses the bundled `polycat` font. You can install it using `make`:

```sh
cd waycat
make install PREFIX="$HOME/.local"
fc-cache -f
```

---

## Waybar Configuration Example

Here is an example snippet showing how to integrate all three modules into your `~/.config/waybar/config.jsonc`:

```jsonc
{
  "modules-right": [
    "custom/herdr",
    "custom/codex",
    "custom/waycat"
  ],

  "custom/waycat": {
    "exec": "waycat --format-enabled --format '$rcpu $frame'",
    "interval": 0,
    "format": "{}"
  },

  "custom/codex": {
    "exec": "waycodex",
    "interval": 60,
    "format": "{}"
  },

  "custom/herdr": {
    "exec": "wayherdr -f '⚒ {working}{{#blocked}} ⛔ {blocked}{{/blocked}}{{#done}} ✓ {done}{{/done}}'",
    "interval": 3,
    "format": "{}"
  }
}
```

In `~/.config/waybar/style.css`, make sure the `polycat` font is available for `waycat`:

```css
#custom-waycat {
  font-family: "polycat", monospace;
}
```

---

## Development

If you use [devenv](https://devenv.sh) or `direnv`:

```sh
devenv shell
```

Workspace commands:

```sh
# Typecheck
cargo check --workspace

# Run all tests
cargo test --workspace

# Linter and formatting
cargo clippy --workspace
cargo fmt --check
```

---

## License

Each crate is individually licensed. See the respective crate directories for license details (e.g., [waycat LICENSE](waycat/LICENSE)).
