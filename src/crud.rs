use std::thread;
use anyhow::Result;
use crate::config::{CrudConfig, DatabaseConfig};

/// Runs the CRUD use case with the given configuration.
/// Currently just prints a hello world message and sleeps forever.
pub fn run(crud_config: CrudConfig, db_config: DatabaseConfig) -> Result<()> {
    println!("Starting CRUD use case with configuration:");
    println!("Database endpoints: {:?}", db_config.endpoints);
    println!("Database prefix: {}", db_config.prefix);
    println!("Number of collections: {}", crud_config.number_of_collections);
    println!("Number of shards: {}", crud_config.number_of_shards);
    println!("Replication factor: {}", crud_config.replication_factor);
    println!("Number of documents: {}", crud_config.number_of_documents);
    println!("Document size: {}", crud_config.document_size);
    println!("Drop first: {}", crud_config.drop_first);
    
    // Sleep forever
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
} 