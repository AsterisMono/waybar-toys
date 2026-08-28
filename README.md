# waybar-toys

A Rust workspace for Waybar tools.

## Workspace crates

- [`waycat`](waycat) — an animated CPU usage widget
- [`waycodex`](waycodex) — OpenAI Codex usage limits and banked resets
- [`wayherdr`](wayherdr) — Herdr agent status for Waybar

## Development

Enter the development shell with `devenv shell`, then use Cargo across the workspace:

```sh
cargo check --workspace
cargo test --workspace
```
