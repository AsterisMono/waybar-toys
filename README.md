# wayherdr

Waybar plugin (Rust) that shows the statuses of coding agents running in a
[herdr](https://herdr.dev) server: how many are **working**, **blocked**,
**done**, and **idle**.

It speaks herdr's newline-delimited JSON socket API directly (no CLI
shelling out), so each bar refresh is a single small request.

## Build

```sh
devenv shell          # enter the environment (rust + cargo)
cargo build --release
```

Or from the outer shell:

```sh
devenv shell -- cargo build --release
```

The release binary lands in `target/release/wayherdr`.

## Waybar config

```jsonc
"custom/herdr": {
    "format": "{}",
    "interval": 3,          // refresh every 3 s
    "exec": "wayherdr"
},
```

Put `custom/herdr` in your `modules-left` / `modules-right` list.

## Socket resolution

The herdr server socket is resolved in this order:

1. `--socket <path>`
2. `$HERDR_SOCKET_PATH`
3. `$HERDR_SESSION=<name>` → `~/.config/herdr/sessions/<name>/herdr.sock`
4. default → `~/.config/herdr/herdr.sock`

## Options

| Flag | Default | Description |
| ---- | ------- | ----------- |
| `-s, --socket <path>` | auto (see above) | herdr API socket path |
| `-f, --format <template>` | `{working}w {blocked}b {done}d {idle}i` | output template |
| `-t, --timeout-ms <ms>` | `2000` | socket timeout |
| `-o, --offline <text>` | `herdr: off` | text when the server is unreachable |

Template tokens: `{working}` `{blocked}` `{done}` `{idle}` `{unknown}`
`{total}`.

Examples:

```sh
wayherdr
# => 2w 1b 1d 3i

wayherdr -f "⚒ {working} ⛔ {blocked} ✓ {done} ∙ {idle}"
# => ⚒ 2 ⛔ 1 ✓ 1 ∙ 3

wayherdr -f "{total} agents: {working} working, {blocked} blocked, {done} done, {idle} idle"
```

Set `WAYHERDR_DEBUG=1` to print why the server was unreachable to stderr.

## Tests

```sh
devenv shell -- cargo test
```

The integration tests spin up a mock herdr server on a local unix socket and
exercise the real wire protocol.
