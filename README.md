# waycat

A lightweight animated CPU-load cat for [Waybar](https://github.com/Alexays/Waybar) and [Polybar](https://github.com/polybar/polybar), written in Rust and based on [polycat](https://github.com/zzqmt/polycat).

![waycat demo](assets/waycat-demo.gif)

The animation speeds up with CPU usage and switches to a sleeping animation when idle. Configuration is entirely command-line based.

## Install

Requires a Rust toolchain and Linux (`/proc/stat`).

```sh
make
make install PREFIX="$HOME/.local"
fc-cache -f
```

This installs the binary and the bundled `polycat` font. Use `sudo make install` for a system-wide installation under `/usr/local`.

## Waybar

```jsonc
"custom/waycat": {
  "exec": "waycat --format-enabled --format '$rcpu $frame'",
  "interval": 0,
  "format": "{}"
}
```

Add `custom/waycat` to a modules array and load the font in `style.css`:

```css
* {
  font-family: "polycat", sans-serif;
}
```

## Polybar

```ini
[module/waycat]
type = custom/script
exec = waycat --format-enabled --format '$rcpu $frame'
tail = true

[bar/main]
font-1 = "polycat"
modules-right = waycat
```

## Options

```sh
waycat --help
```

Common options:

- `--low-rate`, `--high-rate` — animation FPS range
- `--poll-period` — CPU polling interval in milliseconds
- `--smoothing-enabled=false` — disable load smoothing
- `--sleeping-enabled=false` — disable the idle animation
- `--frames`, `--sleeping-frames` — set custom glyph sequences
- `--format-enabled --format '...'` — format output with `$frame`, `$lcpu`, `$rcpu`, or `$$`
- `--stat-path` — read CPU statistics from another file

## Development

```sh
devenv shell
cargo build
cargo test
```

## License

[MIT](LICENSE). Derived from [polycat](https://github.com/zzqmt/polycat).
