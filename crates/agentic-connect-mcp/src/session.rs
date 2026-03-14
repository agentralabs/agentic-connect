//! Session management — owns store, retry, vault, and DB connections.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use agentic_connect::{ConnectionStore, ConnectResult, CredentialVault, DbConnection, RetryEngine};

/// Manages all runtime state for the MCP server.
pub struct SessionManager {
    store: ConnectionStore,
    retry: RetryEngine,
    vault: CredentialVault,
    db_connections: HashMap<String, DbConnection>,
    data_path: PathBuf,
}

impl SessionManager {
    pub fn new(data_path: PathBuf) -> ConnectResult<Self> {
        let db_path = data_path.join("connections.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| agentic_connect::ConnectError::Io(e))?;
        }
        let store = ConnectionStore::open(&db_path)?;
        Ok(Self {
            store,
            retry: RetryEngine::new(),
            vault: CredentialVault::new(),
            db_connections: HashMap::new(),
            data_path,
        })
    }

    pub fn in_memory() -> ConnectResult<Self> {
        Ok(Self {
            store: ConnectionStore::open_memory()?,
            retry: RetryEngine::new(),
            vault: CredentialVault::new(),
            db_connections: HashMap::new(),
            data_path: PathBuf::from(":memory:"),
        })
    }

    pub fn store(&self) -> &ConnectionStore { &self.store }
    pub fn retry(&self) -> &RetryEngine { &self.retry }
    pub fn retry_mut(&mut self) -> &mut RetryEngine { &mut self.retry }
    pub fn vault(&self) -> &CredentialVault { &self.vault }
    pub fn vault_mut(&mut self) -> &mut CredentialVault { &mut self.vault }
    pub fn data_path(&self) -> &Path { &self.data_path }

    /// Open a database connection and store it by name.
    pub fn open_db(&mut self, name: &str, url: &str) -> ConnectResult<()> {
        let conn = DbConnection::from_url(url)?;
        self.db_connections.insert(name.to_string(), conn);
        Ok(())
    }

    /// Get a database connection by name.
    pub fn get_db(&self, name: &str) -> Option<&DbConnection> {
        self.db_connections.get(name)
    }

    /// List open database connections.
    pub fn list_dbs(&self) -> Vec<&str> {
        self.db_connections.keys().map(|s| s.as_str()).collect()
    }
}

pub fn resolve_data_path(custom: Option<PathBuf>) -> PathBuf {
    custom.unwrap_or_else(|| {
        dirs::data_dir().map(|d| d.join("agentic-connect"))
            .unwrap_or_else(|| PathBuf::from(".agentic-connect"))
    })
}
