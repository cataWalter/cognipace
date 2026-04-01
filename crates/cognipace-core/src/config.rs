//! Configuration management for CogniPace

use crate::types::EngineConfig;
use crate::error::{CognipaceError, Result};
use std::path::PathBuf;
use std::fs;
use tracing::info;

/// Default configuration file path
pub fn default_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("cognipace");
    path.push("config.yaml");
    path
}

/// Load configuration from file or return defaults
pub fn load_config() -> Result<EngineConfig> {
    let config_path = default_config_path();
    
    if config_path.exists() {
        info!("Loading configuration from {:?}", config_path);
        let content = fs::read_to_string(&config_path)
            .map_err(|e| CognipaceError::ConfigError(format!("Failed to read config: {}", e)))?;
        
        let config: EngineConfig = serde_yaml::from_str(&content)
            .map_err(|e| CognipaceError::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    } else {
        info!("No configuration file found, using defaults");
        Ok(EngineConfig::default())
    }
}

/// Save configuration to file
pub fn save_config(config: &EngineConfig) -> Result<()> {
    let config_path = default_config_path();
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CognipaceError::ConfigError(format!("Failed to create config directory: {}", e)))?;
    }
    
    let content = serde_yaml::to_string(config)
        .map_err(|e| CognipaceError::ConfigError(format!("Failed to serialize config: {}", e)))?;
    
    fs::write(&config_path, content)
        .map_err(|e| CognipaceError::ConfigError(format!("Failed to write config: {}", e)))?;
    
    info!("Configuration saved to {:?}", config_path);
    Ok(())
}

/// Reset configuration to defaults
pub fn reset_config() -> Result<()> {
    let config_path = default_config_path();
    
    if config_path.exists() {
        fs::remove_file(&config_path)
            .map_err(|e| CognipaceError::ConfigError(format!("Failed to delete config: {}", e)))?;
        
        info!("Configuration reset to defaults");
    }
    
    Ok(())
}
