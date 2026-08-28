use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::Parser;
use reqwest::blocking::Client;
use serde_json::Value;
use waycodex::{DisplayUsage, ResetCreditsResponse, UsageResponse, unix_now};

const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";
const RESET_CREDITS_URL: &str = "https://chatgpt.com/backend-api/wham/rate-limit-reset-credits";

#[derive(Parser, Debug)]
#[command(
    name = "waycodex",
    version,
    about = "Waybar plugin: OpenAI Codex usage and banked resets"
)]
struct Args {
    /// OAuth auth.json path. Defaults to $CODEX_HOME/auth.json, ~/.codex/auth.json,
    /// then ~/.pi/agent/auth.json.
    #[arg(short, long)]
    auth: Option<PathBuf>,

    /// Output template.
    #[arg(
        short,
        long,
        default_value = "5h {5h_used}% 1w {1w_used}% banked {banked} reset {next_reset}"
    )]
    format: String,

    /// HTTP timeout in milliseconds.
    #[arg(short, long, default_value_t = 5000)]
    timeout_ms: u64,

    /// Text printed when credentials or usage are unavailable.
    #[arg(short, long, default_value = "codex: off")]
    offline: String,
}

#[derive(Debug)]
struct Credentials {
    access_token: String,
    account_id: Option<String>,
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            if std::env::var_os("WAYCODEX_DEBUG").is_some() {
                eprintln!("waycodex: {error}");
            }
            println!("{}", args.offline);
        }
    }
}

fn run(args: &Args) -> Result<String, String> {
    let credentials = load_credentials(args.auth.as_deref())?;
    let client = Client::builder()
        .timeout(Duration::from_millis(args.timeout_ms.max(1)))
        .user_agent(concat!("waycodex/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("could not create HTTP client: {error}"))?;

    let usage: UsageResponse = request(&client, USAGE_URL, &credentials)?;
    // This separate endpoint is the authoritative reset inventory. Older
    // servers may include only a summary in the usage response, so failure is
    // non-fatal and the embedded value is used as a fallback.
    let banked = request::<ResetCreditsResponse>(&client, RESET_CREDITS_URL, &credentials)
        .ok()
        .map(|response| response.available_count);
    let now = unix_now();
    let display = DisplayUsage::from_response(&usage, banked, now)
        .ok_or_else(|| "the Codex API did not return both usage windows".to_string())?;
    Ok(display.render(&args.format, now))
}

fn request<T: serde::de::DeserializeOwned>(
    client: &Client,
    url: &str,
    credentials: &Credentials,
) -> Result<T, String> {
    let mut request = client
        .get(url)
        .bearer_auth(&credentials.access_token)
        .header("Accept", "application/json")
        .header("OpenAI-Beta", "codex-1")
        .header("originator", "Codex Desktop");
    if let Some(account_id) = &credentials.account_id {
        request = request.header("ChatGPT-Account-ID", account_id);
    }
    let response = request
        .send()
        .map_err(|error| format!("request failed: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Codex API returned HTTP {status}"));
    }
    response
        .json()
        .map_err(|error| format!("invalid Codex API response: {error}"))
}

fn load_credentials(explicit: Option<&Path>) -> Result<Credentials, String> {
    if let Ok(access_token) = std::env::var("WAYCODEX_ACCESS_TOKEN")
        && !access_token.trim().is_empty()
    {
        return Ok(Credentials {
            access_token,
            account_id: std::env::var("WAYCODEX_ACCOUNT_ID").ok(),
        });
    }

    let paths = if let Some(path) = explicit {
        vec![path.to_path_buf()]
    } else {
        default_auth_paths()
    };
    for path in &paths {
        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&contents) else {
            continue;
        };
        if let Some(credentials) = credentials_from_json(&value) {
            return Ok(credentials);
        }
    }
    Err(format!(
        "no OpenAI Codex OAuth credentials found (checked {})",
        paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn default_auth_paths() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let codex = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|path| path.join(".codex")))
        .map(|path| path.join("auth.json"));
    let pi = home.map(|path| path.join(".pi/agent/auth.json"));
    [codex, pi].into_iter().flatten().collect()
}

fn credentials_from_json(root: &Value) -> Option<Credentials> {
    // Native Codex: {"tokens":{"access_token":"...","account_id":"..."}}
    // Pi: {"openai-codex":{"type":"oauth","access":"..."}}
    let candidate = root
        .get("tokens")
        .or_else(|| root.get("openai-codex"))
        .unwrap_or(root);
    let access_token = candidate
        .get("access_token")
        .or_else(|| candidate.get("access"))
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())?
        .to_string();
    let account_id = candidate
        .get("account_id")
        .or_else(|| root.get("account_id"))
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_string);
    Some(Credentials {
        access_token,
        account_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_native_codex_credentials() {
        let value = serde_json::json!({
            "tokens": {"access_token": "token", "account_id": "account"}
        });
        let credentials = credentials_from_json(&value).unwrap();
        assert_eq!(credentials.access_token, "token");
        assert_eq!(credentials.account_id.as_deref(), Some("account"));
    }

    #[test]
    fn reads_pi_credentials() {
        let value = serde_json::json!({
            "openai-codex": {"type": "oauth", "access": "token"}
        });
        let credentials = credentials_from_json(&value).unwrap();
        assert_eq!(credentials.access_token, "token");
        assert_eq!(credentials.account_id, None);
    }
}
