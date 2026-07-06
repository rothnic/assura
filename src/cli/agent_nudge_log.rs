//! Durable JSONL logging for agent nudge payloads.

use super::AgentNudgeOutput;
use serde::Serialize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const LOG_SCHEMA: &str = "assura.agent-nudge-log.v1";
const DEFAULT_SESSION_ID: &str = "manual";

pub(super) fn maybe_write(project_root: &Path, output: &AgentNudgeOutput) -> Result<(), String> {
    if !logging_enabled() {
        return Ok(());
    }

    let log_dir = log_dir(project_root);
    fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
    let log_path = log_dir.join("nudges.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| error.to_string())?;
    let timestamp_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let session_id =
        env::var("ASSURA_AGENT_SESSION_ID").unwrap_or_else(|_| DEFAULT_SESSION_ID.to_string());
    let record = AgentNudgeLogRecord {
        schema: LOG_SCHEMA,
        session_id,
        timestamp_unix_seconds,
        project_root: path_string(project_root),
        target_agent: output.target_agent,
        event: output.event,
        should_inject: output.summary.should_inject,
        nudge_count: output.summary.nudge_count,
        changed_path_count: output.summary.changed_path_count,
        payload: output,
    };
    let line = serde_json::to_string(&record).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())
}

fn logging_enabled() -> bool {
    if env::var_os("ASSURA_AGENT_LOG_DIR").is_some() {
        return true;
    }
    match env::var("ASSURA_AGENT_LOG") {
        Ok(value) => !matches!(
            value.to_ascii_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        ),
        Err(_) => false,
    }
}

fn log_dir(project_root: &Path) -> PathBuf {
    env::var_os("ASSURA_AGENT_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| project_root.join(".assura").join("agent-sessions"))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Serialize)]
struct AgentNudgeLogRecord<'a> {
    schema: &'static str,
    session_id: String,
    timestamp_unix_seconds: u64,
    project_root: String,
    target_agent: &'static str,
    event: &'static str,
    should_inject: bool,
    nudge_count: usize,
    changed_path_count: usize,
    payload: &'a AgentNudgeOutput,
}
