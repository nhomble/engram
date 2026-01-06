/// HTTP/HTTPS proxy server for intercepting Claude API traffic
///
/// Transparently proxies all traffic while capturing Claude API messages

use super::buffer::{ConversationBuffer, Message};
use super::cert::CertificateAuthority;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::{Bytes, Incoming};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Proxy server configuration
pub struct ProxyConfig {
    /// Port to listen on
    pub port: u16,

    /// Conversation buffer for captured messages
    pub buffer: ConversationBuffer,

    /// CA for signing HTTPS certificates
    pub ca: Arc<CertificateAuthority>,
}

/// Start the proxy server
pub async fn run_proxy(config: ProxyConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    println!("Engram MITM proxy listening on http://{}", addr);
    println!("Configure clients: HTTP_PROXY=http://{} HTTPS_PROXY=http://{}", addr, addr);

    let config = Arc::new(config);

    loop {
        let (stream, _) = listener.accept().await?;
        let config = Arc::clone(&config);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, config).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

/// Handle a single proxy connection
async fn handle_connection(
    stream: TcpStream,
    config: Arc<ProxyConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let io = TokioIo::new(stream);

    http1::Builder::new()
        .serve_connection(
            io,
            service_fn(move |req| {
                let config = Arc::clone(&config);
                async move { handle_request(req, config).await }
            }),
        )
        .await?;

    Ok(())
}

/// Handle an HTTP request (either CONNECT for HTTPS or direct HTTP)
async fn handle_request(
    req: Request<Incoming>,
    config: Arc<ProxyConfig>,
) -> Result<Response<String>, hyper::Error> {
    if req.method() == Method::CONNECT {
        // HTTPS CONNECT tunnel
        handle_connect(req, config).await
    } else {
        // Direct HTTP proxy
        handle_http(req, config).await
    }
}

/// Handle HTTP CONNECT for HTTPS tunneling
async fn handle_connect(
    req: Request<Incoming>,
    _config: Arc<ProxyConfig>,
) -> Result<Response<String>, hyper::Error> {
    // For now, reject CONNECT - we'll implement TLS interception in a future iteration
    // This allows the proxy to work for non-HTTPS traffic initially

    let host = req.uri().authority().map(|a| a.as_str()).unwrap_or("unknown");

    println!("CONNECT request to: {}", host);

    // Return "Not Implemented" for now
    Ok(Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(format!("HTTPS interception not yet implemented for {}", host))
        .unwrap())
}

/// Handle direct HTTP proxy requests
async fn handle_http(
    req: Request<Incoming>,
    config: Arc<ProxyConfig>,
) -> Result<Response<String>, hyper::Error> {
    let uri = req.uri().clone();
    let method = req.method().clone();

    println!("HTTP request: {} {}", method, uri);

    // Check if this is a Claude API request
    let is_claude_api = uri.authority()
        .map(|a| a.host().contains("anthropic.com") || a.host().contains("claude.ai"))
        .unwrap_or(false);

    if is_claude_api {
        println!("  → Intercepted Claude API request");

        // For now, just log that we saw it
        // In a full implementation, we would:
        // 1. Forward the request to the real API
        // 2. Capture the request/response
        // 3. Parse messages and add to buffer

        // For MVP, return a placeholder
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body("Proxy intercepted (forwarding not yet implemented)".to_string())
            .unwrap())
    } else {
        // Non-Claude traffic - just pass through (not implemented yet)
        Ok(Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .body("Proxying non-Claude traffic not yet implemented".to_string())
            .unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_creation() {
        let buffer = ConversationBuffer::new(50);
        let ca = Arc::new(CertificateAuthority::load_or_create().unwrap());

        let config = ProxyConfig {
            port: 8080,
            buffer: buffer.clone(),
            ca,
        };

        assert_eq!(config.port, 8080);
    }

    // Note: Integration tests for actual proxy behavior would go in tests/ directory
    // and would start the proxy server and make requests to it
}
