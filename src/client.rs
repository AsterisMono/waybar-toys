//! Minimal client for the herdr server's newline-delimited JSON socket API.
//!
//! Protocol reference: <https://herdr.dev/docs/socket-api/>
//!
//! One request per line, e.g.:
//! `{"id":"wayherdr","method":"agent.list","params":{}}`
//!
//! Success responses carry the same `id` and a `result` object; errors
//! carry an `error` object with `code` and `message`.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::status::AgentStatus;

/// A single agent entry from the `agent.list` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// Agent label (e.g. `pi`, `codex`).
    pub name: String,
    /// Lifecycle status as classified by herdr.
    pub status: AgentStatus,
    pub pane_id: Option<String>,
    pub workspace_id: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    #[serde(default)]
    #[allow(dead_code)]
    id: Option<String>,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<ErrorBody>,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct AgentListResult {
    #[serde(default)]
    agents: Vec<AgentRaw>,
}

#[derive(Debug, Deserialize)]
struct AgentRaw {
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    agent_status: Option<String>,
    #[serde(default)]
    pane_id: Option<String>,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
}

/// Request `agent.list` from the herdr server at `socket` and return the
/// current agents with their lifecycle statuses.
///
/// `timeout` bounds both the connect-phase I/O and the response read so a
/// wedged server cannot hang the bar update.
pub fn list_agents(socket: &Path, timeout: Duration) -> Result<Vec<Agent>, String> {
    let mut stream = UnixStream::connect(socket)
        .map_err(|e| format!("connect {}: {e}", socket.display()))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| format!("set read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| format!("set write timeout: {e}"))?;

    stream
        .write_all(b"{\"id\":\"wayherdr\",\"method\":\"agent.list\",\"params\":{}}\n")
        .map_err(|e| format!("write: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let read = reader.read_line(&mut line).map_err(|e| format!("read: {e}"))?;
    if read == 0 {
        return Err("connection closed before response".into());
    }

    let response: ApiResponse =
        serde_json::from_str(&line).map_err(|e| format!("parse response: {e}"))?;

    if let Some(err) = response.error {
        return Err(format!("{}: {}", err.code, err.message));
    }

    let value = response
        .result
        .ok_or_else(|| "response missing result".to_string())?;

    let list: AgentListResult =
        serde_json::from_value(value).map_err(|e| format!("parse result: {e}"))?;

    Ok(list
        .agents
        .into_iter()
        .map(|a| Agent {
            name: a.agent.unwrap_or_default(),
            status: a
                .agent_status
                .as_deref()
                .map(AgentStatus::from_raw)
                .unwrap_or(AgentStatus::Unknown),
            pane_id: a.pane_id,
            workspace_id: a.workspace_id,
            cwd: a.cwd,
        })
        .collect())
}
