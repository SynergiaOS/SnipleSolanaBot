//! GHOST PROTOCOL - Pure Rust HTTP Client
//! 
//! Zero dependencies HTTP implementation with proxy rotation
//! Following Geohot doctrine: every byte must earn its existence

use std::collections::VecDeque;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use anyhow::{Result, anyhow};
use tracing::{debug, warn, error};

/// Ghost Protocol Error Types
#[derive(Debug, thiserror::Error)]
pub enum GhostError {
    #[error("Connection failed")]
    ConnectionFailed,
    #[error("Proxy timeout")]
    ProxyTimeout,
    #[error("Invalid response")]
    InvalidResponse,
    #[error("Parse error")]
    ParseError,
    #[error("All proxies failed")]
    AllProxiesFailed,
}

/// HTTP Response without dependencies
#[derive(Debug, Clone)]
pub struct GhostResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub response_time_ms: u64,
}

/// Proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub addr: SocketAddr,
    pub auth: Option<(String, String)>,
    pub timeout_ms: u64,
    pub max_retries: u8,
}

/// Ghost Fetcher - Pure Rust HTTP client with proxy rotation
pub struct GhostFetcher {
    proxy_pool: Vec<ProxyConfig>,
    current_proxy: AtomicUsize,
    user_agents: Vec<&'static str>,
    ua_index: AtomicUsize,
    connection_timeout: Duration,
    read_timeout: Duration,
}

impl GhostFetcher {
    /// Create new Ghost Fetcher with proxy pool
    pub fn new(proxies: Vec<&str>) -> Result<Self> {
        let mut proxy_pool = Vec::new();
        
        for proxy_str in proxies {
            let addr = proxy_str.parse::<SocketAddr>()
                .map_err(|_| anyhow!("Invalid proxy address: {}", proxy_str))?;
            
            proxy_pool.push(ProxyConfig {
                addr,
                auth: None,
                timeout_ms: 5000,
                max_retries: 3,
            });
        }
        
        if proxy_pool.is_empty() {
            return Err(anyhow!("No valid proxies provided"));
        }
        
        Ok(Self {
            proxy_pool,
            current_proxy: AtomicUsize::new(0),
            user_agents: Self::default_user_agents(),
            ua_index: AtomicUsize::new(0),
            connection_timeout: Duration::from_millis(5000),
            read_timeout: Duration::from_millis(10000),
        })
    }
    
