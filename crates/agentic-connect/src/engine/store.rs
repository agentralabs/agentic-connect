//! ConnectionStore — SQLite-backed storage for connections, profiles, and credentials.

use chrono::Utc;
use rusqlite::{params, Connection as SqlConn};
use std::path::Path;

use crate::types::{
    Connection, ConnectionId, ConnectionProfile, ConnectError, ConnectResult,
    HealthCheck, HealthStatus, Protocol, StoredCredential,
};

/// Persistent storage for all connection data.
pub struct ConnectionStore {
    db: SqlConn,
}

impl ConnectionStore {
    /// Open or create a store at the given path.
    pub fn open(path: &Path) -> ConnectResult<Self> {
        let db = SqlConn::open(path)?;
        let store = Self { db };
        store.init_tables()?;
        Ok(store)
    }

    /// Create an in-memory store (for testing).
    pub fn open_memory() -> ConnectResult<Self> {
        let db = SqlConn::open_in_memory()?;
        let store = Self { db };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> ConnectResult<()> {
        self.db.execute_batch(
            "CREATE TABLE IF NOT EXISTS connections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                protocol TEXT NOT NULL,
                host TEXT NOT NULL,
                port INTEGER,
                path TEXT,
                auth_json TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                last_used TEXT,
                metadata_json TEXT NOT NULL DEFAULT 'null'
            );
            CREATE TABLE IF NOT EXISTS profiles (
                connection_id TEXT PRIMARY KEY,
                profile_json TEXT NOT NULL,
                FOREIGN KEY (connection_id) REFERENCES connections(id)
            );
            CREATE TABLE IF NOT EXISTS credentials (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                auth_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_rotated TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]'
            );
            CREATE TABLE IF NOT EXISTS health_checks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                connection_id TEXT NOT NULL,
                status TEXT NOT NULL,
                latency_ms REAL,
                message TEXT,
                checked_at TEXT NOT NULL,
                FOREIGN KEY (connection_id) REFERENCES connections(id)
            );
            CREATE INDEX IF NOT EXISTS idx_health_conn ON health_checks(connection_id);
            CREATE INDEX IF NOT EXISTS idx_health_time ON health_checks(checked_at);",
        )?;
        Ok(())
    }

    /// Save a connection.
    pub fn save_connection(&self, conn: &Connection) -> ConnectResult<()> {
        let auth_json = conn.auth.as_ref().map(|a| serde_json::to_string(a).unwrap_or_default());
        let tags_json = serde_json::to_string(&conn.tags).unwrap_or_else(|_| "[]".into());
        let meta_json = serde_json::to_string(&conn.metadata).unwrap_or_else(|_| "null".into());

        self.db.execute(
            "INSERT OR REPLACE INTO connections
             (id, name, protocol, host, port, path, auth_json, tags_json, created_at, last_used, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                conn.id.to_string(),
                conn.name,
                serde_json::to_string(&conn.protocol).unwrap_or_default(),
                conn.host,
                conn.port,
                conn.path,
                auth_json,
                tags_json,
                conn.created_at.to_rfc3339(),
                conn.last_used.map(|t| t.to_rfc3339()),
                meta_json,
            ],
        )?;
        Ok(())
    }

    /// Get a connection by ID.
    pub fn get_connection(&self, id: &ConnectionId) -> ConnectResult<Option<Connection>> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, protocol, host, port, path, auth_json, tags_json,
                    created_at, last_used, metadata_json
             FROM connections WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id.to_string()], |row| {
            Ok(self.row_to_connection(row))
        });

        match result {
            Ok(conn) => Ok(Some(conn?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ConnectError::Database(e.to_string())),
        }
    }

    /// List all connections, optionally filtered by tag.
    pub fn list_connections(&self, tag: Option<&str>) -> ConnectResult<Vec<Connection>> {
        let mut stmt = self.db.prepare(
            "SELECT id, name, protocol, host, port, path, auth_json, tags_json,
                    created_at, last_used, metadata_json
             FROM connections ORDER BY name",
        )?;

        let rows = stmt.query_map([], |row| Ok(self.row_to_connection(row)))?;
        let mut connections = Vec::new();
        for row in rows {
            let conn = row.map_err(|e| ConnectError::Database(e.to_string()))??;
            if let Some(t) = tag {
                if conn.tags.iter().any(|ct| ct == t) {
                    connections.push(conn);
                }
            } else {
                connections.push(conn);
            }
        }
        Ok(connections)
    }

    /// Delete a connection by ID.
    pub fn delete_connection(&self, id: &ConnectionId) -> ConnectResult<bool> {
        let count = self.db.execute(
            "DELETE FROM connections WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(count > 0)
    }

    /// Save a connection profile (soul).
    pub fn save_profile(&self, profile: &ConnectionProfile) -> ConnectResult<()> {
        let json = serde_json::to_string(profile)
            .map_err(|e| ConnectError::Serialization(e.to_string()))?;
        self.db.execute(
            "INSERT OR REPLACE INTO profiles (connection_id, profile_json) VALUES (?1, ?2)",
            params![profile.connection_id.to_string(), json],
        )?;
        Ok(())
    }

    /// Get a connection profile by connection ID.
    pub fn get_profile(&self, conn_id: &ConnectionId) -> ConnectResult<Option<ConnectionProfile>> {
        let mut stmt = self.db.prepare(
            "SELECT profile_json FROM profiles WHERE connection_id = ?1",
        )?;
        match stmt.query_row(params![conn_id.to_string()], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        }) {
            Ok(json) => {
                let profile: ConnectionProfile = serde_json::from_str(&json)?;
                Ok(Some(profile))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ConnectError::Database(e.to_string())),
        }
    }

    /// Save a health check result.
    pub fn save_health_check(&self, check: &HealthCheck) -> ConnectResult<()> {
        let status = serde_json::to_string(&check.status).unwrap_or_default();
        self.db.execute(
            "INSERT INTO health_checks (connection_id, status, latency_ms, message, checked_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                check.connection_id.to_string(),
                status,
                check.latency_ms,
                check.message,
                check.checked_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get recent health checks for a connection.
    pub fn get_health_history(
        &self,
        conn_id: &ConnectionId,
        limit: usize,
    ) -> ConnectResult<Vec<HealthCheck>> {
        let mut stmt = self.db.prepare(
            "SELECT connection_id, status, latency_ms, message, checked_at
             FROM health_checks WHERE connection_id = ?1
             ORDER BY checked_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![conn_id.to_string(), limit as i64], |row| {
            let status_str: String = row.get(1)?;
            let status: HealthStatus =
                serde_json::from_str(&status_str).unwrap_or(HealthStatus::Unknown);
            let checked_str: String = row.get(4)?;
            Ok(HealthCheck {
                connection_id: *conn_id,
                protocol: Protocol::Http, // stored separately if needed
                host: String::new(),
                port: 0,
                status,
                latency_ms: row.get(2)?,
                message: row.get(3)?,
                checked_at: chrono::DateTime::parse_from_rfc3339(&checked_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        let mut checks = Vec::new();
        for row in rows {
            checks.push(row.map_err(|e| ConnectError::Database(e.to_string()))?);
        }
        Ok(checks)
    }

    /// Stats summary.
    pub fn stats(&self) -> ConnectResult<StoreStats> {
        let conn_count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM connections", [], |row| row.get(0),
        )?;
        let profile_count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM profiles", [], |row| row.get(0),
        )?;
        let health_count: i64 = self.db.query_row(
            "SELECT COUNT(*) FROM health_checks", [], |row| row.get(0),
        )?;
        Ok(StoreStats {
            connection_count: conn_count as usize,
            profile_count: profile_count as usize,
            health_check_count: health_count as usize,
        })
    }

    fn row_to_connection(&self, row: &rusqlite::Row) -> ConnectResult<Connection> {
        let id_str: String = row.get(0).map_err(|e| ConnectError::Database(e.to_string()))?;
        let protocol_str: String = row.get(2).map_err(|e| ConnectError::Database(e.to_string()))?;
        let auth_json: Option<String> = row.get(6).map_err(|e| ConnectError::Database(e.to_string()))?;
        let tags_json: String = row.get(7).map_err(|e| ConnectError::Database(e.to_string()))?;
        let created_str: String = row.get(8).map_err(|e| ConnectError::Database(e.to_string()))?;
        let last_used_str: Option<String> = row.get(9).map_err(|e| ConnectError::Database(e.to_string()))?;
        let meta_json: String = row.get(10).map_err(|e| ConnectError::Database(e.to_string()))?;

        Ok(Connection {
            id: uuid::Uuid::parse_str(&id_str)
                .map_err(|e| ConnectError::Database(e.to_string()))?,
            name: row.get(1).map_err(|e| ConnectError::Database(e.to_string()))?,
            protocol: serde_json::from_str(&protocol_str).unwrap_or(Protocol::Http),
            host: row.get(3).map_err(|e| ConnectError::Database(e.to_string()))?,
            port: row.get(4).map_err(|e| ConnectError::Database(e.to_string()))?,
            path: row.get(5).map_err(|e| ConnectError::Database(e.to_string()))?,
            auth: auth_json.and_then(|j| serde_json::from_str(&j).ok()),
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_used: last_used_str.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            metadata: serde_json::from_str(&meta_json).unwrap_or(serde_json::Value::Null),
        })
    }
}

/// Store statistics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoreStats {
    pub connection_count: usize,
    pub profile_count: usize,
    pub health_check_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_memory() {
        let store = ConnectionStore::open_memory().unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.connection_count, 0);
    }

    #[test]
    fn test_save_and_get_connection() {
        let store = ConnectionStore::open_memory().unwrap();
        let conn = Connection::from_url("test", "https://api.example.com/v1").unwrap();
        store.save_connection(&conn).unwrap();
        let loaded = store.get_connection(&conn.id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "test");
    }

    #[test]
    fn test_list_connections() {
        let store = ConnectionStore::open_memory().unwrap();
        let c1 = Connection::from_url("api", "https://api.example.com").unwrap();
        let c2 = Connection::from_url("ssh", "ssh://server.example.com").unwrap();
        store.save_connection(&c1).unwrap();
        store.save_connection(&c2).unwrap();
        let all = store.list_connections(None).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_connection() {
        let store = ConnectionStore::open_memory().unwrap();
        let conn = Connection::from_url("test", "https://example.com").unwrap();
        let id = conn.id;
        store.save_connection(&conn).unwrap();
        assert!(store.delete_connection(&id).unwrap());
        assert!(store.get_connection(&id).unwrap().is_none());
    }

    #[test]
    fn test_profile_roundtrip() {
        let store = ConnectionStore::open_memory().unwrap();
        let conn = Connection::from_url("test", "https://example.com").unwrap();
        store.save_connection(&conn).unwrap();
        let mut profile = ConnectionProfile::new(conn.id);
        profile.record_latency(150.0);
        store.save_profile(&profile).unwrap();
        let loaded = store.get_profile(&conn.id).unwrap().unwrap();
        assert_eq!(loaded.baseline.sample_count, 1);
    }
}
