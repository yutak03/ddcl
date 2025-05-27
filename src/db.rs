use std::collections::HashMap;
use std::process::Stdio;

use tokio::process::Command;

use crate::config::{DatabaseConnection, DatabaseType};
use crate::error::{AppError, Result};
use crate::validation::{validate_container_name, validate_database_name, validate_username};

/// Database connection abstraction
pub struct DatabaseConnector;

/// Detected database container information
#[derive(Debug, Clone)]
pub struct DetectedContainer {
    pub name: String,
    pub db_type: DatabaseType,
    pub image: String,
    pub ports: Vec<String>,
    pub status: String,
}

impl DatabaseConnector {
    /// Connect to the database
    pub async fn connect(connection: &DatabaseConnection) -> Result<()> {
        match connection.db_type {
            DatabaseType::PostgreSQL => Self::connect_postgresql(connection).await,
            DatabaseType::MySQL => Self::connect_mysql(connection).await,
            DatabaseType::MongoDB => Self::connect_mongodb(connection).await,
        }
    }

    /// Connect to PostgreSQL
    async fn connect_postgresql(connection: &DatabaseConnection) -> Result<()> {
        // Validate inputs
        validate_container_name(&connection.container)?;
        validate_username(&connection.user)?;
        if let Some(db) = &connection.database {
            validate_database_name(db)?;
        }

        let mut cmd = Command::new("docker");
        cmd.arg("exec")
            .arg("-it")
            .arg(&connection.container)
            .arg("psql");

        // Add database name (if specified)
        if let Some(db) = &connection.database {
            cmd.arg("-d").arg(db);
        }

        // Add username
        cmd.arg("-U").arg(&connection.user);

        // Add additional options if available
        if let Some(options) = &connection.options {
            for (key, value) in options {
                cmd.arg(format!("--{}", key)).arg(value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "Failed to connect to PostgreSQL container: {:?}",
                status
            )));
        }