    /// Default user agent rotation pool
    fn default_user_agents() -> Vec<&'static str> {
        vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:91.0) Gecko/20100101",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:91.0) Gecko/20100101",
        ]
    }
    
    /// Get next proxy in rotation
    fn next_proxy(&self) -> &ProxyConfig {
        let idx = self.current_proxy.fetch_add(1, Ordering::Relaxed) % self.proxy_pool.len();
        &self.proxy_pool[idx]
    }
    
    /// Get random user agent
    fn random_user_agent(&self) -> &str {
        let idx = self.ua_index.fetch_add(1, Ordering::Relaxed) % self.user_agents.len();
        self.user_agents[idx]
    }
    
    /// Fetch URL through proxy with full manual HTTP implementation
    pub async fn fetch(&self, url: &str) -> Result<GhostResponse> {
        let start_time = Instant::now();
        
        // Parse URL manually (zero dependencies)
        let (host, path, port, is_https) = self.parse_url(url)?;
        
        // Try each proxy until success
        for attempt in 0..self.proxy_pool.len() {
            let proxy = self.next_proxy();
            
            match self.fetch_through_proxy(url, &host, &path, port, is_https, proxy).await {
                Ok(mut response) => {
                    response.response_time_ms = start_time.elapsed().as_millis() as u64;
                    debug!("Ghost fetch successful: {} via proxy {} in {}ms", 
                           url, proxy.addr, response.response_time_ms);
                    return Ok(response);
                }
                Err(e) => {
                    warn!("Proxy {} failed for {}: {:?}", proxy.addr, url, e);
                    continue;
                }
            }
        }
        
        Err(anyhow!("All proxies failed for URL: {}", url))
    }
    
    /// Manual URL parsing (zero dependencies)
    fn parse_url(&self, url: &str) -> Result<(String, String, u16, bool)> {
        let url = url.trim();
        
        let (is_https, url_without_scheme) = if url.starts_with("https://") {
            (true, &url[8..])
        } else if url.starts_with("http://") {
            (false, &url[7..])
        } else {
            return Err(anyhow!("URL must start with http:// or https://"));
        };
        
        let (host_port, path) = if let Some(slash_pos) = url_without_scheme.find('/') {
            (&url_without_scheme[..slash_pos], &url_without_scheme[slash_pos..])
        } else {
            (url_without_scheme, "/")
        };
        
        let (host, port) = if let Some(colon_pos) = host_port.find(':') {
            let host = host_port[..colon_pos].to_string();
            let port = host_port[colon_pos + 1..].parse::<u16>()
                .map_err(|_| anyhow!("Invalid port number"))?;
            (host, port)
        } else {
            let default_port = if is_https { 443 } else { 80 };
            (host_port.to_string(), default_port)
        };
        
        Ok((host, path.to_string(), port, is_https))
    }
    
    /// Fetch through specific proxy
    async fn fetch_through_proxy(
        &self,
        url: &str,
        host: &str,
        path: &str,
        port: u16,
        _is_https: bool,
        proxy: &ProxyConfig,
    ) -> Result<GhostResponse> {
        // Connect to proxy
        let mut stream = timeout(
            self.connection_timeout,
            TcpStream::connect(proxy.addr)
        ).await
        .map_err(|_| GhostError::ProxyTimeout)?
        .map_err(|_| GhostError::ConnectionFailed)?;
        
        // Build HTTP request manually
        let user_agent = self.random_user_agent();
        let request = self.build_http_request(host, path, port, user_agent)?;
        
        // Send request
        stream.write_all(request.as_bytes()).await
            .map_err(|_| GhostError::ConnectionFailed)?;
        
        // Read response with timeout
        let mut response_buffer = Vec::with_capacity(8192);
        let mut temp_buffer = [0u8; 4096];
        
        loop {
            match timeout(self.read_timeout, stream.read(&mut temp_buffer)).await {
                Ok(Ok(0)) => break, // EOF
                Ok(Ok(n)) => response_buffer.extend_from_slice(&temp_buffer[..n]),
                Ok(Err(_)) => return Err(anyhow!("Read error")),
                Err(_) => return Err(anyhow!("Read timeout")),
            }
            
            // Prevent excessive memory usage
            if response_buffer.len() > 1024 * 1024 { // 1MB limit
                break;
            }
        }
        
        // Parse HTTP response manually
        self.parse_http_response(&response_buffer)
    }
    
    /// Build HTTP/1.1 request manually
    fn build_http_request(&self, host: &str, path: &str, port: u16, user_agent: &str) -> Result<String> {
        let host_header = if port == 80 || port == 443 {
            host.to_string()
        } else {
            format!("{}:{}", host, port)
        };
        
        Ok(format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             User-Agent: {}\r\n\
             Accept: */*\r\n\
             Connection: close\r\n\
             Cache-Control: no-cache\r\n\
             \r\n",
            path, host_header, user_agent
        ))
    }
    
    /// Parse HTTP response manually (zero dependencies)
    fn parse_http_response(&self, response_data: &[u8]) -> Result<GhostResponse> {
        let response_str = String::from_utf8_lossy(response_data);
        
        // Find header/body separator
        let header_end = response_str.find("\r\n\r\n")
            .ok_or_else(|| anyhow!("Invalid HTTP response: no header/body separator"))?;
        
        let headers_section = &response_str[..header_end];
        let body = response_str[header_end + 4..].to_string();
        
        // Parse status line
        let mut lines = headers_section.lines();
        let status_line = lines.next()
            .ok_or_else(|| anyhow!("Invalid HTTP response: no status line"))?;
        
        let status_code = self.parse_status_code(status_line)?;
        
        // Parse headers
        let mut headers = Vec::new();
        for line in lines {
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.push((name, value));
            }
        }
        
        Ok(GhostResponse {
            status_code,
            headers,
            body,
            response_time_ms: 0, // Will be set by caller
        })
    }
    
    /// Parse HTTP status code
    fn parse_status_code(&self, status_line: &str) -> Result<u16> {
        let parts: Vec<&str> = status_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid status line: {}", status_line));
        }
        
        parts[1].parse::<u16>()
            .map_err(|_| anyhow!("Invalid status code: {}", parts[1]))
    }
}

/// Ghost Protocol configuration
#[derive(Debug, Clone)]
pub struct GhostConfig {
    pub proxy_list: Vec<String>,
    pub connection_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub max_retries: u8,
    pub user_agent_rotation: bool,
}

impl Default for GhostConfig {
    fn default() -> Self {
        Self {
            proxy_list: vec![
                "127.0.0.1:8080".to_string(),
                "127.0.0.1:8081".to_string(),
            ],
            connection_timeout_ms: 5000,
            read_timeout_ms: 10000,
            max_retries: 3,
            user_agent_rotation: true,
        }
    }
}

/// Initialize Ghost Protocol with configuration
pub fn init_ghost_protocol(config: GhostConfig) -> Result<Arc<GhostFetcher>> {
    let proxy_refs: Vec<&str> = config.proxy_list.iter().map(|s| s.as_str()).collect();
    let fetcher = GhostFetcher::new(proxy_refs)?;
    Ok(Arc::new(fetcher))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_url_parsing() {
        let fetcher = GhostFetcher::new(vec!["127.0.0.1:8080"]).unwrap();
        
        let (host, path, port, is_https) = fetcher.parse_url("https://api.example.com/v1/data").unwrap();
        assert_eq!(host, "api.example.com");
        assert_eq!(path, "/v1/data");
        assert_eq!(port, 443);
        assert!(is_https);
    }
    
    #[tokio::test]
    async fn test_http_request_building() {
        let fetcher = GhostFetcher::new(vec!["127.0.0.1:8080"]).unwrap();
        let request = fetcher.build_http_request("api.example.com", "/test", 80, "TestAgent").unwrap();
        
        assert!(request.contains("GET /test HTTP/1.1"));
        assert!(request.contains("Host: api.example.com"));
        assert!(request.contains("User-Agent: TestAgent"));
    }
}
