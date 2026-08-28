//! Integration test: speak the real socket protocol against a local mock
//! herdr server.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::thread;
use std::time::Duration;

use wayherdr::client;
use wayherdr::status::AgentStatus;

fn unique_socket() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "wayherdr-test-{}-{}.sock",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ))
}

/// Run `client::list_agents` against a mock server that answers the
/// `agent.list` request with the given canned JSON `result` body.
fn run_with_mock(result_body: &str) -> Result<Vec<client::Agent>, String> {
    let socket = unique_socket();
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).expect("bind mock socket");
    // Move the listener into the mock thread so it is dropped there.
    let mock_socket = socket.clone();
    let result_body = result_body.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut line = String::new();
        BufReader::new(stream.try_clone().unwrap()).read_line(&mut line).unwrap();
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["method"], "agent.list");
        let response = format!(
            "{{\"id\":\"wayherdr\",\"result\":{}}}",
            result_body
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(b"\n").unwrap();
        drop(stream);
    });

    let outcome = client::list_agents(&mock_socket, Duration::from_secs(5));
    handle.join().expect("mock server thread panicked");
    let _ = std::fs::remove_file(&socket);
    outcome
}

#[test]
fn lists_agents_with_statuses() {
    let agents = run_with_mock(
        r#"{"agents":[
             {"agent":"pi","agent_status":"working","pane_id":"w1:p1","cwd":"/a"},
             {"agent":"codex","agent_status":"blocked","pane_id":"w1:p2"},
             {"agent":"claude","agent_status":"idle","pane_id":"w2:p1"},
             {"agent":"gemini","agent_status":"done","pane_id":"w2:p2"},
             {"agent":"cursor","agent_status":"weird-future-state","pane_id":"w3:p1"}
           ],"type":"agent_list"}"#,
    )
    .expect("list_agents should succeed");

    assert_eq!(agents.len(), 5);
    assert_eq!(agents[0].name, "pi");
    assert_eq!(agents[0].status, AgentStatus::Working);
    assert_eq!(agents[0].pane_id.as_deref(), Some("w1:p1"));
    assert_eq!(agents[0].cwd.as_deref(), Some("/a"));
    assert_eq!(agents[1].status, AgentStatus::Blocked);
    assert_eq!(agents[2].status, AgentStatus::Idle);
    assert_eq!(agents[3].status, AgentStatus::Done);
    // Unrecognized statuses map to Unknown rather than failing.
    assert_eq!(agents[4].status, AgentStatus::Unknown);
}

#[test]
fn empty_agent_list() {
    let agents = run_with_mock(r#"{"agents":[],"type":"agent_list"}"#)
        .expect("list_agents should succeed");
    assert!(agents.is_empty());
}

#[test]
fn server_error_surfaces_as_err() {
    let err = run_with_mock(r#"null"#)
        .expect_err("missing result should be an error");
    assert!(err.contains("result"), "unexpected error: {err}");
}

#[test]
fn missing_socket_is_an_error() {
    let path = std::env::temp_dir().join("wayherdr-does-not-exist.sock");
    let _ = std::fs::remove_file(&path);
    let err = client::list_agents(&path, Duration::from_secs(1))
        .expect_err("connecting to a missing socket should fail");
    assert!(err.contains("connect"), "unexpected error: {err}");
}

#[test]
fn silent_close_is_an_error() {
    // Server accepts, says nothing, closes.
    let socket = unique_socket();
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).expect("bind");
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("accept");
        drop(stream); // close immediately, no response
    });
    let err = client::list_agents(&socket, Duration::from_secs(2))
        .expect_err("closed connection should be an error");
    handle.join().unwrap();
    let _ = std::fs::remove_file(&socket);
    assert!(
        err.contains("closed") || err.contains("read") || err.contains("timed out"),
        "unexpected error: {err}"
    );
}
