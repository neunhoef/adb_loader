use anyhow::Result;
use crate::config::{CrudConfig, DatabaseConfig, UseCaseConfig};
use crate::arangodb::{create_client, create_database, collection_exists, create_collection, database_exists, drop_database};
use log::info;
use tokio::runtime::Builder;

/// Runs the CRUD use case with the given configuration.
/// Sets up a tokio runtime with the configured number of threads and executes the async code.
pub fn run(crud_config: CrudConfig, db_config: DatabaseConfig, usecase_config: UseCaseConfig) -> Result<()> {
    info!("Starting CRUD use case with configuration:");
    info!("Database endpoints: {:?}", db_config.endpoints);
    info!("Database prefix: {}", db_config.prefix);
    info!("Number of collections: {}", crud_config.number_of_collections);
    info!("Number of shards: {}", crud_config.number_of_shards);
    info!("Replication factor: {}", crud_config.replication_factor);
    info!("Number of documents: {}", crud_config.number_of_documents);
    info!("Document size: {}", crud_config.document_size);
    info!("Drop first: {}", crud_config.drop_first);
    info!("Number of threads: {}", usecase_config.threads);

    // Create a multi-threaded runtime with the configured number of threads
    let runtime = Builder::new_multi_thread()
        .worker_threads(usecase_config.threads as usize)
        .enable_all()
        .build()?;

    // Run the async code
    runtime.block_on(run_async(crud_config, db_config))
}

/// Initializes the database and collections according to the configuration.
/// Returns true if the database already existed and had all collections.
async fn initialize_database_and_collections(
    client: &reqwest::Client,
    db_config: &DatabaseConfig,
    crud_config: &CrudConfig,
) -> anyhow::Result<bool> {
    let db_name = format!("{}{}", db_config.prefix, "crud");
    
    // First check if database exists
    if database_exists(client, db_config, &db_name).await? {
        // Database exists, check if all collections exist
        let mut all_collections_exist = true;
        for i in 1..=crud_config.number_of_collections {
            let coll_name = format!("c{}", i);
            if !collection_exists(client, db_config, &db_name, &coll_name).await? {
                all_collections_exist = false;
                break;
            }
        }

        if all_collections_exist {
            // Everything exists, we're done
            return Ok(true);
        }

        // Collections don't exist, drop the database
        drop_database(client, db_config, &db_name).await?;
    }

    // At this point, either the database didn't exist or we just dropped it
    // Create the database
    create_database(client, db_config, &db_name).await?;

    // Create all collections
    for i in 1..=crud_config.number_of_collections {
        let coll_name = format!("c{}", i);
        create_collection(
            client,
            db_config,
            &db_name,
            &coll_name,
            crud_config.number_of_shards,
            crud_config.replication_factor
        ).await?;
    }

    Ok(false)
}

/// The actual async implementation of the CRUD use case.
async fn run_async(crud_config: CrudConfig, db_config: DatabaseConfig) -> anyhow::Result<()> {
    let client = create_client().await;
    
    let database_existed = initialize_database_and_collections(&client, &db_config, &crud_config).await?;
    info!("Database initialization complete. Database existed: {}", database_existed);
    
    // TODO: Implement the actual CRUD operations here
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
} 