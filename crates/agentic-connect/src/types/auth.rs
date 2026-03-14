//! Authentication types — Invention 2: Adaptive Authentication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Supported authentication methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethod {
    /// No authentication required.
    None,

    /// HTTP Basic authentication.
    Basic { username: String, password: String },

    /// Bearer token (API key, JWT, etc).
    Bearer { token: String },

    /// API key in header or query parameter.
    ApiKey {
        key: String,
        header_name: Option<String>,
        query_param: Option<String>,
    },

    /// OAuth 2.0 with token refresh.
    OAuth2 {
        client_id: String,
        client_secret: Option<String>,
        access_token: Option<String>,
        refresh_token: Option<String>,
        token_url: String,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
        grant_type: OAuth2GrantType,
    },

    /// SSH key authentication.
    SshKey {
        username: String,
        private_key_path: String,
        passphrase: Option<String>,
    },

    /// SSH password authentication.
    SshPassword { username: String, password: String },

    /// Mutual TLS (client certificate).
    MutualTls {
        cert_path: String,
        key_path: String,
        ca_path: Option<String>,
    },
}

/// OAuth 2.0 grant types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2GrantType {
    AuthorizationCode,
    ClientCredentials,
    RefreshToken,
    DeviceCode,
}

impl AuthMethod {
    /// Whether this auth method has an expirable token.
    pub fn has_expiry(&self) -> bool {
        matches!(self, AuthMethod::OAuth2 { .. })
    }

    /// Whether the token is expired (if applicable).
    pub fn is_expired(&self) -> bool {
        match self {
            AuthMethod::OAuth2 { expires_at, .. } => {
                expires_at.map_or(false, |exp| exp < Utc::now())
            }
            _ => false,
        }
    }

    /// Whether this auth method needs a refresh.
    pub fn needs_refresh(&self) -> bool {
        match self {
            AuthMethod::OAuth2 {
                expires_at,
                refresh_token,
                ..
            } => {
                refresh_token.is_some()
                    && expires_at.map_or(false, |exp| {
                        // Refresh 5 minutes before expiry
                        exp - chrono::Duration::minutes(5) < Utc::now()
                    })
            }
            _ => false,
        }
    }

    /// Display name for the auth method type.
    pub fn method_name(&self) -> &'static str {
        match self {
            AuthMethod::None => "none",
            AuthMethod::Basic { .. } => "basic",
            AuthMethod::Bearer { .. } => "bearer",
            AuthMethod::ApiKey { .. } => "api_key",
            AuthMethod::OAuth2 { .. } => "oauth2",
            AuthMethod::SshKey { .. } => "ssh_key",
            AuthMethod::SshPassword { .. } => "ssh_password",
            AuthMethod::MutualTls { .. } => "mtls",
        }
    }
}

/// Stored credential in the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: uuid::Uuid,
    pub name: String,
    pub auth: AuthMethod,
    pub created_at: DateTime<Utc>,
    pub last_rotated: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}
