use crate::config::{DatabaseConnection, DatabaseType};
use crate::db::DatabaseConnector;
use crate::error::{AppError, Result};
use dialoguer::{Input, Password, Select, theme::ColorfulTheme};
use std::str::FromStr;

// Implementation of From trait to convert dialoguer::Error
impl From<dialoguer::Error> for AppError {
    fn from(err: dialoguer::Error) -> Self {
        AppError::Other(err.to_string())
    }
}

/// Get connection information interactively
pub async fn get_connection_interactively() -> Result<(String, DatabaseConnection)> {
    let theme = ColorfulTheme::default();

    // Input alias name
    let alias: String = Input::with_theme(&theme)
        .with_prompt("alias name")
        .interact()?;

    // Input container name (with auto-detection option)
    let container = get_container_interactively(&theme).await?;

    // Select database type
    let db_types = &["PostgreSQL", "MySQL", "MongoDB"];
    let db_type_index = Select::with_theme(&theme)
        .with_prompt("Database type")
        .items(db_types)
        .default(0)
        .interact()?;
    let db_type = DatabaseType::from_str(db_types[db_type_index])?;

    // Input username (set default value according to database type)
    let default_user = match db_type {
        DatabaseType::PostgreSQL => "postgres",
        DatabaseType::MySQL => "root",
        DatabaseType::MongoDB => "mongo",
    };
    let user: String = Input::with_theme(&theme)
        .with_prompt("DB username")
        .default(default_user.to_string())
        .interact()?;

    // Input password (optional)
    let password: String = Password::with_theme(&theme)
        .with_prompt("Password (Optional)")
        .allow_empty_password(true)
        .interact()?;
    let password = if password.is_empty() {
        None
    } else {
        Some(password)
    };

    // Input database name (optional)
    let database: String = Input::with_theme(&theme)
        .with_prompt("Database name(Optional)")
        .allow_empty(true)
        .interact()?;
    let database = if database.is_empty() {
        None
    } else {
        Some(database)
    };

    // Input port number (optional)
    let port_str: String = Input::with_theme(&theme)
        .with_prompt("Port number (Optional)")
        .allow_empty(true)
        .interact()?;
    let port = if port_str.is_empty() {
        None
    } else {
        match port_str.parse::<u16>() {
            Ok(p) => Some(p),
            Err(_) => {
                println!("Warning: Invalid port number provided, using default port.");
                None
            }
        }
    };

    // Create connection information
    let connection = DatabaseConnection {
        db_type,
        container,
        user,
        password,
        database,
        port,
        options: None,
    };

    Ok((alias, connection))
}

/// Select or input container interactively
async fn get_container_interactively(theme: &ColorfulTheme) -> Result<String> {
    // Detect running database containers
    let detected_containers = DatabaseConnector::detect_database_containers().await?;

    if detected_containers.is_empty() {
        // Manual input if no containers detected
        println!("No database containers detected. Please enter manually.");
        let container: String = Input::with_theme(theme)
            .with_prompt("Docker container name")
            .interact()?;
        Ok(container)
    } else {
        // Select from detected containers
        let mut options: Vec<String> = detected_containers
            .iter()
            .map(|c| format!("{} ({} - {})", c.name, c.db_type, c.image))
            .collect();
        options.push("Enter manually...".to_string());

        let selection = Select::with_theme(theme)
            .with_prompt("Please select a detected database container")
            .items(&options)
            .default(0)
            .interact()?;

        if selection == options.len() - 1 {
            // Manual input selected
            let container: String = Input::with_theme(theme)
                .with_prompt("Docker container name")
                .interact()?;
            Ok(container)
        } else {
            // Selected detected container
            Ok(detected_containers[selection].name.clone())
        }
    }
}

/// Get connection information interactively (with auto-detection)
pub async fn get_connection_with_auto_detect() -> Result<(String, DatabaseConnection)> {
    let theme = ColorfulTheme::default();

    // Detect running database containers
    let detected_containers = DatabaseConnector::detect_database_containers().await?;

    if detected_containers.is_empty() {
        println!("No database containers detected.");
        return get_connection_interactively().await;
    }

    // Select from detected containers
    let mut options: Vec<String> = detected_containers
        .iter()
        .map(|c| format!("{} ({} - {})", c.name, c.db_type, c.image))
        .collect();
    options.push("Enter manually...".to_string());

    let selection = Select::with_theme(&theme)
        .with_prompt("Please select a detected database container")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == options.len() - 1 {
        // Manual input selected
        return get_connection_interactively().await;
    }

    // Use information from detected container
    let selected_container = &detected_containers[selection];

    // Input alias name
    let alias: String = Input::with_theme(&theme)
        .with_prompt("alias name")
        .default(selected_container.name.clone())
        .interact()?;

    // Get default connection information
    let defaults = DatabaseConnector::get_container_default_connection(
        &selected_container.name,
        &selected_container.db_type,
    )
    .await?;

    // Input username
    let default_user =
        defaults
            .get("user")
            .cloned()
            .unwrap_or_else(|| match selected_container.db_type {
                DatabaseType::PostgreSQL => "postgres".to_string(),
                DatabaseType::MySQL => "root".to_string(),
                DatabaseType::MongoDB => "mongo".to_string(),
            });
    let user: String = Input::with_theme(&theme)
        .with_prompt("DB username")
        .default(default_user)
        .interact()?;

    // Input password
    let password = if let Some(default_password) = defaults.get("password") {
        println!("Password detected from environment variable");
        let use_default = Select::with_theme(&theme)
            .with_prompt("Use password from environment variable?")
            .items(&["Yes", "No (enter new password)"])
            .default(0)
            .interact()?;

        if use_default == 0 {
            Some(default_password.clone())
        } else {
            let password: String = Password::with_theme(&theme)
                .with_prompt("Password")
                .allow_empty_password(true)
                .interact()?;
            if password.is_empty() {
                None
            } else {
                Some(password)
            }
        }
    } else {
        let password: String = Password::with_theme(&theme)
            .with_prompt("Password (Optional)")
            .allow_empty_password(true)
            .interact()?;
        if password.is_empty() {
            None
        } else {
            Some(password)
        }
    };

    // Input database name
    let default_database = defaults.get("database").cloned().unwrap_or_default();
    let database: String = Input::with_theme(&theme)
        .with_prompt("Database name (Optional)")
        .default(default_database)
        .allow_empty(true)
        .interact()?;
    let database = if database.is_empty() {
        None
    } else {
        Some(database)
    };

    // Input port number
    let port_str: String = Input::with_theme(&theme)
        .with_prompt("Port number (Optional)")
        .allow_empty(true)
        .interact()?;
    let port = if port_str.is_empty() {
        None
    } else {
        match port_str.parse::<u16>() {
            Ok(p) => Some(p),
            Err(_) => {
                println!("Warning: Invalid port number provided, using default port.");
                None
            }
        }
    };

    // Create connection information
    let connection = DatabaseConnection {
        db_type: selected_container.db_type.clone(),
        container: selected_container.name.clone(),
        user,
        password,
        database,
        port,
        options: None,
    };

    Ok((alias, connection))
}
