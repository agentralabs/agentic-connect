//! Database engine — SQLite query execution with schema discovery.

use rusqlite::Connection as SqlConn;
use std::collections::HashMap;
use std::path::Path;

use crate::types::{ConnectError, ConnectResult};

/// Database connection wrapper with schema introspection.
pub struct DbConnection {
    db: SqlConn,
    db_type: DbType,
    path: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum DbType { Sqlite }

/// Column metadata from schema discovery.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub default_value: Option<String>,
}

/// Table metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub row_count: Option<i64>,
}

/// Query result as rows of key-value pairs.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
    pub affected: usize,
}

impl DbConnection {
    /// Open a SQLite database.
    pub fn open_sqlite(path: &str) -> ConnectResult<Self> {
        let db = if path == ":memory:" {
            SqlConn::open_in_memory()?
        } else {
            SqlConn::open(Path::new(path))?
        };
        Ok(Self { db, db_type: DbType::Sqlite, path: path.into() })
    }

    /// Detect database type from connection string.
    pub fn from_url(url: &str) -> ConnectResult<Self> {
        if url.starts_with("sqlite://") || url.ends_with(".db") || url.ends_with(".sqlite") {
            let path = url.strip_prefix("sqlite://").unwrap_or(url);
            Self::open_sqlite(path)
        } else {
            Err(ConnectError::NotSupported(
                "Only SQLite is supported in this build. Use sqlx feature for Postgres/MySQL".into()
            ))
        }
    }

    /// Execute a query and return results.
    pub fn query(&self, sql: &str) -> ConnectResult<QueryResult> {
        let mut stmt = self.db.prepare(sql)?;
        let col_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let col_count = col_names.len();

        let rows: Vec<HashMap<String, serde_json::Value>> = stmt
            .query_map([], |row| {
                let mut map = HashMap::new();
                for i in 0..col_count {
                    let val = match row.get_ref(i) {
                        Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
                        Ok(rusqlite::types::ValueRef::Integer(n)) => serde_json::json!(n),
                        Ok(rusqlite::types::ValueRef::Real(f)) => serde_json::json!(f),
                        Ok(rusqlite::types::ValueRef::Text(s)) => {
                            serde_json::Value::String(String::from_utf8_lossy(s).into())
                        }
                        Ok(rusqlite::types::ValueRef::Blob(b)) => {
                            serde_json::json!(format!("<blob {} bytes>", b.len()))
                        }
                        Err(_) => serde_json::Value::Null,
                    };
                    map.insert(col_names[i].clone(), val);
                }
                Ok(map)
            })?
            .filter_map(|r| r.ok())
            .collect();

        let count = rows.len();
        Ok(QueryResult { columns: col_names, rows, row_count: count, affected: 0 })
    }

    /// Execute a write statement (INSERT/UPDATE/DELETE).
    pub fn execute(&self, sql: &str) -> ConnectResult<usize> {
        let affected = self.db.execute(sql, [])?;
        Ok(affected)
    }

    /// Discover all tables and their schemas.
    pub fn discover_schema(&self) -> ConnectResult<Vec<TableInfo>> {
        let tables = self.query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        )?;

        let mut result = Vec::new();
        for row in &tables.rows {
            if let Some(serde_json::Value::String(name)) = row.get("name") {
                let columns = self.table_columns(name)?;
                let count = self.query(&format!("SELECT COUNT(*) as cnt FROM \"{}\"", name))?;
                let row_count = count.rows.first()
                    .and_then(|r| r.get("cnt"))
                    .and_then(|v| v.as_i64());
                result.push(TableInfo { name: name.clone(), columns, row_count });
            }
        }
        Ok(result)
    }

    /// Get column info for a table.
    pub fn table_columns(&self, table: &str) -> ConnectResult<Vec<ColumnInfo>> {
        let info = self.query(&format!("PRAGMA table_info(\"{}\")", table))?;
        let mut cols = Vec::new();
        for row in &info.rows {
            cols.push(ColumnInfo {
                name: row.get("name").and_then(|v| v.as_str()).unwrap_or("").into(),
                data_type: row.get("type").and_then(|v| v.as_str()).unwrap_or("").into(),
                nullable: row.get("notnull").and_then(|v| v.as_i64()).unwrap_or(0) == 0,
                primary_key: row.get("pk").and_then(|v| v.as_i64()).unwrap_or(0) != 0,
                default_value: row.get("dflt_value").and_then(|v| v.as_str()).map(String::from),
            });
        }
        Ok(cols)
    }

    pub fn db_type(&self) -> DbType { self.db_type }
    pub fn path(&self) -> &str { &self.path }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_query() {
        let db = DbConnection::open_sqlite(":memory:").unwrap();
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.execute("INSERT INTO test VALUES (1, 'alice')").unwrap();
        db.execute("INSERT INTO test VALUES (2, 'bob')").unwrap();
        let result = db.query("SELECT * FROM test ORDER BY id").unwrap();
        assert_eq!(result.row_count, 2);
        assert_eq!(result.columns, vec!["id", "name"]);
    }

    #[test]
    fn test_schema_discovery() {
        let db = DbConnection::open_sqlite(":memory:").unwrap();
        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL, age REAL)").unwrap();
        let schema = db.discover_schema().unwrap();
        assert_eq!(schema.len(), 1);
        assert_eq!(schema[0].name, "users");
        assert_eq!(schema[0].columns.len(), 3);
        assert!(schema[0].columns[0].primary_key);
    }

    #[test]
    fn test_from_url_sqlite() {
        let db = DbConnection::from_url("sqlite://:memory:").unwrap();
        db.execute("CREATE TABLE t (x INTEGER)").unwrap();
        let r = db.query("SELECT COUNT(*) as c FROM t").unwrap();
        assert_eq!(r.rows[0]["c"], serde_json::json!(0));
    }
}
