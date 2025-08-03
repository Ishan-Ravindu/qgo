use anyhow::Result;
use sqlx::{AnyPool, Column, Row};
use std::collections::HashMap;
use std::time::Duration;

use crate::config::{Connection, DatabaseType};
use crate::error::QgoError;

pub struct Database {
    pool: AnyPool,
    connection: Connection,
    tables_cache: Option<Vec<String>>,
    columns_cache: Option<HashMap<String, Vec<String>>>,
}

impl Database {
    pub async fn connect(connection: Connection, timeout: Duration) -> Result<Self> {
        let connection_string = connection.connection_string();
        
        // Log connection attempt (without password for security)
        println!("Connecting to {} database at {}:{}...", 
                 connection.db_type, connection.host, connection.port);
        
        // Apply timeout to the connection attempt
        let connect_future = AnyPool::connect(&connection_string);
        let pool = tokio::time::timeout(timeout, connect_future)
            .await
            .map_err(|_| {
                eprintln!("Connection timeout after {} seconds", timeout.as_secs());
                QgoError::Database(sqlx::Error::PoolTimedOut)
            })?
            .map_err(|e| {
                eprintln!("Database connection failed: {}", e);
                QgoError::Database(e)
            })?;

        Ok(Self {
            pool,
            connection,
            tables_cache: None,
            columns_cache: None,
        })
    }

    pub async fn test_connection(connection: &Connection, timeout: Duration) -> Result<()> {
        let connection_string = connection.connection_string();
        
        println!("Testing connection to {} database at {}:{}...", 
                 connection.db_type, connection.host, connection.port);
        
        // Apply timeout to the connection attempt
        let connect_future = AnyPool::connect(&connection_string);
        let pool = tokio::time::timeout(timeout, connect_future)
            .await
            .map_err(|_| {
                eprintln!("Connection test timeout after {} seconds", timeout.as_secs());
                QgoError::Database(sqlx::Error::PoolTimedOut)
            })?
            .map_err(|e| {
                eprintln!("Database connection test failed: {}", e);
                QgoError::Database(e)
            })?;

        let _test_conn = pool.acquire().await.map_err(|e| {
            eprintln!("Failed to acquire database connection: {}", e);
            QgoError::Database(e)
        })?;
        
        pool.close().await;
        
        Ok(())
    }

    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        let trimmed_query = query.trim();
        
        if trimmed_query.is_empty() {
            return Err(QgoError::InvalidQuery("Query cannot be empty".to_string()).into());
        }
        
        // Check if query is safe (read-only operations)
        let lower_query = trimmed_query.to_lowercase();
        let allowed_prefixes = ["select", "show", "describe", "explain", "with"];
        
        let is_allowed = allowed_prefixes.iter().any(|prefix| {
            lower_query.starts_with(prefix)
        });
        
        if !is_allowed {
            return Err(QgoError::InvalidQuery(
                "Only SELECT, SHOW, DESCRIBE, EXPLAIN, and WITH queries are allowed".to_string()
            ).into());
        }

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                eprintln!("Query execution failed: {}", e);
                QgoError::Database(e)
            })?;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                row_count: 0,
            });
        }

        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect();

        let mut result_rows = Vec::new();
        for row in rows {
            let mut result_row = Vec::new();
            for (i, _column) in columns.iter().enumerate() {
                let value: Option<String> = row.try_get(i).ok();
                result_row.push(value.unwrap_or_else(|| "NULL".to_string()));
            }
            result_rows.push(result_row);
        }

        let row_count = result_rows.len();

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count,
        })
    }

    pub async fn get_tables(&mut self) -> Result<Vec<String>> {
        if let Some(ref tables) = self.tables_cache {
            return Ok(tables.clone());
        }

        let query = match self.connection.db_type {
            DatabaseType::MySQL => "SHOW TABLES",
            DatabaseType::PostgreSQL => {
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'"
            }
            DatabaseType::SQLite => {
                "SELECT name FROM sqlite_master WHERE type='table'"
            }
        };

        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QgoError::Database(e))?;

        let tables: Vec<String> = rows
            .iter()
            .filter_map(|row| row.try_get::<String, _>(0).ok())
            .collect();

        self.tables_cache = Some(tables.clone());
        Ok(tables)
    }

    pub async fn get_columns(&mut self, table: &str) -> Result<Vec<String>> {
        if let Some(ref cache) = self.columns_cache {
            if let Some(columns) = cache.get(table) {
                return Ok(columns.clone());
            }
        }

        let query = match self.connection.db_type {
            DatabaseType::MySQL => format!("SHOW COLUMNS FROM `{}`", table),
            DatabaseType::PostgreSQL => format!(
                "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' AND table_schema = 'public'",
                table
            ),
            DatabaseType::SQLite => format!("PRAGMA table_info({})", table),
        };

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QgoError::Database(e))?;

        let columns: Vec<String> = match self.connection.db_type {
            DatabaseType::SQLite => {
                // SQLite PRAGMA returns: cid, name, type, notnull, dflt_value, pk
                rows.iter()
                    .filter_map(|row| row.try_get::<String, _>(1).ok()) // name is at index 1
                    .collect()
            }
            _ => {
                rows.iter()
                    .filter_map(|row| row.try_get::<String, _>(0).ok())
                    .collect()
            }
        };

        if self.columns_cache.is_none() {
            self.columns_cache = Some(HashMap::new());
        }
        
        if let Some(ref mut cache) = self.columns_cache {
            cache.insert(table.to_string(), columns.clone());
        }

        Ok(columns)
    }

    pub fn get_connection(&self) -> &Connection {
        &self.connection
    }

    #[allow(dead_code)]
    pub async fn refresh_cache(&mut self) -> Result<()> {
        self.tables_cache = None;
        self.columns_cache = None;
        self.get_tables().await?;
        
        // Pre-populate columns cache for all tables
        let tables = self.tables_cache.clone().unwrap_or_default();
        for table in tables {
            self.get_columns(&table).await?;
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
}

impl QueryResult {
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
