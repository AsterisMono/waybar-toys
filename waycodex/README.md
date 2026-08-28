# waycodex

A small Waybar plugin that shows OpenAI Codex's **5-hour** and **weekly**
usage, the number of **banked usage resets**, and the countdown to the next
usage-window reset.

It reads existing OAuth credentials and calls OpenAI's Codex usage API. It
never writes or refreshes credentials and never redeems a banked reset.

## Build

```sh
cargo build --release -p waycodex
```

The binary is written to `target/release/waycodex`.

## Waybar config

```jsonc
"custom/codex": {
    "format": "{}",
    "interval": 60,
    "exec": "waycodex"
}
```

The default output looks like:

```text
5h 31% 1w 54% banked 2 reset 1h 12m
```

Percentages are **used**, not remaining.

## Credentials

`waycodex` checks these sources in order:

1. `WAYCODEX_ACCESS_TOKEN` (and optional `WAYCODEX_ACCOUNT_ID`)
2. `--auth <path>`
3. `$CODEX_HOME/auth.json`
4. `~/.codex/auth.json`
5. `~/.pi/agent/auth.json` (`openai-codex` OAuth entry)

Normally, signing in with the Codex CLI or pi is enough. Because `waycodex`
does not refresh OAuth tokens, run Codex/pi again to renew an expired login.
Credential files are only read locally; the token is sent only to
`https://chatgpt.com/backend-api/wham/...`.

## Options

| Flag | Default | Description |
| --- | --- | --- |
| `-a, --auth <path>` | automatic | OAuth JSON file |
| `-f, --format <template>` | see below | output template |
| `-t, --timeout-ms <ms>` | `5000` | HTTP timeout |
| `-o, --offline <text>` | `codex: off` | fallback text |

Default template:

```text
5h {5h_used}% 1w {1w_used}% banked {banked} reset {next_reset}
```

Available tokens:

- `{5h_used}`, `{1w_used}` — consumed percentage
- `{5h_left}`, `{1w_left}` — remaining percentage
- `{banked}` — available banked usage resets (`?` when unavailable)
- `{next_reset}` — countdown to the closest usage-window reset
- `{5h_reset}`, `{1w_reset}` — per-window reset countdowns

Example:

```sh
waycodex -f 'C {5h_left}%/{1w_left}% left · R{banked} · {next_reset}'
```

Set `WAYCODEX_DEBUG=1` to print connection/authentication errors to stderr.
