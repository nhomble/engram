/// MITM integration for automatic memory capture
///
/// Integrates with mitmproxy to capture Claude API traffic and extract memories

pub mod analyzer;
pub mod buffer;
pub mod client;

use buffer::ConversationBuffer;
use client::MitmproxyClient;
use crate::engram::Engram;
use std::error::Error;
use tokio::time::{sleep, Duration};

/// Run the mitmproxy watcher in a loop
///
/// Polls mitmproxy's /flows endpoint, filters for Claude API traffic,
/// buffers requests, and periodically runs the analyzer to extract memories.
pub async fn run_watcher(
    url: &str,
    interval_secs: u64,
    batch_size: usize,
    engram: Engram,
) -> Result<(), Box<dyn Error>> {
    println!("🔍 Starting mitmproxy watcher");
    println!("   URL: {}", url);
    println!("   Poll interval: {}s", interval_secs);
    println!("   Batch size: {}", batch_size);
    println!();

    let client = MitmproxyClient::new(url);
    let buffer = ConversationBuffer::new(50); // Keep last 50 requests in memory
    let mut last_flow_id: Option<String> = None;
    let mut requests_since_analysis = 0;

    loop {
        // Poll for new flows
        let flows = match last_flow_id {
            Some(ref id) => client.get_flows_since(id).await?,
            None => client.get_flows().await?,
        };

        if !flows.is_empty() {
            println!("📥 Fetched {} new flows", flows.len());

            // Update last flow ID
            if let Some(last_flow) = flows.last() {
                last_flow_id = Some(last_flow.id.clone());
            }

            // Filter for Claude API flows
            let claude_flows = MitmproxyClient::filter_claude_flows(&flows);

            if !claude_flows.is_empty() {
                println!("   → Found {} Claude API requests", claude_flows.len());

                // Extract request bodies and add to buffer
                let bodies = MitmproxyClient::extract_request_bodies(&claude_flows);
                for body in bodies {
                    println!("   → Captured {} bytes of JSON", body.len());
                    buffer.push(body);
                    requests_since_analysis += 1;
                }

                // Run analyzer if we've accumulated enough requests
                if requests_since_analysis >= batch_size {
                    println!("\n🧠 Running analyzer on {} requests...", buffer.len());

                    match analyzer::analyze_conversation(&buffer) {
                        Ok(result) => {
                            println!("   → Extracted {} memories", result.memories.len());

                            if !result.memories.is_empty() {
                                // Store memories in engram
                                for memory in &result.memories {
                                    match engram.add_memory(memory) {
                                        Ok(id) => println!("   ✓ Stored: {} ({})", memory, id),
                                        Err(e) => eprintln!("   ✗ Failed to store memory: {}", e),
                                    }
                                }
                            }

                            // Clear buffer and reset counter
                            buffer.clear();
                            requests_since_analysis = 0;
                        }
                        Err(e) => {
                            eprintln!("   ✗ Analyzer failed: {}", e);
                        }
                    }

                    println!();
                }
            }
        }

        // Sleep before next poll
        sleep(Duration::from_secs(interval_secs)).await;
    }
}
