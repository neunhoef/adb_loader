mod config;
mod crud;

use std::path::PathBuf;
use std::thread;
use clap::Parser;
use anyhow::Result;
use serde_yaml;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let config = config::Config::from_file(&args.config)?;
    
    // Dump the configuration in YAML format
    println!("Loaded configuration (YAML format):");
    println!("{}", serde_yaml::to_string(&config)?);
    
    println!("\nConfiguration summary:");
    println!("Version: {}", config.version);
    println!("Endpoints: {:?}", config.database.endpoints);
    println!("Username: {}", config.database.username);
    println!("Prefix: {}", config.database.prefix);
    println!("Metrics port: {}", config.metrics_port);
    println!("\nActive use cases:");
    println!("CRUD: {} ({} threads)", config.active_usecases.crud.on, config.active_usecases.crud.threads);
    println!("Graph: {} ({} threads)", config.active_usecases.graph.on, config.active_usecases.graph.threads);

    // Start CRUD use case if enabled
    if config.active_usecases.crud.on {
        let crud_config = config.crud.clone();
        let db_config = config.database.clone();
        thread::spawn(move || {
            if let Err(e) = crud::run(crud_config, db_config) {
                eprintln!("CRUD use case failed: {}", e);
            }
        });
    }
    
    // Keep main thread alive
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
