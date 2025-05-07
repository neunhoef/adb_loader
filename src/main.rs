mod config;

use std::path::PathBuf;
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
    println!("Endpoints: {:?}", config.endpoints);
    println!("Username: {}", config.username);
    println!("Prefix: {}", config.prefix);
    println!("Metrics port: {}", config.metrics_port);
    println!("\nActive use cases:");
    println!("CRUD: {} ({} threads)", config.active_usecases.crud.on, config.active_usecases.crud.threads);
    println!("Graph: {} ({} threads)", config.active_usecases.graph.on, config.active_usecases.graph.threads);
    
    Ok(())
}
