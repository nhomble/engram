/// Mitmproxy HTTP API client
///
/// Polls mitmproxy's /flows endpoint to fetch captured traffic

use serde::{Deserialize, Serialize};
use std::error::Error;

/// Mitmproxy flow object (simplified, only fields we need)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flow {
    pub id: String,
    pub r#type: String,
    pub request: Option<Request>,
    pub response: Option<Response>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {
    pub method: String,
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub content: Option<String>,
}

/// Mitmproxy API client
pub struct MitmproxyClient {
    base_url: String,
    client: reqwest::Client,
}

impl MitmproxyClient {
    /// Create a new mitmproxy client
    ///
    /// # Arguments
    /// * `base_url` - Base URL of mitmproxy web interface (e.g., "http://localhost:8081")
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Fetch all flows from mitmproxy
    pub async fn get_flows(&self) -> Result<Vec<Flow>, Box<dyn Error>> {
        let url = format!("{}/flows", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch flows: {}", response.status()).into());
        }

        let flows: Vec<Flow> = response.json().await?;
        Ok(flows)
    }

    /// Fetch flows since a given ID (for incremental polling)
    ///
    /// Returns only flows that were added after the given flow ID
    pub async fn get_flows_since(&self, since_id: &str) -> Result<Vec<Flow>, Box<dyn Error>> {
        let all_flows = self.get_flows().await?;

        // Find the index of the "since" flow
        let since_index = all_flows
            .iter()
            .position(|f| f.id == since_id);

        match since_index {
            Some(idx) => {
                // Return flows after this index
                Ok(all_flows.into_iter().skip(idx + 1).collect())
            }
            None => {
                // "since" flow not found, return all flows (it may have been cleared)
                Ok(all_flows)
            }
        }
    }

    /// Filter flows for Claude API traffic
    ///
    /// Returns only flows where the host contains "anthropic.com" or "claude.ai"
    pub fn filter_claude_flows(flows: &[Flow]) -> Vec<Flow> {
        flows
            .iter()
            .filter(|flow| {
                flow.request
                    .as_ref()
                    .map(|req| {
                        req.host.contains("anthropic.com") || req.host.contains("claude.ai")
                    })
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Extract request bodies from flows as JSON strings
    ///
    /// Returns the raw JSON request body from each flow
    pub fn extract_request_bodies(flows: &[Flow]) -> Vec<String> {
        flows
            .iter()
            .filter_map(|flow| {
                flow.request
                    .as_ref()
                    .and_then(|req| req.content.clone())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_claude_flows() {
        let flows = vec![
            Flow {
                id: "1".to_string(),
                r#type: "http".to_string(),
                request: Some(Request {
                    method: "POST".to_string(),
                    scheme: "https".to_string(),
                    host: "api.anthropic.com".to_string(),
                    port: 443,
                    path: "/v1/messages".to_string(),
                    headers: vec![],
                    content: Some(r#"{"model":"claude-3-sonnet"}"#.to_string()),
                }),
                response: None,
            },
            Flow {
                id: "2".to_string(),
                r#type: "http".to_string(),
                request: Some(Request {
                    method: "GET".to_string(),
                    scheme: "https".to_string(),
                    host: "example.com".to_string(),
                    port: 443,
                    path: "/".to_string(),
                    headers: vec![],
                    content: None,
                }),
                response: None,
            },
        ];

        let claude_flows = MitmproxyClient::filter_claude_flows(&flows);
        assert_eq!(claude_flows.len(), 1);
        assert_eq!(claude_flows[0].id, "1");
    }

    #[test]
    fn test_extract_request_bodies() {
        let flows = vec![
            Flow {
                id: "1".to_string(),
                r#type: "http".to_string(),
                request: Some(Request {
                    method: "POST".to_string(),
                    scheme: "https".to_string(),
                    host: "api.anthropic.com".to_string(),
                    port: 443,
                    path: "/v1/messages".to_string(),
                    headers: vec![],
                    content: Some(r#"{"model":"claude-3-sonnet"}"#.to_string()),
                }),
                response: None,
            },
            Flow {
                id: "2".to_string(),
                r#type: "http".to_string(),
                request: Some(Request {
                    method: "POST".to_string(),
                    scheme: "https".to_string(),
                    host: "api.anthropic.com".to_string(),
                    port: 443,
                    path: "/v1/messages".to_string(),
                    headers: vec![],
                    content: None,  // No content
                }),
                response: None,
            },
        ];

        let bodies = MitmproxyClient::extract_request_bodies(&flows);
        assert_eq!(bodies.len(), 1);
        assert!(bodies[0].contains("claude-3-sonnet"));
    }
}
