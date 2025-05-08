use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub version: String,
    pub database: DatabaseConfig,
    pub active_usecases: ActiveUseCases,
    pub metrics_port: u16,
    pub crud: CrudConfig,
    pub graph: GraphConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub endpoints: Vec<String>,
    pub username: String,
    pub password: String,
    pub prefix: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveUseCases {
    pub crud: UseCaseConfig,
    pub graph: UseCaseConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UseCaseConfig {
    pub on: bool,
    pub threads: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrudConfig {
    pub number_of_collections: u32,
    pub number_of_shards: u32,
    pub replication_factor: u32,
    pub number_of_documents: u32,
    pub document_size: u32,
    pub drop_first: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub insert_concurrency: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GraphConfig {
    pub number_of_vertices: u32,
    pub number_of_edges: u32,
    pub number_of_shards: u32,
    pub replication_factor: u32,
    pub smart: bool,
    pub vertex_size: u32,
    pub edge_size: u32,
    pub drop_first: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

