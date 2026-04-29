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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.auto_scan);
        assert!(config.scan_dirs.is_empty());
        assert!(!config.verbose);
    }

    #[test]
    fn test_deserialize_config_full() {
        let json = r#"{
            "autoScan": false,
            "scanDirs": ["src/types", "src/resolvers"],
            "verbose": true
        }"#;
        
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.auto_scan);
        assert_eq!(config.scan_dirs, vec!["src/types", "src/resolvers"]);
        assert!(config.verbose);
    }

    #[test]
    fn test_deserialize_config_partial() {
        let json = r#"{
            "scanDirs": ["src/types"]
        }"#;
        
        let config: Config = serde_json::from_str(json).unwrap();
        // auto_scan defaults to true
        assert!(config.auto_scan);
        assert_eq!(config.scan_dirs, vec!["src/types"]);
        // verbose defaults to false
        assert!(!config.verbose);
    }

    #[test]
    fn test_deserialize_config_empty() {
        let json = "{}";
        
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.auto_scan);
        assert!(config.scan_dirs.is_empty());
        assert!(!config.verbose);
    }

    #[test]
    fn test_serialize_config() {
        let config = Config {
            auto_scan: true,
            scan_dirs: vec!["src/types".to_string()],
            verbose: false,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"autoScan\":true"));
        assert!(json.contains("\"scanDirs\":[\"src/types\"]"));
        assert!(json.contains("\"verbose\":false"));
    }
}
