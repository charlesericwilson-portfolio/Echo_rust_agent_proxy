// sessions.rs
use anyhow::{bail, Result};
use std::path::PathBuf;
use std::process::Command;
use tokio::time::{sleep, Duration};

pub use crate::ACTIVE_SESSIONS;

/// Start or reuse a tmux session
/// Start or reuse a tmux session
/// Start or reuse a tmux session with a persistent shell
pub async fn start_or_reuse_session(_home: PathBuf, name: &str, _initial_command: &str) -> Result<()> {
    let exists = Command::new("tmux")
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if exists {
        println!("Echo: Reusing existing tmux session '{}'.", name);
        return Ok(());
    }

    println!("Echo: Creating new tmux session '{}'", name);

    let status = Command::new("tmux")
        .args(["new-session", "-d", "-s", name, "bash -i"])
        .status()?;

    if !status.success() {
        bail!("Failed to create tmux session '{}'", name);
    }

    let mut sessions = ACTIVE_SESSIONS.lock().await;
    sessions.insert(name.to_string(), (String::new(), String::new()));

    sleep(Duration::from_millis(600)).await;

    Ok(())
}

/// Send command to tmux session and return ONLY the new output (cleaned)
/// Send a command to tmux session and return ONLY the new output
/// Send command to tmux session and return only the new output
/// Send command and capture ONLY the fresh output after the marker
/// Send command and capture ONLY the fresh output using markers
pub async fn execute_in_session(_home: PathBuf, session_name: &str, command: String) -> Result<String> {
    let sessions = ACTIVE_SESSIONS.lock().await;
    if !sessions.contains_key(session_name) {
        bail!("Session '{}' not active.", session_name);
    }
    drop(sessions);

    // Unique start and end markers
    let start_marker = format!("===ECHO_START_{}===", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let end_marker = format!("===ECHO_END_{}===", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() + 1);

    // Send: start_marker + command + end_marker
    let input = format!("echo '{}'\n{}\necho '{}'", start_marker, command, end_marker);

    let _ = Command::new("tmux")
        .args(["send-keys", "-t", session_name, &input, "Enter"])
        .status();

    // Wait longer for long-running commands
    sleep(Duration::from_millis(1500)).await;

    // Capture recent output
    let output = Command::new("tmux")
        .args(["capture-pane", "-p", "-S", "-150", "-t", session_name])
        .output()?;

    let raw = String::from_utf8_lossy(&output.stdout).to_string();

    // Extract text BETWEEN start_marker and end_marker
    if let Some(start_pos) = raw.find(&start_marker) {
        let after_start = &raw[start_pos + start_marker.len()..];
        if let Some(end_pos) = after_start.find(&end_marker) {
            let fresh_output = &after_start[0..end_pos];

            let cleaned: String = fresh_output
                .lines()
                .filter(|line| {
                    let l = line.trim();
                    !l.is_empty()
                        && !l.ends_with('$')
                        && !l.starts_with("eric@")
                        && !l.contains(start_marker.trim())
                        && !l.contains(end_marker.trim())
                        && !l.contains("Echo:")
                })
                .collect::<Vec<_>>()
                .join("\n");

            let final_output = cleaned.trim();
            if !final_output.is_empty() {
                return Ok(final_output.to_string());
            }
        }
    }

    // Fallback
    Ok(format!("Command '{}' completed in session '{}'.", command, session_name))
}

/// End / kill a tmux session gracefully
pub async fn end_session(_home_dir: PathBuf, name: &str) -> Result<()> {
    let mut sessions = ACTIVE_SESSIONS.lock().await;

    if sessions.remove(name).is_some() {
        println!("Echo: Terminating tmux session '{}'.", name);

        // Send Ctrl+C first for graceful shutdown
        let _ = Command::new("tmux").args(["send-keys", "-t", name, "C-c"]).status();
        sleep(Duration::from_millis(600)).await;

        // Kill the session
        let _ = Command::new("tmux").args(["kill-session", "-t", name]).status();

        Ok(())
    } else {
        bail!("Session '{}' not active.", name);
    }
}

/// Clean up all sessions on exit
pub async fn clean_up_sessions() -> Result<()> {
    let mut sessions = ACTIVE_SESSIONS.lock().await;
    let names: Vec<String> = sessions.keys().cloned().collect();

    for name in names {
        println!("Echo: Cleaning up tmux session '{}'.", name);
        let _ = Command::new("tmux").args(["kill-session", "-t", &name]).status();
    }

    sessions.clear();
    Ok(())
}
