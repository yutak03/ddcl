use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

use crate::config::{DatabaseConnection, DatabaseType};

/// CLI tool for easily connecting to Docker database containers
#[derive(Debug, Parser)]
#[command(
    name = "dbcli",
    about = "A tool to easily connect to Docker database containers",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Connect to a database container
    #[command(name = "connect", about = "Connect to a database container")]
    Connect(ConnectArgs),

    /// Add a connection configuration
    #[command(name = "add", about = "Add a connection configuration")]
    Add(AddArgs),

    /// Remove a connection configuration
    #[command(name = "remove", about = "Remove a connection configuration")]
    Remove(RemoveArgs),

    /// Display a list of connection configurations
    #[command(name = "list", about = "Display a list of connection configurations")]
    List,
}

/// Connect command arguments
#[derive(Debug, Args)]
pub struct ConnectArgs {
    /// Alias name (if not specified, container name and other arguments are required)
    pub alias: Option<String>,

    /// Container name (when not using alias)
    #[arg(short, long)]
    pub container: Option<String>,

    /// Database type (postgres, mysql, or mongodb)
    #[arg(short, long)]
    pub db_type: Option<String>,

    /// Username
    #[arg(short, long)]
    pub user: Option<String>,

    /// Password
    #[arg(short, long)]
    pub password: Option<String>,

    /// Database name
    #[arg(short = 'n', long)]
    pub database: Option<String>,

    /// Port number
    #[arg(short = 'P', long)]
    pub port: Option<u16>,
}

impl ConnectArgs {
    /// Convert connection info to DatabaseConnection
    pub fn to_connection(&self) -> Option<DatabaseConnection> {
        if let (Some(container), Some(db_type_str), Some(user)) =
            (&self.container, &self.db_type, &self.user)
        {
            if let Ok(db_type) = DatabaseType::from_str(db_type_str) {
                return Some(DatabaseConnection {
                    db_type,
                    container: container.clone(),
                    user: user.clone(),
                    password: self.password.clone(),
                    database: self.database.clone(),
                    port: self.port,
                    options: None,
                });
            }
        }
        None
    }
}

/// Add command arguments
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Alias name
    #[arg(required = false)]
    pub alias: Option<String>,

    /// Container name
    #[arg(short, long)]
    pub container: Option<String>,

    /// Database type (postgres, mysql, or mongodb)
    #[arg(short, long)]
    pub db_type: Option<String>,

    /// Username
    #[arg(short, long)]
    pub user: Option<String>,

    /// Password
    #[arg(short, long)]
    pub password: Option<String>,

    /// Database name
    #[arg(short = 'n', long)]
    pub database: Option<String>,

    /// Port number
    #[arg(short = 'P', long)]
    pub port: Option<u16>,

    /// Use interactive mode
    #[arg(short, long)]
    pub interactive: bool,

    /// Use auto-detect mode (auto-detect running DB containers)
    #[arg(short = 'a', long)]
    pub auto_detect: bool,
}

impl AddArgs {
    /// Convert connection info to DatabaseConnection
    pub fn to_connection(&self) -> Result<DatabaseConnection, String> {
        // Return None for interactive mode (caller handles user input)
        if self.interactive
            && (self.container.is_none() || self.db_type.is_none() || self.user.is_none())
        {
            return Err("Missing required information in interactive mode".to_string());
        }

        let db_type_str = match &self.db_type {
            Some(db_type) => db_type,
            None => return Err("Database type not specified".to_string()),
        };

        let db_type = DatabaseType::from_str(db_type_str)
            .map_err(|e| format!("Database type parse error: {}", e))?;

        let container = match &self.container {
            Some(container) => container.clone(),
            None => return Err("Container name not specified".to_string()),
        };

        let user = match &self.user {
            Some(user) => user.clone(),
            None => return Err("Username not specified".to_string()),
        };

        Ok(DatabaseConnection {
            db_type,
            container,
            user,
            password: self.password.clone(),
            database: self.database.clone(),
            port: self.port,
            options: None,
        })
    }
}

/// Remove command arguments
#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Alias name to remove
    pub alias: String,
}
