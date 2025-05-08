use reqwest::Client;
use crate::config::DatabaseConfig;
use thiserror::Error;
use serde_json::json;

#[derive(Debug, Error)]
pub enum ArangoError {
    #[error("Database already exists: {0}")]
    DatabaseExists(String),
    #[error("Database does not exist: {0}")]
    DatabaseNotFound(String),
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Creates an async HTTP client configured for ArangoDB communication
/// 
/// # Returns
/// A configured reqwest Client that can be used for async HTTP requests
pub async fn create_client() -> Client {
    // Create a client with default settings
    // We'll add more configuration options as needed
    Client::new()
}

/// Creates a new database in ArangoDB
/// 
/// # Arguments
/// * `client` - The HTTP client to use for the request
/// * `config` - The database configuration containing connection details
/// * `db_name` - The name of the database to create
/// 
/// # Returns
/// Result indicating success or failure
/// 
/// # Errors
/// * `ArangoError::DatabaseExists` - If the database already exists
/// * `ArangoError::RequestError` - If the HTTP request fails
/// * `ArangoError::InvalidResponse` - If the response cannot be parsed
pub async fn create_database(
    client: &Client,
    config: &DatabaseConfig,
    db_name: &str,
) -> Result<(), ArangoError> {
    let endpoint = format!("{}/_api/database", config.endpoints[0]);
    
    let response = client
        .post(&endpoint)
        .json(&json!({
            "name": db_name
        }))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        
        // Check if the error is due to database already existing
        if status.as_u16() == 409 && error_text.contains("duplicate") {
            return Err(ArangoError::DatabaseExists(db_name.to_string()));
        }
        
        Err(ArangoError::InvalidResponse(format!(
            "Failed to create database: {} - {}",
            status, error_text
        )))
    }
}

/// Deletes a database from ArangoDB
/// 
/// # Arguments
/// * `client` - The HTTP client to use for the request
/// * `config` - The database configuration containing connection details
/// * `db_name` - The name of the database to delete
/// 
/// # Returns
/// Result indicating success or failure
/// 
/// # Errors
/// * `ArangoError::DatabaseNotFound` - If the database doesn't exist
/// * `ArangoError::RequestError` - If the HTTP request fails
/// * `ArangoError::InvalidResponse` - If the response cannot be parsed
pub async fn drop_database(
    client: &Client,
    config: &DatabaseConfig,
    db_name: &str,
) -> Result<(), ArangoError> {
    let endpoint = format!("{}/_api/database/{}", config.endpoints[0], db_name);
    
    let response = client
        .delete(&endpoint)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        
        // Check if the error is due to database not existing
        if status.as_u16() == 404 {
            return Err(ArangoError::DatabaseNotFound(db_name.to_string()));
        }
        
        Err(ArangoError::InvalidResponse(format!(
            "Failed to delete database: {} - {}",
            status, error_text
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> DatabaseConfig {
        DatabaseConfig {
            endpoints: vec!["http://localhost:8529".to_string()],
            username: "root".to_string(),
            password: "".to_string(),
            prefix: "test_".to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_and_drop_database() {
        let config = create_test_config();
        let client = create_client().await;
        let db_name = "test_db_creation";

        // First creation should succeed
        let result = create_database(&client, &config, db_name).await;
        assert!(result.is_ok(), "First database creation should succeed");

        // Second creation should fail with DatabaseExists
        let result = create_database(&client, &config, db_name).await;
        match result {
            Err(ArangoError::DatabaseExists(name)) => {
                assert_eq!(name, db_name, "Error should contain the correct database name");
            }
            _ => panic!("Second creation should fail with DatabaseExists error"),
        }

        // Drop the database
        let result = drop_database(&client, &config, db_name).await;
        assert!(result.is_ok(), "Database deletion should succeed");

        // Try to drop it again, should fail with DatabaseNotFound
        let result = drop_database(&client, &config, db_name).await;
        match result {
            Err(ArangoError::DatabaseNotFound(name)) => {
                assert_eq!(name, db_name, "Error should contain the correct database name");
            }
            _ => panic!("Second deletion should fail with DatabaseNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_create_database_empty_name() {
        let config = create_test_config();
        let client = create_client().await;

        // Creation with empty name should fail
        let result = create_database(&client, &config, "").await;
        assert!(result.is_err(), "Creating database with empty name should fail");
        
        // Verify it's not a DatabaseExists error
        match result {
            Err(ArangoError::DatabaseExists(_)) => {
                panic!("Empty name should not result in DatabaseExists error");
            }
            Err(ArangoError::DatabaseNotFound(_)) => {
                panic!("Empty name should not result in DatabaseNotFound error");
            }
            Err(ArangoError::InvalidResponse(_)) => {
                // This is the expected error type
            }
            Err(ArangoError::RequestError(_)) => {
                // This is also acceptable
            }
            Ok(()) => {
                panic!("Empty name should not result in Ok()");
            }
        }
    }

    #[tokio::test]
    async fn test_drop_nonexistent_database() {
        let config = create_test_config();
        let client = create_client().await;
        let db_name = "nonexistent_test_db";

        // Try to drop a non-existent database
        let result = drop_database(&client, &config, db_name).await;
        match result {
            Err(ArangoError::DatabaseNotFound(name)) => {
                assert_eq!(name, db_name, "Error should contain the correct database name");
            }
            Ok(()) => panic!("Dropping non-existent database should fail"),
            Err(e) => panic!("Expected DatabaseNotFound error, got: {:?}", e),
        }
    }
} 