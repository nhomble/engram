/// Conversation buffer with sliding window for MITM proxy
///
/// Buffers messages from Claude API traffic to pass to analyzer agent

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A message in the conversation (request or response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Timestamp when message was captured
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Message role (user, assistant, system)
    pub role: String,

    /// Message content
    pub content: String,
}

/// Thread-safe conversation buffer with sliding window
#[derive(Clone)]
pub struct ConversationBuffer {
    messages: Arc<Mutex<VecDeque<Message>>>,
    max_size: usize,
}

impl ConversationBuffer {
    /// Create a new buffer with given maximum size
    ///
    /// When the buffer exceeds max_size, oldest messages are dropped (sliding window)
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add a message to the buffer
    ///
    /// If buffer is full, removes oldest message first
    pub fn push(&self, message: Message) {
        let mut messages = self.messages.lock().unwrap();

        // Remove oldest if at capacity
        if messages.len() >= self.max_size {
            messages.pop_front();
        }

        messages.push_back(message);
    }

    /// Get all messages in the buffer as a Vec
    pub fn get_all(&self) -> Vec<Message> {
        let messages = self.messages.lock().unwrap();
        messages.iter().cloned().collect()
    }

    /// Get messages since a given timestamp
    pub fn get_since(&self, since: chrono::DateTime<chrono::Utc>) -> Vec<Message> {
        let messages = self.messages.lock().unwrap();
        messages
            .iter()
            .filter(|m| m.timestamp > since)
            .cloned()
            .collect()
    }

    /// Get the N most recent messages
    pub fn get_recent(&self, n: usize) -> Vec<Message> {
        let messages = self.messages.lock().unwrap();
        let start = if messages.len() > n { messages.len() - n } else { 0 };
        messages.iter().skip(start).cloned().collect()
    }

    /// Clear all messages from the buffer
    pub fn clear(&self) {
        let mut messages = self.messages.lock().unwrap();
        messages.clear();
    }

    /// Get current buffer size
    pub fn len(&self) -> usize {
        let messages = self.messages.lock().unwrap();
        messages.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        let messages = self.messages.lock().unwrap();
        messages.is_empty()
    }

    /// Serialize messages to JSON for analyzer agent
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let messages = self.get_all();
        serde_json::to_string_pretty(&messages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_message(role: &str, content: &str, offset_secs: i64) -> Message {
        Message {
            timestamp: chrono::Utc::now() + chrono::Duration::seconds(offset_secs),
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn test_basic_push_and_get() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(create_test_message("user", "Hello", 0));
        buffer.push(create_test_message("assistant", "Hi there", 1));

        let messages = buffer.get_all();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi there");
    }

    #[test]
    fn test_sliding_window() {
        let buffer = ConversationBuffer::new(3);

        buffer.push(create_test_message("user", "Message 1", 0));
        buffer.push(create_test_message("user", "Message 2", 1));
        buffer.push(create_test_message("user", "Message 3", 2));
        buffer.push(create_test_message("user", "Message 4", 3));

        let messages = buffer.get_all();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, "Message 2");
        assert_eq!(messages[1].content, "Message 3");
        assert_eq!(messages[2].content, "Message 4");
    }

    #[test]
    fn test_get_recent() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(create_test_message("user", "Message 1", 0));
        buffer.push(create_test_message("user", "Message 2", 1));
        buffer.push(create_test_message("user", "Message 3", 2));
        buffer.push(create_test_message("user", "Message 4", 3));

        let recent = buffer.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].content, "Message 3");
        assert_eq!(recent[1].content, "Message 4");
    }

    #[test]
    fn test_get_since() {
        let now = chrono::Utc::now();
        let buffer = ConversationBuffer::new(10);

        buffer.push(Message {
            timestamp: now - chrono::Duration::seconds(10),
            role: "user".to_string(),
            content: "Old message".to_string(),
        });
        buffer.push(Message {
            timestamp: now + chrono::Duration::seconds(1),
            role: "user".to_string(),
            content: "New message".to_string(),
        });

        let since = buffer.get_since(now);
        assert_eq!(since.len(), 1);
        assert_eq!(since[0].content, "New message");
    }

    #[test]
    fn test_clear() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(create_test_message("user", "Message 1", 0));
        buffer.push(create_test_message("user", "Message 2", 1));

        assert_eq!(buffer.len(), 2);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_json_serialization() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(create_test_message("user", "Hello", 0));
        buffer.push(create_test_message("assistant", "Hi", 1));

        let json = buffer.to_json().unwrap();
        assert!(json.contains("Hello"));
        assert!(json.contains("Hi"));
        assert!(json.contains("user"));
        assert!(json.contains("assistant"));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let buffer = ConversationBuffer::new(100);
        let mut handles = vec![];

        for i in 0..10 {
            let buffer_clone = buffer.clone();
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    buffer_clone.push(create_test_message(
                        "user",
                        &format!("Thread {} Message {}", i, j),
                        0,
                    ));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(buffer.len(), 100);
    }
}
