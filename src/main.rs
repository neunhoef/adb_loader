mod arangodb;
mod config;
mod crud;

use anyhow::Result;
use clap::Parser;
use log::{error, info};
use serde_yaml;
use std::path::PathBuf;
use std::thread;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    // Initialize the logger
    env_logger::init();

    let args = Args::parse();

    let config = config::Config::from_file(&args.config)?;

    // Dump the configuration in YAML format
    println!("Loaded configuration (YAML format):");
    println!("{}", serde_yaml::to_string(&config)?);

    info!("Configuration summary:");
    info!("Version: {}", config.version);
    info!("Endpoints: {:?}", config.database.endpoints);
    info!("Username: {}", config.database.username);
    info!("Prefix: {}", config.database.prefix);
    info!("Metrics port: {}", config.metrics_port);
    info!("Active use cases:");
    info!(
        "CRUD: {} ({} threads)",
        config.active_usecases.crud.on, config.active_usecases.crud.threads
    );
    info!(
        "Graph: {} ({} threads)",
        config.active_usecases.graph.on, config.active_usecases.graph.threads
    );

    // Start CRUD use case if enabled
    if config.active_usecases.crud.on {
        let crud_config = config.crud.clone();
        let db_config = config.database.clone();
        let usecase_config = config.active_usecases.crud.clone();
        thread::spawn(move || {
            if let Err(e) = crud::run(crud_config, db_config, usecase_config) {
                error!("CRUD use case failed: {}", e);
            }
        });
    }

    // Keep main thread alive
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
