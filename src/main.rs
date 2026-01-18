use serde::Deserialize;
use std::io::{self, BufRead, Write};

#[derive(Deserialize)]
struct StreamMessage {
    #[serde(rename = "type")]
    msg_type: String,
    message: Option<AssistantMessage>,
    result: Option<String>,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { name: String, input: serde_json::Value },
    #[serde(other)]
    Other,
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_tool_use(name: &str, input: &serde_json::Value) -> String {
    match name {
        "Read" => {
            let file_path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("?");
            format!("📖 Read: {}", file_path)
        }
        "Edit" => {
            let file_path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("?");
            format!("✏️  Edit: {}", file_path)
        }
        "Write" => {
            let file_path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("?");
            format!("📝 Write: {}", file_path)
        }
        "Bash" => {
            let command = input.get("command").and_then(|v| v.as_str()).unwrap_or("?");
            format!("💻 Bash: {}", truncate(command, 80))
        }
        "Glob" => {
            let pattern = input.get("pattern").and_then(|v| v.as_str()).unwrap_or("?");
            format!("🔍 Glob: {}", pattern)
        }
        "Grep" => {
            let pattern = input.get("pattern").and_then(|v| v.as_str()).unwrap_or("?");
            format!("🔍 Grep: {}", pattern)
        }
        "TodoWrite" => "📋 TodoWrite".to_string(),
        "Task" => {
            let description = input.get("description").and_then(|v| v.as_str()).unwrap_or("?");
            format!("🤖 Task: {}", description)
        }
        _ => format!("🔧 {}", name),
    }
}

fn process_line(line: &str) -> Option<String> {
    let msg: StreamMessage = serde_json::from_str(line).ok()?;

    match msg.msg_type.as_str() {
        "assistant" => {
            let message = msg.message?;
            let mut output = Vec::new();

            for block in message.content {
                match block {
                    ContentBlock::Text { text } => {
                        if !text.trim().is_empty() {
                            output.push(text);
                        }
                    }
                    ContentBlock::ToolUse { name, input } => {
                        output.push(format_tool_use(&name, &input));
                    }
                    ContentBlock::Other => {}
                }
            }

            if output.is_empty() {
                None
            } else {
                Some(output.join("\n"))
            }
        }
        "result" => {
            let result = msg.result?;
            Some(format!("✅ Done: {}", truncate(&result, 80)))
        }
        _ => None,
    }
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if let Some(output) = process_line(&line) {
            let _ = writeln!(stdout, "{}", output);
            let _ = stdout.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "text", "text": "Hello world"}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("Hello world".to_string()));
    }

    #[test]
    fn test_read_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Read", "input": {"file_path": "/src/main.rs"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("📖 Read: /src/main.rs".to_string()));
    }

    #[test]
    fn test_edit_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Edit", "input": {"file_path": "/src/lib.rs"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("✏️  Edit: /src/lib.rs".to_string()));
    }

    #[test]
    fn test_write_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Write", "input": {"file_path": "/new_file.txt"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("📝 Write: /new_file.txt".to_string()));
    }

    #[test]
    fn test_bash_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Bash", "input": {"command": "ls -la"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("💻 Bash: ls -la".to_string()));
    }

    #[test]
    fn test_bash_tool_truncation() {
        let long_cmd = "a".repeat(100);
        let input = format!(r#"{{"type": "assistant", "message": {{"content": [{{"type": "tool_use", "name": "Bash", "input": {{"command": "{}"}}}}]}}}}"#, long_cmd);
        let result = process_line(&input).unwrap();
        assert!(result.len() < 100);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_glob_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Glob", "input": {"pattern": "**/*.rs"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("🔍 Glob: **/*.rs".to_string()));
    }

    #[test]
    fn test_grep_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Grep", "input": {"pattern": "fn main"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("🔍 Grep: fn main".to_string()));
    }

    #[test]
    fn test_todowrite_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "TodoWrite", "input": {"todos": []}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("📋 TodoWrite".to_string()));
    }

    #[test]
    fn test_task_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Task", "input": {"description": "Search for files"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("🤖 Task: Search for files".to_string()));
    }

    #[test]
    fn test_other_tool() {
        let input = r#"{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "WebFetch", "input": {"url": "https://example.com"}}]}}"#;
        let result = process_line(input);
        assert_eq!(result, Some("🔧 WebFetch".to_string()));
    }

    #[test]
    fn test_result_message() {
        let input = r#"{"type": "result", "result": "Task completed successfully."}"#;
        let result = process_line(input);
        assert_eq!(result, Some("✅ Done: Task completed successfully.".to_string()));
    }

    #[test]
    fn test_result_truncation() {
        let long_result = "a".repeat(100);
        let input = format!(r#"{{"type": "result", "result": "{}"}}"#, long_result);
        let result = process_line(&input).unwrap();
        assert!(result.len() < 100);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_malformed_json() {
        let input = "this is not valid json";
        let result = process_line(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_unknown_message_type() {
        let input = r#"{"type": "unknown", "data": {}}"#;
        let result = process_line(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_truncate_function() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
        assert_eq!(truncate("exactly10!", 10), "exactly10!");
    }
}
