use thiserror::Error;

#[derive(Error, Debug)]
pub enum QgoError {
    #[error("Configuration error: {0}")]
    Config(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Export error: {0}")]
    #[allow(dead_code)]
    Export(String),
    
    #[error("Interactive input error: {0}")]
    #[allow(dead_code)]
    Input(String),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, QgoError>;
