//! Sensor trait definitions

use async_trait::async_trait;
use cognipace_core::types::SensorData;
use cognipace_core::error::Result;
use tokio::sync::mpsc;

/// Trait for all sensor implementations
#[async_trait]
pub trait Sensor: Send + Sync {
    /// Get the sensor name
    fn name(&self) -> &str;
    
    /// Check if the sensor is enabled
    fn is_enabled(&self) -> bool;
    
    /// Initialize the sensor
    async fn initialize(&mut self) -> Result<()>;
    
    /// Start the sensor and begin streaming data
    async fn start(&mut self, sender: mpsc::Sender<SensorData>) -> Result<()>;
    
    /// Stop the sensor
    async fn stop(&mut self) -> Result<()>;
    
    /// Shutdown and cleanup resources
    async fn shutdown(&mut self) -> Result<()>;
}

/// Sensor status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorStatus {
    /// Sensor is not initialized
    Uninitialized,
    /// Sensor is ready but not running
    Ready,
    /// Sensor is actively collecting data
    Running,
    /// Sensor has encountered an error
    Error(String),
}
