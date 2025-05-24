use regex::Regex;
use crate::error::AppError;

/// Validates container name to prevent command injection
pub fn validate_container_name(name: &str) -> Result<(), AppError> {
    let valid_pattern = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_.-]*$")
        .expect("Failed to compile regex");
    
    if !valid_pattern.is_match(name) {
        return Err(AppError::ValidationError(
            "Invalid container name. Only alphanumeric characters, dots, hyphens, and underscores are allowed".to_string()
        ));
    }
    
    if name.len() > 255 {
        return Err(AppError::ValidationError(
            "Container name is too long (max 255 characters)".to_string()
        ));
    }
    
    Ok(())
}

/// Validates database username to prevent command injection
pub fn validate_username(username: &str) -> Result<(), AppError> {
    let valid_pattern = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_.-]*$")
        .expect("Failed to compile regex");
    
    if !valid_pattern.is_match(username) {
        return Err(AppError::ValidationError(
            "Invalid username. Must start with a letter and contain only alphanumeric characters, dots, hyphens, and underscores".to_string()
        ));
    }
    
    if username.len() > 64 {
        return Err(AppError::ValidationError(
            "Username is too long (max 64 characters)".to_string()
        ));
    }
    
    Ok(())
}

/// Validates database name to prevent command injection
pub fn validate_database_name(name: &str) -> Result<(), AppError> {
    let valid_pattern = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$")
        .expect("Failed to compile regex");
    
    if !valid_pattern.is_match(name) {
        return Err(AppError::ValidationError(
            "Invalid database name. Must start with a letter and contain only alphanumeric characters and underscores".to_string()
        ));
    }
    
    if name.len() > 64 {
        return Err(AppError::ValidationError(
            "Database name is too long (max 64 characters)".to_string()
        ));
    }
    
    Ok(())
}

/// Sanitizes input for safe shell usage
pub fn sanitize_for_shell(input: &str) -> String {
    shell_escape::escape(input.into()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_container_names() {
        assert!(validate_container_name("mysql").is_ok());
        assert!(validate_container_name("postgres-db").is_ok());
        assert!(validate_container_name("mongo_test.1").is_ok());
        assert!(validate_container_name("db123").is_ok());
    }

    #[test]
    fn test_invalid_container_names() {
        assert!(validate_container_name("-invalid").is_err());
        assert!(validate_container_name("test space").is_err());
        assert!(validate_container_name("test;echo").is_err());
        assert!(validate_container_name("test$(whoami)").is_err());
        assert!(validate_container_name("").is_err());
    }

    #[test]
    fn test_valid_usernames() {
        assert!(validate_username("root").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("app.user").is_ok());
    }

    #[test]
    fn test_invalid_usernames() {
        assert!(validate_username("123user").is_err());
        assert!(validate_username("user space").is_err());
        assert!(validate_username("user;drop").is_err());
        assert!(validate_username("").is_err());
    }

    #[test]
    fn test_sanitize_shell() {
        assert_eq!(sanitize_for_shell("normal"), "normal");
        assert_eq!(sanitize_for_shell("test;echo"), "'test;echo'");
        assert_eq!(sanitize_for_shell("$(whoami)"), "'$(whoami)'");
    }
}