        Ok(())
    }

    /// Connect to MySQL
    async fn connect_mysql(connection: &DatabaseConnection) -> Result<()> {
        // Validate inputs
        validate_container_name(&connection.container)?;
        validate_username(&connection.user)?;
        if let Some(db) = &connection.database {
            validate_database_name(db)?;
        }

        let mut cmd = Command::new("docker");
        cmd.arg("exec")
            .arg("-it")
            .arg(&connection.container)
            .arg("mysql");

        // Add database name (if specified)
        if let Some(db) = &connection.database {
            cmd.arg(db);
        }

        // Add username
        cmd.arg("-u").arg(&connection.user);

        // Add password (if specified)
        if let Some(password) = &connection.password {
            // Use -p flag with password directly (no space between -p and password)
            cmd.arg(format!("-p{}", password));
        }

        // Add additional options if available
        if let Some(options) = &connection.options {
            for (key, value) in options {
                cmd.arg(format!("--{}", key)).arg(value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "Failed to connect to MySQL container: {:?}",
                status
            )));
        }

        Ok(())
    }

    /// Connect to MongoDB
    async fn connect_mongodb(connection: &DatabaseConnection) -> Result<()> {
        // Validate inputs
        validate_container_name(&connection.container)?;
        if !connection.user.is_empty() {
            validate_username(&connection.user)?;
        }
        if let Some(db) = &connection.database {
            validate_database_name(db)?;
        }

        let mut cmd = Command::new("docker");
        cmd.arg("exec")
            .arg("-it")
            .arg(&connection.container)
            .arg("mongosh");

        // Add authentication credentials (if specified)
        if !connection.user.is_empty() {
            cmd.arg("-u").arg(&connection.user);

            if let Some(password) = &connection.password {
                cmd.arg("-p").arg(password);
            }
        }

        // Add database name (if specified)
        if let Some(db) = &connection.database {
            cmd.arg(db);
        }

        // Add additional options if available
        if let Some(options) = &connection.options {
            for (key, value) in options {
                cmd.arg(format!("--{}", key)).arg(value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "Failed to connect to MongoDB container: {:?}",
                status
            )));
        }

        Ok(())
    }

    /// Check if container is running
    pub async fn check_container(container_name: &str) -> Result<bool> {
        // Validate container name
        validate_container_name(container_name)?;

        let output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}")
            .output()
            .await?;

        if !output.status.success() {
            return Err(AppError::Docker(
                "Failed to retrieve Docker container list".to_string(),
            ));
        }

        let containers = String::from_utf8_lossy(&output.stdout);
        Ok(containers.lines().any(|name| name.trim() == container_name))
    }

    /// Auto-detect running database containers
    pub async fn detect_database_containers() -> Result<Vec<DetectedContainer>> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}\t{{.Image}}\t{{.Ports}}\t{{.Status}}")
            .output()
            .await?;

        if !output.status.success() {
            return Err(AppError::Docker(
                "Failed to retrieve Docker container list".to_string(),
            ));
        }

        let mut detected_containers = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);

        for line in output_str.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let image = parts[1].to_string();
                let ports: Vec<String> =
                    parts[2].split(',').map(|s| s.trim().to_string()).collect();
                let status = parts[3].to_string();

                // Determine database type from image name
                if let Some(db_type) = Self::detect_database_type(&image, &ports).await {
                    detected_containers.push(DetectedContainer {
                        name,
                        db_type,
                        image,
                        ports,
                        status,
                    });
                }
            }
        }

        Ok(detected_containers)
    }

    /// Determine database type from image name and ports
    async fn detect_database_type(image: &str, ports: &[String]) -> Option<DatabaseType> {
        let image_lower = image.to_lowercase();

        // Determine by image name
        if image_lower.contains("postgres") || image_lower.contains("postgresql") {
            return Some(DatabaseType::PostgreSQL);
        }
        if image_lower.contains("mysql") || image_lower.contains("mariadb") {
            return Some(DatabaseType::MySQL);
        }
        if image_lower.contains("mongo") {
            return Some(DatabaseType::MongoDB);
        }

        // Determine by port number
        for port in ports {
            if port.contains("5432") {
                return Some(DatabaseType::PostgreSQL);
            }
            if port.contains("3306") {
                return Some(DatabaseType::MySQL);
            }
            if port.contains("27017") {
                return Some(DatabaseType::MongoDB);
            }
        }

        None
    }

    /// Get environment variables from container and infer default connection info
    pub async fn get_container_default_connection(
        container_name: &str,
        db_type: &DatabaseType,
    ) -> Result<HashMap<String, String>> {
        // Validate container name
        validate_container_name(container_name)?;

        let output = Command::new("docker")
            .arg("exec")
            .arg(container_name)
            .arg("env")
            .output()
            .await?;

        if !output.status.success() {
            return Ok(HashMap::new());
        }

        let mut env_vars = HashMap::new();
        let output_str = String::from_utf8_lossy(&output.stdout);

        // Define allowed environment variables for security
        const ALLOWED_ENV_VARS: &[&str] = &[
            "POSTGRES_USER",
            "POSTGRESQL_USER",
            "POSTGRES_DB",
            "POSTGRESQL_DATABASE",
            "POSTGRES_PASSWORD",
            "POSTGRESQL_PASSWORD",
            "MYSQL_DATABASE",
            "MYSQL_ROOT_PASSWORD",
            "MYSQL_USER",
            "MYSQL_PASSWORD",
            "MONGO_INITDB_ROOT_USERNAME",
            "MONGO_INITDB_DATABASE",
            "MONGO_INITDB_ROOT_PASSWORD",
        ];

        for line in output_str.lines() {
            if let Some((key, value)) = line.split_once('=') {
                // Only store allowed environment variables
                if ALLOWED_ENV_VARS.contains(&key) {
                    env_vars.insert(key.to_string(), value.to_string());
                }
            }
        }

        let mut defaults = HashMap::new();

        match db_type {
            DatabaseType::PostgreSQL => {
                defaults.insert(
                    "user".to_string(),
                    env_vars
                        .get("POSTGRES_USER")
                        .or(env_vars.get("POSTGRESQL_USER"))
                        .unwrap_or(&"postgres".to_string())
                        .clone(),
                );
                defaults.insert(
                    "database".to_string(),
                    env_vars
                        .get("POSTGRES_DB")
                        .or(env_vars.get("POSTGRESQL_DATABASE"))
                        .unwrap_or(&"postgres".to_string())
                        .clone(),
                );
                if let Some(password) = env_vars
                    .get("POSTGRES_PASSWORD")
                    .or(env_vars.get("POSTGRESQL_PASSWORD"))
                {
                    defaults.insert("password".to_string(), password.clone());
                }
            }
            DatabaseType::MySQL => {
                defaults.insert("user".to_string(), "root".to_string());
                if let Some(db) = env_vars.get("MYSQL_DATABASE") {
                    defaults.insert("database".to_string(), db.clone());
                }
                if let Some(password) = env_vars.get("MYSQL_ROOT_PASSWORD") {
                    defaults.insert("password".to_string(), password.clone());
                }
            }
            DatabaseType::MongoDB => {
                defaults.insert(
                    "user".to_string(),
                    env_vars
                        .get("MONGO_INITDB_ROOT_USERNAME")
                        .unwrap_or(&"root".to_string())
                        .clone(),
                );
                defaults.insert(
                    "database".to_string(),
                    env_vars
                        .get("MONGO_INITDB_DATABASE")
                        .unwrap_or(&"admin".to_string())
                        .clone(),
                );
                if let Some(password) = env_vars.get("MONGO_INITDB_ROOT_PASSWORD") {
                    defaults.insert("password".to_string(), password.clone());
                }
            }
        }

        Ok(defaults)
    }
}
