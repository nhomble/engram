/// Conversation buffer with sliding window for MITM proxy
///
/// Buffers raw JSON from Claude API traffic to pass to analyzer agent

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Thread-safe conversation buffer with sliding window
/// Stores raw JSON strings from Claude API requests
#[derive(Clone)]
pub struct ConversationBuffer {
    /// Raw JSON request bodies
    requests: Arc<Mutex<VecDeque<String>>>,
    max_size: usize,
}

impl ConversationBuffer {
    /// Create a new buffer with given maximum size
    ///
    /// When the buffer exceeds max_size, oldest requests are dropped (sliding window)
    pub fn new(max_size: usize) -> Self {
        Self {
            requests: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    /// Add a raw JSON request to the buffer
    ///
    /// If buffer is full, removes oldest request first
    pub fn push(&self, json: String) {
        let mut requests = self.requests.lock().unwrap();

        // Remove oldest if at capacity
        if requests.len() >= self.max_size {
            requests.pop_front();
        }

        requests.push_back(json);
    }

    /// Get all requests in the buffer as a Vec
    pub fn get_all(&self) -> Vec<String> {
        let requests = self.requests.lock().unwrap();
        requests.iter().cloned().collect()
    }

    /// Get the N most recent requests
    pub fn get_recent(&self, n: usize) -> Vec<String> {
        let requests = self.requests.lock().unwrap();
        let start = if requests.len() > n { requests.len() - n } else { 0 };
        requests.iter().skip(start).cloned().collect()
    }

    /// Clear all requests from the buffer
    pub fn clear(&self) {
        let mut requests = self.requests.lock().unwrap();
        requests.clear();
    }

    /// Get current buffer size
    pub fn len(&self) -> usize {
        let requests = self.requests.lock().unwrap();
        requests.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        let requests = self.requests.lock().unwrap();
        requests.is_empty()
    }

    /// Get all requests as a single JSON array string for analyzer
    pub fn to_json_array(&self) -> String {
        let requests = self.get_all();
        format!("[{}]", requests.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push_and_get() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(r#"{"messages":[{"role":"user","content":"Hello"}]}"#.to_string());
        buffer.push(r#"{"messages":[{"role":"assistant","content":"Hi there"}]}"#.to_string());

        let requests = buffer.get_all();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].contains("Hello"));
        assert!(requests[1].contains("Hi there"));
    }

    #[test]
    fn test_sliding_window() {
        let buffer = ConversationBuffer::new(3);

        buffer.push(r#"{"msg":"1"}"#.to_string());
        buffer.push(r#"{"msg":"2"}"#.to_string());
        buffer.push(r#"{"msg":"3"}"#.to_string());
        buffer.push(r#"{"msg":"4"}"#.to_string());

        let requests = buffer.get_all();
        assert_eq!(requests.len(), 3);
        assert!(requests[0].contains("2"));
        assert!(requests[1].contains("3"));
        assert!(requests[2].contains("4"));
    }

    #[test]
    fn test_get_recent() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(r#"{"msg":"1"}"#.to_string());
        buffer.push(r#"{"msg":"2"}"#.to_string());
        buffer.push(r#"{"msg":"3"}"#.to_string());
        buffer.push(r#"{"msg":"4"}"#.to_string());

        let recent = buffer.get_recent(2);
        assert_eq!(recent.len(), 2);
        assert!(recent[0].contains("3"));
        assert!(recent[1].contains("4"));
    }

    #[test]
    fn test_clear() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(r#"{"msg":"1"}"#.to_string());
        buffer.push(r#"{"msg":"2"}"#.to_string());

        assert_eq!(buffer.len(), 2);

        buffer.clear();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_json_array() {
        let buffer = ConversationBuffer::new(10);

        buffer.push(r#"{"msg":"1"}"#.to_string());
        buffer.push(r#"{"msg":"2"}"#.to_string());

        let json_array = buffer.to_json_array();
        assert_eq!(json_array, r#"[{"msg":"1"},{"msg":"2"}]"#);
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
                    buffer_clone.push(format!(r#"{{"thread":{},"msg":{}}}"#, i, j));
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
