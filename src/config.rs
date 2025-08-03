use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::error::QgoError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub name: String,
    pub db_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing, default)]
    pub password: String,
    pub database: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    SQLite,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub connections: Vec<Connection>,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub query_timeout_seconds: u64,
    pub max_rows_display: Option<usize>,
    pub auto_completion: bool,
    pub history_size: usize,
    pub export_format: ExportFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    Table,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            query_timeout_seconds: 5,
            max_rows_display: Some(1000),
            auto_completion: true,
            history_size: 1000,
            export_format: ExportFormat::Table,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            let config = Self {
                connections: Vec::new(),
                settings: Settings::default(),
            };
            config.save().await?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path).await?;
        
        // Try to parse the config, handling legacy format
        match serde_json::from_str::<Config>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("Warning: Failed to parse existing config: {}", e);
                eprintln!("Creating a backup and using default configuration...");
                
                // Backup the old config
                let backup_path = config_path.with_extension("json.backup");
                if let Err(backup_err) = fs::copy(&config_path, &backup_path).await {
                    eprintln!("Warning: Failed to create backup: {}", backup_err);
                }
                
                // Create new default config
                let config = Self {
                    connections: Vec::new(),
                    settings: Settings::default(),
                };
                config.save().await?;
                Ok(config)
            }
        }
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content).await?;
        Ok(())
    }

    pub fn add_connection(&mut self, connection: Connection) {
        // Remove any existing connection with the same name
        self.connections.retain(|c| c.name != connection.name);
        self.connections.push(connection);
    }

    pub fn remove_connection(&mut self, id: &Uuid) -> Result<()> {
        let initial_len = self.connections.len();
        self.connections.retain(|c| c.id != *id);
        
        if self.connections.len() == initial_len {
            return Err(QgoError::ConnectionNotFound(id.to_string()).into());
        }
        
        Ok(())
    }

    pub fn get_connection_by_name(&self, name: &str) -> Option<&Connection> {
        self.connections.iter().find(|c| c.name == name)
    }

    #[allow(dead_code)]
    pub fn get_connection_by_id(&self, id: &Uuid) -> Option<&Connection> {
        self.connections.iter().find(|c| c.id == *id)
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find config directory"
            ))?;
        
        Ok(config_dir.join("qgo").join("config.json"))
    }
}

impl Connection {
    pub fn new(
        name: String,
        db_type: DatabaseType,
        host: String,
        port: u16,
        username: String,
        password: String,
        database: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            db_type,
            host,
            port,
            username,
            password,
            database,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn connection_string(&self) -> String {
        match self.db_type {
            DatabaseType::MySQL => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    urlencoding::encode(&self.username),
                    urlencoding::encode(&self.password), 
                    self.host, 
                    self.port, 
                    urlencoding::encode(&self.database)
                )
            }
            DatabaseType::PostgreSQL => {
                format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    urlencoding::encode(&self.username),
                    urlencoding::encode(&self.password),
                    self.host,
                    self.port,
                    urlencoding::encode(&self.database)
                )
            }
            DatabaseType::SQLite => {
                // For SQLite, the database field should be the file path
                if self.database.starts_with("/") || self.database.contains(":") {
                    format!("sqlite://{}", self.database)
                } else {
                    format!("sqlite://./{}", self.database)
                }
            }
        }
    }

    pub fn display_name(&self) -> String {
        format!("{} ({}:{})", self.name, self.host, self.port)
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::SQLite => write!(f, "SQLite"),
        }
    }
}
