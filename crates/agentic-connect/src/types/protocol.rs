//! Protocol types — Invention 1: Protocol Omniscience.

use serde::{Deserialize, Serialize};

/// Supported protocol types that AgenticConnect can handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
    WebSocket,
    Wss,
    Grpc,
    Ssh,
    Ftp,
    Sftp,
    Smtp,
    Imap,
    Dns,
    Mqtt,
    Amqp,
    Redis,
    Postgres,
    Mysql,
    Tcp,
    Udp,
}

impl Protocol {
    /// Default port for this protocol, if known.
    pub fn default_port(&self) -> Option<u16> {
        match self {
            Protocol::Http => Some(80),
            Protocol::Https => Some(443),
            Protocol::WebSocket => Some(80),
            Protocol::Wss => Some(443),
            Protocol::Grpc => Some(443),
            Protocol::Ssh => Some(22),
            Protocol::Ftp => Some(21),
            Protocol::Sftp => Some(22),
            Protocol::Smtp => Some(587),
            Protocol::Imap => Some(993),
            Protocol::Dns => Some(53),
            Protocol::Mqtt => Some(1883),
            Protocol::Amqp => Some(5672),
            Protocol::Redis => Some(6379),
            Protocol::Postgres => Some(5432),
            Protocol::Mysql => Some(3306),
            Protocol::Tcp | Protocol::Udp => None,
        }
    }

    /// Whether this protocol supports TLS natively.
    pub fn supports_tls(&self) -> bool {
        matches!(
            self,
            Protocol::Https
                | Protocol::Wss
                | Protocol::Grpc
                | Protocol::Sftp
                | Protocol::Imap
        )
    }

    /// Detect protocol from a URL scheme string.
    pub fn from_scheme(scheme: &str) -> Option<Self> {
        match scheme.to_lowercase().as_str() {
            "http" => Some(Protocol::Http),
            "https" => Some(Protocol::Https),
            "ws" => Some(Protocol::WebSocket),
            "wss" => Some(Protocol::Wss),
            "grpc" | "grpcs" => Some(Protocol::Grpc),
            "ssh" => Some(Protocol::Ssh),
            "ftp" => Some(Protocol::Ftp),
            "sftp" => Some(Protocol::Sftp),
            "smtp" | "smtps" => Some(Protocol::Smtp),
            "imap" | "imaps" => Some(Protocol::Imap),
            "dns" => Some(Protocol::Dns),
            "mqtt" | "mqtts" => Some(Protocol::Mqtt),
            "amqp" | "amqps" => Some(Protocol::Amqp),
            "redis" | "rediss" => Some(Protocol::Redis),
            "postgres" | "postgresql" => Some(Protocol::Postgres),
            "mysql" => Some(Protocol::Mysql),
            _ => None,
        }
    }

    /// Display name for this protocol.
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::Http => "HTTP",
            Protocol::Https => "HTTPS",
            Protocol::WebSocket => "WebSocket",
            Protocol::Wss => "WebSocket Secure",
            Protocol::Grpc => "gRPC",
            Protocol::Ssh => "SSH",
            Protocol::Ftp => "FTP",
            Protocol::Sftp => "SFTP",
            Protocol::Smtp => "SMTP",
            Protocol::Imap => "IMAP",
            Protocol::Dns => "DNS",
            Protocol::Mqtt => "MQTT",
            Protocol::Amqp => "AMQP",
            Protocol::Redis => "Redis",
            Protocol::Postgres => "PostgreSQL",
            Protocol::Mysql => "MySQL",
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

/// What a protocol can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    pub request_response: bool,
    pub streaming: bool,
    pub pub_sub: bool,
    pub bidirectional: bool,
    pub supports_tls: bool,
    pub supports_compression: bool,
}

impl Protocol {
    /// Get capabilities for this protocol.
    pub fn capabilities(&self) -> ProtocolCapabilities {
        match self {
            Protocol::Http | Protocol::Https => ProtocolCapabilities {
                request_response: true,
                streaming: false,
                pub_sub: false,
                bidirectional: false,
                supports_tls: matches!(self, Protocol::Https),
                supports_compression: true,
            },
            Protocol::WebSocket | Protocol::Wss => ProtocolCapabilities {
                request_response: true,
                streaming: true,
                pub_sub: true,
                bidirectional: true,
                supports_tls: matches!(self, Protocol::Wss),
                supports_compression: true,
            },
            Protocol::Grpc => ProtocolCapabilities {
                request_response: true,
                streaming: true,
                pub_sub: false,
                bidirectional: true,
                supports_tls: true,
                supports_compression: true,
            },
            Protocol::Mqtt | Protocol::Amqp => ProtocolCapabilities {
                request_response: false,
                streaming: true,
                pub_sub: true,
                bidirectional: true,
                supports_tls: false,
                supports_compression: false,
            },
            _ => ProtocolCapabilities {
                request_response: true,
                streaming: false,
                pub_sub: false,
                bidirectional: matches!(self, Protocol::Ssh),
                supports_tls: self.supports_tls(),
                supports_compression: false,
            },
        }
    }
}
