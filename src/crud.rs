use crate::arangodb::{
    collection_exists, create_client, create_collection, create_database, database_exists,
    drop_database,
};
use crate::config::{CrudConfig, DatabaseConfig, UseCaseConfig};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use log::info;
use rand::distr::{Alphanumeric, SampleString};
use rand::{rng, Rng};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Builder;

fn generate_random_ascii(length: usize) -> String {
    // Uses thread_rng as the random number generator
    // and Alphanumeric as the distribution of characters
    Alphanumeric.sample_string(&mut rng(), length)
}

/// Runs the CRUD use case with the given configuration.
/// Sets up a tokio runtime with the configured number of threads and executes the async code.
pub fn run(
    crud_config: CrudConfig,
    db_config: DatabaseConfig,
    usecase_config: UseCaseConfig,
) -> Result<()> {
    info!("Starting CRUD use case with configuration:");
    info!("Database endpoints: {:?}", db_config.endpoints);
    info!("Database prefix: {}", db_config.prefix);
    info!(
        "Number of collections: {}",
        crud_config.number_of_collections
    );
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

/// Generates a random document with the specified approximate size and number of attributes.
/// The document will have a _key field, a number field, a bool field, and additional
/// string attributes to reach the desired size.
fn generate_document(key: u32, target_size: u32, num_attributes: u32) -> HashMap<String, Value> {
    let mut doc = HashMap::new();

    // Add the _key field
    doc.insert("_key".to_string(), json!(format!("K{}", key)));

    // Add random number and boolean
    let mut rng = rng();
    doc.insert("number".to_string(), json!(rng.random::<i32>()));
    doc.insert("bool".to_string(), json!(rng.random::<bool>()));

    // Calculate approximate size per attribute
    // Subtract size of _key, number, and bool fields (rough estimate)
    let remaining_size = target_size.saturating_sub(50);
    let size_per_attr = remaining_size / num_attributes;

    // Add string attributes
    for i in 1..=num_attributes {
        let attr_name = format!("a{}", i);
        let attr_value = generate_random_ascii(size_per_attr as usize);
        doc.insert(attr_name, json!(attr_value));
    }

    doc
}

/// Inserts documents into a collection in batches using concurrent requests
async fn insert_documents(
    client: &reqwest::Client,
    db_config: &DatabaseConfig,
    insert_concurrency: u32,
    db_name: &str,
    collection_name: &str,
    num_documents: u32,
    document_size: u32,
    num_attributes: u32,
) -> anyhow::Result<()> {
    const BATCH_SIZE: u32 = 1000;

    // Create a shared client reference
    let client = Arc::new(client.clone());
    let endpoints: Arc<Vec<String>> = Arc::new(
        db_config
            .endpoints
            .iter()
            .map(|ep| format!("{}/_db/{}/_api/document/{}", ep, db_name, collection_name))
            .collect(),
    );

    // Create a stream of batch ranges
    let batches = (1..=num_documents)
        .step_by(BATCH_SIZE as usize)
        .map(|start| {
            let end = (start + BATCH_SIZE - 1).min(num_documents);
            (start, end)
        });

    // Process batches concurrently with a buffer of insert_concurrency:
    stream::iter(batches)
        .map(|(batch_start, batch_end)| {
            let client = Arc::clone(&client);
            let endpoints = Arc::clone(&endpoints);

            async move {
                let mut batch = Vec::new();
                for i in batch_start..=batch_end {
                    let doc = generate_document(i, document_size, num_attributes);
                    batch.push(doc);
                }

                let mut rng = rng();
                let index = rng.random_range(0..endpoints.len());
                let endpoint = &endpoints[index];
                let response = client.post(endpoint).json(&batch).send().await?;

                if !response.status().is_success() {
                    let error_status = response.status();
                    let error_text = response.text().await?;
                    return Err(anyhow::anyhow!(
                        "Failed to insert documents: {} - {}",
                        error_status,
                        error_text
                    ));
                }

                info!(
                    "Inserted documents {} to {} into collection {}",
                    batch_start, batch_end, collection_name
                );

                Ok::<_, anyhow::Error>(())
            }
        })
        .buffer_unordered(insert_concurrency as usize)
        .collect::<Vec<anyhow::Result<()>>>()
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<()>>>()?;

    Ok(())
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
        // If not dropping first, check if all collections exist
        if !crud_config.drop_first {
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
        }

        // Drop the database if we're either dropping first or collections don't exist
        info!("Dropping database {}", db_name);
        drop_database(client, db_config, &db_name).await?;
    }

    // At this point, either the database didn't exist or we just dropped it
    // Create the database
    info!("Creating database {}", db_name);
    create_database(client, db_config, &db_name).await?;

    // Create all collections and insert documents
    for i in 1..=crud_config.number_of_collections {
        let coll_name = format!("c{}", i);
        create_collection(
            client,
            db_config,
            &db_name,
            &coll_name,
            crud_config.number_of_shards,
            crud_config.replication_factor,
        )
        .await?;

        // Insert documents into the collection
        // Use 5 attributes by default to reach the desired document size
        insert_documents(
            client,
            db_config,
            crud_config.insert_concurrency,
            &db_name,
            &coll_name,
            crud_config.number_of_documents,
            crud_config.document_size,
            5,
        )
        .await?;
    }

    Ok(false)
}

/// The actual async implementation of the CRUD use case.
async fn run_async(crud_config: CrudConfig, db_config: DatabaseConfig) -> anyhow::Result<()> {
    let client = create_client().await;

    let database_existed =
        initialize_database_and_collections(&client, &db_config, &crud_config).await?;
    info!(
        "Database initialization complete. Database existed: {}",
        database_existed
    );

    // TODO: Implement the actual CRUD operations here
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
