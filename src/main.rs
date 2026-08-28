//! wayherdr: waybar plugin that shows agent statuses from a running herdr
//! server.
//!
//! Intended to be re-invoked by waybar's `custom` module on an interval;
//! each run prints exactly one line.

use std::path::{PathBuf};
use std::time::Duration;

use clap::Parser;

use wayherdr::client;
use wayherdr::status::Counts;

#[derive(Parser, Debug)]
#[command(
    name = "wayherdr",
    version,
    about = "Waybar plugin: agent statuses from a running herdr server"
)]
struct Args {
    /// Path to the herdr API socket
    /// (default: $HERDR_SOCKET_PATH, $HERDR_SESSION, or ~/.config/herdr/herdr.sock).
    #[arg(short, long)]
    socket: Option<PathBuf>,

    /// Output template. Tokens: {working} {blocked} {done} {idle} {unknown} {total}.
    #[arg(short, long, default_value = "{working}w {blocked}b {done}d {idle}i")]
    format: String,

    /// Socket timeout in milliseconds.
    #[arg(short, long, default_value_t = 2000)]
    timeout_ms: u64,

    /// Text printed when the herdr server is unreachable.
    #[arg(short, long, default_value = "herdr: off")]
    offline: String,
}

/// Herdr config directory ($XDG_CONFIG_HOME or $HOME/.config) plus `herdr`.
fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));
    base.map(|p| p.join("herdr"))
}

/// Resolve the socket path using herdr's documented precedence:
/// 1. explicit `--socket`
/// 2. `HERDR_SOCKET_PATH`
/// 3. `HERDR_SESSION=<name>` -> `~/.config/herdr/sessions/<name>/herdr.sock`
/// 4. default session -> `~/.config/herdr/herdr.sock`
fn resolve_socket(args: &Args) -> Option<PathBuf> {
    if let Some(path) = &args.socket {
        return Some(path.clone());
    }
    if let Ok(path) = std::env::var("HERDR_SOCKET_PATH") {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    let config = config_dir()?;
    if let Ok(session) = std::env::var("HERDR_SESSION") {
        if !session.is_empty() {
            return Some(config.join("sessions").join(session).join("herdr.sock"));
        }
    }
    Some(config.join("herdr.sock"))
}

fn main() {
    let args = Args::parse();
    let timeout = Duration::from_millis(args.timeout_ms.max(1));

    let socket = resolve_socket(&args);
    let result = socket.as_deref().map(|s| client::list_agents(s, timeout));

    match result {
        Some(Ok(agents)) => {
            let mut counts = Counts::default();
            for agent in agents {
                counts.add(agent.status);
            }
            println!("{}", counts.render(&args.format));
        }
        _ => {
            // Waybar keeps showing whatever we print; stay quiet-friendly.
            if std::env::var_os("WAYHERDR_DEBUG").is_some() {
                eprintln!("wayherdr: herdr server unreachable (socket={socket:?})");
            }
            println!("{}", args.offline);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wayherdr::status::AgentStatus;

    #[test]
    fn counts_cover_all_states() {
        let mut counts = Counts::default();
        for status in AgentStatus::ALL {
            counts.add(status);
        }
        assert_eq!(counts.total(), 5);
    }
}
