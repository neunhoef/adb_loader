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
    #[error("Collection already exists: {0}")]
    CollectionExists(String),
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

/// Checks if a database exists in ArangoDB
/// 
/// # Arguments
/// * `client` - The HTTP client to use for the request
/// * `config` - The database configuration containing connection details
/// * `db_name` - The name of the database to check
/// 
/// # Returns
/// Result containing a boolean indicating if the database exists
pub async fn database_exists(
    client: &Client,
    config: &DatabaseConfig,
    db_name: &str,
) -> Result<bool, ArangoError> {
    let endpoint = format!("{}/_db/{}/_api/database/current", config.endpoints[0], db_name);
    
    let response = client
        .get(&endpoint)
        .send()
        .await?;

    Ok(response.status().is_success())
}

/// Checks if a collection exists in a database
/// 
/// # Arguments
/// * `client` - The HTTP client to use for the request
/// * `config` - The database configuration containing connection details
/// * `db_name` - The name of the database containing the collection
/// * `collection_name` - The name of the collection to check
/// 
/// # Returns
/// Result containing a boolean indicating if the collection exists
pub async fn collection_exists(
    client: &Client,
    config: &DatabaseConfig,
    db_name: &str,
    collection_name: &str,
) -> Result<bool, ArangoError> {
    let endpoint = format!("{}/_db/{}/_api/collection/{}", config.endpoints[0], db_name, collection_name);
    
    let response = client
        .get(&endpoint)
        .send()
        .await?;

    Ok(response.status().is_success())
}

/// Creates a new collection in a database
/// 
/// # Arguments
/// * `client` - The HTTP client to use for the request
/// * `config` - The database configuration containing connection details
/// * `db_name` - The name of the database to create the collection in
/// * `collection_name` - The name of the collection to create
/// * `number_of_shards` - The number of shards for the collection
/// * `replication_factor` - The replication factor for the collection
/// 
/// # Returns
/// Result indicating success or failure
pub async fn create_collection(
    client: &Client,
    config: &DatabaseConfig,
    db_name: &str,
    collection_name: &str,
    number_of_shards: u32,
    replication_factor: u32,
) -> Result<(), ArangoError> {
    let endpoint = format!("{}/_db/{}/_api/collection", config.endpoints[0], db_name);
    
    let response = client
        .post(&endpoint)
        .json(&json!({
            "name": collection_name,
            "numberOfShards": number_of_shards,
            "replicationFactor": replication_factor
        }))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        
        // Check if the error is due to collection already existing
        if status.as_u16() == 409 && error_text.contains("duplicate") {
            return Err(ArangoError::CollectionExists(collection_name.to_string()));
        }
        
        Err(ArangoError::InvalidResponse(format!(
            "Failed to create collection: {} - {}",
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
            Err(ArangoError::CollectionExists(_)) => {
                panic!("Empty name should not result in CollectionExists error");
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

    #[tokio::test]
    async fn test_collection_creation_workflow() {
        let config = create_test_config();
        let client = create_client().await;
        let db_name = "test_collection_db";
        let collection_name = "test_collection";

        // Create the database first
        let result = create_database(&client, &config, db_name).await;
        assert!(result.is_ok(), "Database creation should succeed");

        // Verify collection doesn't exist initially
        let exists = collection_exists(&client, &config, db_name, collection_name).await;
        assert!(exists.is_ok(), "Collection existence check should succeed");
        assert!(!exists.unwrap(), "Collection should not exist initially");

        // Create the collection
        let result = create_collection(&client, &config, db_name, collection_name, 1, 1).await;
        assert!(result.is_ok(), "Collection creation should succeed");

        // Verify collection exists after creation
        let exists = collection_exists(&client, &config, db_name, collection_name).await;
        assert!(exists.is_ok(), "Collection existence check should succeed");
        assert!(exists.unwrap(), "Collection should exist after creation");

        // Clean up - drop the database
        let result = drop_database(&client, &config, db_name).await;
        assert!(result.is_ok(), "Database deletion should succeed");
    }

    #[tokio::test]
    async fn test_database_exists() {
        let config = create_test_config();
        let client = create_client().await;
        let db_name = "test_exists_db";

        // First check - database should not exist
        let exists = database_exists(&client, &config, db_name).await;
        assert!(exists.is_ok(), "Database existence check should succeed");
        assert!(!exists.unwrap(), "Database should not exist initially");

        // Create the database
        let result = create_database(&client, &config, db_name).await;
        assert!(result.is_ok(), "Database creation should succeed");

        // Second check - database should now exist
        let exists = database_exists(&client, &config, db_name).await;
        assert!(exists.is_ok(), "Database existence check should succeed");
        assert!(exists.unwrap(), "Database should exist after creation");

        // Clean up - drop the database
        let result = drop_database(&client, &config, db_name).await;
        assert!(result.is_ok(), "Database deletion should succeed");

        // Final check - database should not exist again
        let exists = database_exists(&client, &config, db_name).await;
        assert!(exists.is_ok(), "Database existence check should succeed");
        assert!(!exists.unwrap(), "Database should not exist after deletion");
    }
} 