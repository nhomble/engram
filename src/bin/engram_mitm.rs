/// Engram MITM - Network proxy for capturing Claude memories
///
/// Intercepts Claude API traffic and uses a dedicated analyzer agent
/// to extract memories autonomously

mod mitm;

use mitm::buffer::ConversationBuffer;
use mitm::cert::CertificateAuthority;
use mitm::proxy::{ProxyConfig, run_proxy};
use mitm::analyzer;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

/// Configuration from environment variables
struct Config {
    /// Port to listen on (default: 8080)
    port: u16,

    /// Messages before analysis (default: 10)
    batch_size: usize,

    /// Seconds before analysis (default: 300)
    interval_secs: u64,

    /// Max messages in buffer (default: 50)
    buffer_size: usize,
}

impl Config {
    fn from_env() -> Self {
        Self {
            port: std::env::var("ENGRAM_MITM_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),

            batch_size: std::env::var("ENGRAM_MITM_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),

            interval_secs: std::env::var("ENGRAM_MITM_INTERVAL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),

            buffer_size: std::env::var("ENGRAM_MITM_BUFFER_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(50),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Engram MITM - Memory extraction proxy\n");

    // Load configuration
    let config = Config::from_env();

    println!("Configuration:");
    println!("  Port: {}", config.port);
    println!("  Batch size: {} messages", config.batch_size);
    println!("  Interval: {}s", config.interval_secs);
    println!("  Buffer size: {} messages\n", config.buffer_size);

    // Load or create CA certificate
    let ca = CertificateAuthority::load_or_create()?;

    // Create conversation buffer
    let buffer = ConversationBuffer::new(config.buffer_size);

    // Start analyzer task
    let analyzer_buffer = buffer.clone();
    let batch_size = config.batch_size;
    let interval_duration = Duration::from_secs(config.interval_secs);

    tokio::spawn(async move {
        let mut ticker = interval(interval_duration);
        let mut last_analyzed = 0;

        loop {
            ticker.tick().await;

            let current_size = analyzer_buffer.len();

            // Analyze if we have enough new messages OR interval elapsed
            if current_size >= last_analyzed + batch_size || (current_size > 0 && current_size > last_analyzed) {
                println!("\n=== Analyzing conversation ({} messages) ===", current_size);

                match analyzer::analyze_conversation(&analyzer_buffer) {
                    Ok(result) => {
                        if result.memories.is_empty() {
                            println!("No new memories extracted");
                        } else {
                            println!("Extracted {} memories:", result.memories.len());
                            for (i, memory) in result.memories.iter().enumerate() {
                                println!("  {}. {}", i + 1, memory);
                            }

                            // Execute memories
                            match analyzer::execute_memories(&result.memories) {
                                Ok(ids) => {
                                    println!("Added {} memories to engram", ids.len());
                                    for id in ids {
                                        println!("  → {}", id);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error adding memories: {}", e);
                                }
                            }
                        }

                        if !result.stderr.is_empty() {
                            eprintln!("Analyzer stderr: {}", result.stderr);
                        }
                    }
                    Err(e) => {
                        eprintln!("Analysis failed: {}", e);
                    }
                }

                last_analyzed = current_size;
                println!("===========================================\n");
            }
        }
    });

    // Start proxy server
    let proxy_config = ProxyConfig {
        port: config.port,
        buffer,
        ca: Arc::new(ca),
    };

    // Handle Ctrl+C gracefully
    tokio::select! {
        result = run_proxy(proxy_config) => {
            if let Err(e) = result {
                eprintln!("Proxy error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\nShutting down gracefully...");
        }
    }

    Ok(())
}
