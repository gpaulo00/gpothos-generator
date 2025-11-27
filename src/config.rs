use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration loaded from .gpothosrc.json
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Enable automatic scanning of directories for manual resolvers
    #[serde(default = "default_true")]
    pub auto_scan: bool,
    
    /// Directories to scan for manual resolvers (relative to project root)
    #[serde(default)]
    pub scan_dirs: Vec<String>,
    
    /// Enable verbose output during scanning
    #[serde(default)]
    pub verbose: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_scan: true,
            scan_dirs: Vec::new(),
            verbose: false,
        }
    }
}

impl Config {
    /// Load configuration from .gpothosrc.json file
    /// Returns default config if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = ".gpothosrc.json";
        
        if !Path::new(config_path).exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&content)?;
        
        Ok(config)
    }
}
