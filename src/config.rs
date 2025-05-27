use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Database types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database
    MySQL,
    /// MongoDB database
    MongoDB,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::MongoDB => write!(f, "MongoDB"),
        }
    }
}

impl std::str::FromStr for DatabaseType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" | "psql" => Ok(DatabaseType::PostgreSQL),
            "mysql" | "mariadb" => Ok(DatabaseType::MySQL),
            "mongodb" | "mongo" => Ok(DatabaseType::MongoDB),
            _ => Err(AppError::UnknownDatabaseType(s.to_string())),
        }
    }
}

/// Database connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    /// Database type
    pub db_type: DatabaseType,
    /// Container name
    pub container: String,
    /// Username
    pub user: String,
    /// Password
    pub password: Option<String>,
    /// Database name
    pub database: Option<String>,
    /// Port number
    pub port: Option<u16>,
    /// Additional options
    pub options: Option<HashMap<String, String>>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Version
    pub version: String,
    /// Database connection aliases
    pub connections: HashMap<String, DatabaseConnection>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            connections: HashMap::new(),
        }
    }
}

impl Config {
    /// Create new configuration object
    pub fn new() -> Self {
        Default::default()
    }

    /// Get configuration file path
    pub fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "docker_db_container_login")
            .ok_or_else(|| AppError::Config("Failed to get config directory".to_string()))?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)
            .map_err(|e| AppError::Config(format!("Failed to create config directory: {}", e)))?;

        Ok(config_dir.join("config.yaml"))
    }

    /// Load from configuration file
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let config_str = serde_yaml::to_string(self)?;
        fs::write(&config_path, config_str)?;

        // Set file permissions to 600 (owner read/write only) on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&config_path, perms)?;
        }

        Ok(())
    }

    /// Add connection information
    pub fn add_connection(&mut self, name: String, connection: DatabaseConnection) -> Result<()> {
        self.connections.insert(name, connection);
        self.save()?;
        Ok(())
    }

    /// Remove connection information
    pub fn remove_connection(&mut self, name: &str) -> Result<()> {
        if self.connections.remove(name).is_none() {
            return Err(AppError::AliasNotFound(name.to_string()));
        }
        self.save()?;
        Ok(())
    }

    /// Get connection information from alias
    pub fn get_connection(&self, name: &str) -> Result<&DatabaseConnection> {
        self.connections
            .get(name)
            .ok_or_else(|| AppError::AliasNotFound(name.to_string()))
    }

    /// Get list of connections
    pub fn list_connections(&self) -> Vec<(&String, &DatabaseConnection)> {
        self.connections.iter().collect()
    }
}
