// commands.rs
pub fn extract_session_command(response_text: &str) -> Option<(String, String)> {
    for line in response_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SESSION:") {
            let rest = rest.trim();
            if let Some((session_name, command)) = rest.split_once(' ') {
                return Some((
                    session_name.trim().to_string(),
                    command.trim().to_string(),
                ));
            } else if !rest.is_empty() {
                return Some((rest.to_string(), String::new()));
            }
        }
    }
    None
}

pub fn extract_run_command(response_text: &str) -> Option<(String, String)> {
    for line in response_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("TOOL_NAME: RUN") {
            let rest = rest.trim();
            if let Some(session_name) = rest.split_whitespace().next() {
                let command = rest.replacen(session_name, "", 1).trim().to_string();
                return Some((session_name.to_string(), format!("run {}", command)));
            }
        }
    }
    None
}

pub fn extract_end_command(response_text: &str) -> Option<String> {
    for line in response_text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("END_SESSION:") {
            return Some(name.trim().to_string());
        }
    }
    None
}

pub fn extract_command(response_text: &str) -> Option<String> {
    for line in response_text.lines() {
        let line = line.trim();
        if let Some(cmd) = line.strip_prefix("COMMAND:") {
            return Some(cmd.trim().to_string());
        }
    }
    None
}
