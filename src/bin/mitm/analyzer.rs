/// Memory analyzer using Claude Code headless mode
///
/// Spawns Claude CLI to analyze conversations and extract memories

use super::buffer::ConversationBuffer;
use std::io::Write;
use std::process::{Command, Stdio};

const ANALYZER_PROMPT_TEMPLATE: &str = r#"You are a memory extraction agent. Review this conversation between a user and Claude assistant.

Your job: identify learnings worth storing in engram (memory database).

Store when you see:
- User corrections or stated preferences
- Architecture decisions or technical patterns discovered
- Non-obvious workflows or gotchas learned
- Error solutions with context
- Configuration patterns

For each memory, output ONLY:
engram add "concise, self-contained fact"

Output only engram commands, one per line. No explanations.

Conversation to analyze:
{conversation_json}
"#;

/// Result of analyzing a conversation
#[derive(Debug)]
pub struct AnalysisResult {
    /// Memory contents that were extracted
    pub memories: Vec<String>,

    /// Raw output from analyzer
    pub raw_output: String,

    /// Analyzer stderr (for debugging)
    pub stderr: String,
}

/// Analyze a conversation and extract memories using Claude Code headless mode
///
/// Spawns `claude --model haiku` and passes the conversation for analysis
pub fn analyze_conversation(buffer: &ConversationBuffer) -> Result<AnalysisResult, Box<dyn std::error::Error>> {
    let messages = buffer.get_all();

    if messages.is_empty() {
        return Ok(AnalysisResult {
            memories: vec![],
            raw_output: String::new(),
            stderr: String::new(),
        });
    }

    // Serialize conversation to JSON
    let conversation_json = serde_json::to_string_pretty(&messages)?;

    // Build analyzer prompt
    let prompt = ANALYZER_PROMPT_TEMPLATE.replace("{conversation_json}", &conversation_json);

    // Spawn Claude CLI in headless mode
    let mut child = Command::new("claude")
        .arg("--model")
        .arg("haiku")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write prompt to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes())?;
    }

    // Wait for completion and capture output
    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Parse engram commands from output
    let memories = parse_engram_add_commands(&stdout);

    Ok(AnalysisResult {
        memories,
        raw_output: stdout,
        stderr,
    })
}

/// Execute extracted memories by adding them to engram
///
/// Returns the IDs of successfully added memories
pub fn execute_memories(memories: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut ids = vec![];

    for memory in memories {
        // Execute: engram add "content"
        let output = Command::new("engram")
            .arg("add")
            .arg(memory)
            .output()?;

        if output.status.success() {
            // Parse ID from stdout (format: "Added memory: <id>")
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(id) = extract_memory_id(&stdout) {
                ids.push(id);
            }
        }
    }

    Ok(ids)
}

/// Parse "engram add" commands from analyzer output
///
/// Extracts the content between quotes in lines like: engram add "content"
fn parse_engram_add_commands(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();

            // Look for: engram add "content"
            if trimmed.starts_with("engram add") {
                // Extract content between quotes
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        let content = &trimmed[start + 1..start + 1 + end];
                        return Some(content.to_string());
                    }
                }
            }

            None
        })
        .collect()
}

/// Extract memory ID from engram add output
///
/// Parses: "Added memory: engram-xxx"
fn extract_memory_id(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| {
            if line.contains("Added memory:") {
                line.split_whitespace().last().map(String::from)
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_command() {
        let output = r#"engram add "User prefers concise responses""#;
        let commands = parse_engram_add_commands(output);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "User prefers concise responses");
    }

    #[test]
    fn test_parse_multiple_commands() {
        let output = r#"
engram add "Always run tests before commit"
engram add "OAuth requires HTTPS in production"
engram add "Use Divio documentation structure"
        "#;
        let commands = parse_engram_add_commands(output);
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "Always run tests before commit");
        assert_eq!(commands[1], "OAuth requires HTTPS in production");
        assert_eq!(commands[2], "Use Divio documentation structure");
    }

    #[test]
    fn test_parse_with_noise() {
        let output = r#"
Let me analyze this conversation...

engram add "User likes Rust for system tools"

I also noticed...
engram add "Project uses cargo for builds"

Done.
        "#;
        let commands = parse_engram_add_commands(output);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], "User likes Rust for system tools");
        assert_eq!(commands[1], "Project uses cargo for builds");
    }

    #[test]
    fn test_parse_no_commands() {
        let output = "No memories found in this conversation.";
        let commands = parse_engram_add_commands(output);
        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn test_extract_memory_id() {
        let output = "Added memory: engram-abc123";
        let id = extract_memory_id(output);
        assert_eq!(id, Some("engram-abc123".to_string()));
    }

    #[test]
    fn test_extract_memory_id_multiline() {
        let output = r#"
Processing...
Added memory: engram-xyz789
Done.
        "#;
        let id = extract_memory_id(output);
        assert_eq!(id, Some("engram-xyz789".to_string()));
    }
}
