//! Engine — connection lifecycle, storage, retry, vault, HTTP, DB, webhooks, TLS.

pub mod db_engine;
#[cfg(feature = "http")]
pub mod http_client;
pub mod protocol_detect;
pub mod retry_engine;
pub mod store;
pub mod tls_inspect;
pub mod vault;
pub mod webhook;

pub use db_engine::DbConnection;
#[cfg(feature = "http")]
pub use http_client::{http_request, HttpResponse};
pub use protocol_detect::{probe_host, scan_host, ProbeResult};
pub use retry_engine::RetryEngine;
pub use store::ConnectionStore;
pub use tls_inspect::{inspect_tls, TlsInfo};
pub use vault::CredentialVault;
pub use webhook::{compute_hmac_sha256, verify_hmac_sha256};
