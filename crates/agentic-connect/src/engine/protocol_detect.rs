//! Protocol detection engine — probes hosts to identify protocols.

use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::types::{ConnectResult, Protocol};

/// Result of a protocol probe.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProbeResult {
    pub host: String,
    pub port: u16,
    pub protocol: Option<Protocol>,
    pub reachable: bool,
    pub latency_ms: u64,
    pub banner: Option<String>,
    pub tls: bool,
}

/// Probe a host:port to detect the protocol by reading the server banner.
pub async fn probe_host(host: &str, port: u16, timeout_ms: u64) -> ProbeResult {
    let addr = format!("{}:{}", host, port);
    let start = std::time::Instant::now();
    let dur = Duration::from_millis(timeout_ms);

    // Try TCP connect
    let stream = match timeout(dur, TcpStream::connect(&addr)).await {
        Ok(Ok(s)) => s,
        _ => {
            return ProbeResult {
                host: host.into(), port, protocol: None,
                reachable: false, latency_ms: start.elapsed().as_millis() as u64,
                banner: None, tls: false,
            };
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    // Try reading a banner (some protocols send one on connect)
    let banner = read_banner(&stream, Duration::from_millis(2000)).await;

    // Detect protocol from port + banner
    let protocol = detect_from_banner_and_port(port, banner.as_deref());
    let tls = matches!(protocol, Some(Protocol::Https | Protocol::Wss | Protocol::Sftp | Protocol::Imap));

    ProbeResult {
        host: host.into(), port, protocol, reachable: true,
        latency_ms, banner, tls,
    }
}

/// Probe multiple common ports on a host.
pub async fn scan_host(host: &str, timeout_ms: u64) -> Vec<ProbeResult> {
    let ports = [22, 80, 443, 3306, 5432, 6379, 8080, 8443, 9090, 27017];
    let mut results = Vec::new();
    for &port in &ports {
        let r = probe_host(host, port, timeout_ms).await;
        if r.reachable {
            results.push(r);
        }
    }
    results
}

async fn read_banner(stream: &TcpStream, timeout_dur: Duration) -> Option<String> {
    use tokio::io::AsyncReadExt;
    let mut buf = vec![0u8; 512];
    match timeout(timeout_dur, stream.readable()).await {
        Ok(Ok(())) => {
            match stream.try_read(&mut buf) {
                Ok(n) if n > 0 => {
                    let s = String::from_utf8_lossy(&buf[..n]).to_string();
                    Some(s.trim().to_string())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn detect_from_banner_and_port(port: u16, banner: Option<&str>) -> Option<Protocol> {
    // Banner-based detection first
    if let Some(b) = banner {
        let lower = b.to_lowercase();
        if lower.starts_with("ssh-") { return Some(Protocol::Ssh); }
        if lower.starts_with("220 ") && lower.contains("smtp") { return Some(Protocol::Smtp); }
        if lower.starts_with("* ok") || lower.starts_with("* preauth") { return Some(Protocol::Imap); }
        if lower.starts_with("+ok") { return Some(Protocol::Ftp); } // POP3 actually, but close
        if lower.starts_with("220 ") && lower.contains("ftp") { return Some(Protocol::Ftp); }
        if lower.starts_with("+pong") || lower.starts_with("-err") { return Some(Protocol::Redis); }
        if lower.contains("mysql") { return Some(Protocol::Mysql); }
        if lower.starts_with("http/") { return Some(Protocol::Http); }
    }

    // Port-based fallback
    match port {
        22 => Some(Protocol::Ssh),
        80 | 8080 => Some(Protocol::Http),
        443 | 8443 => Some(Protocol::Https),
        21 => Some(Protocol::Ftp),
        25 | 587 => Some(Protocol::Smtp),
        993 => Some(Protocol::Imap),
        53 => Some(Protocol::Dns),
        1883 => Some(Protocol::Mqtt),
        5672 => Some(Protocol::Amqp),
        6379 => Some(Protocol::Redis),
        5432 => Some(Protocol::Postgres),
        3306 => Some(Protocol::Mysql),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ssh_banner() {
        let proto = detect_from_banner_and_port(22, Some("SSH-2.0-OpenSSH_8.9"));
        assert_eq!(proto, Some(Protocol::Ssh));
    }

    #[test]
    fn test_detect_redis_banner() {
        let proto = detect_from_banner_and_port(6379, Some("+PONG"));
        assert_eq!(proto, Some(Protocol::Redis));
    }

    #[test]
    fn test_detect_by_port_no_banner() {
        assert_eq!(detect_from_banner_and_port(443, None), Some(Protocol::Https));
        assert_eq!(detect_from_banner_and_port(5432, None), Some(Protocol::Postgres));
        assert_eq!(detect_from_banner_and_port(9999, None), None);
    }
}
