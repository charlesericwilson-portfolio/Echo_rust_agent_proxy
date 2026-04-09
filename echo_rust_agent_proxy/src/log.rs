// log.rs
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use anyhow::Result;

pub async fn save_chat_log_entry(
    log_dir: &PathBuf,
    user_message: &str,
    assistant_response: &str,
    from: &str,
) -> Result<()> {
    tokio::fs::create_dir_all(log_dir).await?;

    let file_path = log_dir.join("echo_chat.jsonl");

    let mut messages = Vec::new();

    if !user_message.is_empty() {
        messages.push(format!(
            r#"{{"role": "user", "content": "{}"}}"#,
            user_message.trim().replace('"', r#"\""#)
        ));
    }

    if !assistant_response.is_empty() {
        let content = if from.contains("SESSION_START") {
            "=== SESSION START ==="
        } else if from.contains("SESSION_END") {
            "=== SESSION END ==="
        } else if !from.is_empty() && from != "main" && from != "assistant" && from != "user" {
            &format!("Session: {}", from)
        } else {
            assistant_response.trim()
        };

        messages.push(format!(
            r#"{{"role": "assistant", "content": "{}"}}"#,
            content.replace('"', r#"\""#)
        ));
    }

    let messages_str = messages.join(",");

    let log_line = format!(r#"{{"messages": [{}]}}"#, messages_str);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&file_path)
        .map_err(|e| anyhow::anyhow!("Failed to open {}: {}", file_path.display(), e))?;

    writeln!(file, "{}", log_line)
        .map_err(|e| anyhow::anyhow!("Failed to write log: {}", e))?;

    Ok(())
}
