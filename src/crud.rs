use std::thread;
use anyhow::Result;
use crate::config::{CrudConfig, DatabaseConfig};
use log::info;

/// Runs the CRUD use case with the given configuration.
/// Currently just prints a hello world message and sleeps forever.
pub fn run(crud_config: CrudConfig, db_config: DatabaseConfig) -> Result<()> {
    info!("Starting CRUD use case with configuration:");
    info!("Database endpoints: {:?}", db_config.endpoints);
    info!("Database prefix: {}", db_config.prefix);
    info!("Number of collections: {}", crud_config.number_of_collections);
    info!("Number of shards: {}", crud_config.number_of_shards);
    info!("Replication factor: {}", crud_config.replication_factor);
    info!("Number of documents: {}", crud_config.number_of_documents);
    info!("Document size: {}", crud_config.document_size);
    info!("Drop first: {}", crud_config.drop_first);
    
    // Sleep forever
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
} 