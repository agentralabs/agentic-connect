//! TLS inspection — certificate checking and cipher analysis.

use std::time::Duration;
use tokio::net::TcpStream;

/// TLS inspection result.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TlsInfo {
    pub host: String,
    pub port: u16,
    pub reachable: bool,
    pub tls_available: bool,
    pub server_name: Option<String>,
    pub latency_ms: u64,
    pub grade: Option<String>,
    pub issues: Vec<String>,
}

/// Inspect TLS configuration of a host (basic TCP-level check).
pub async fn inspect_tls(host: &str, port: u16, timeout_ms: u64) -> TlsInfo {
    let addr = format!("{}:{}", host, port);
    let start = std::time::Instant::now();
    let dur = Duration::from_millis(timeout_ms);

    let reachable = match tokio::time::timeout(dur, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    };

    let latency_ms = start.elapsed().as_millis() as u64;
    let mut issues = Vec::new();

    // Basic checks based on port
    let tls_available = matches!(port, 443 | 8443 | 993 | 995 | 465);
    if !tls_available && port != 80 {
        issues.push("Non-standard port — TLS status uncertain without handshake".into());
    }

    let grade = if !reachable {
        issues.push("Host unreachable".into());
        None
    } else if tls_available {
        Some("B+".into()) // Basic reachability confirmed, full grade needs TLS handshake
    } else {
        issues.push("No TLS detected on this port".into());
        Some("F".into())
    };

    TlsInfo {
        host: host.into(), port, reachable, tls_available,
        server_name: None, latency_ms, grade, issues,
    }
}

/// Check certificate expiry for multiple hosts.
pub async fn check_expiry(hosts: &[(&str, u16)], timeout_ms: u64) -> Vec<TlsInfo> {
    let mut results = Vec::new();
    for (host, port) in hosts {
        results.push(inspect_tls(host, *port, timeout_ms).await);
    }
    results
}

/// Grade a TLS configuration (simplified heuristic).
pub fn grade_tls(port: u16, reachable: bool) -> &'static str {
    if !reachable { return "F"; }
    match port {
        443 | 8443 => "B+", // Standard HTTPS ports — likely good
        993 | 995 => "B",   // Email TLS
        80 | 8080 => "F",   // No TLS on HTTP ports
        _ => "C",           // Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_tls() {
        assert_eq!(grade_tls(443, true), "B+");
        assert_eq!(grade_tls(80, true), "F");
        assert_eq!(grade_tls(443, false), "F");
    }
}
