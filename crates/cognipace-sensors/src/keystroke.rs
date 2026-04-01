//! Keystroke dynamics sensor implementation
//!
//! Captures keystroke timing (not content) for cognitive load analysis.
//! Uses OS-level hooks for cross-platform support.

use async_trait::async_trait;
use cognipace_core::types::{SensorData, KeystrokeData};
use cognipace_core::error::{CognipaceError, Result};
use tokio::sync::mpsc;
use tracing::{info, warn, debug};
use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::traits::{Sensor, SensorStatus};

/// Keystroke sensor configuration
#[derive(Debug, Clone)]
pub struct KeystrokeConfig {
    pub enabled: bool,
}

impl Default for KeystrokeConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Keystroke dynamics sensor
pub struct KeystrokeSensor {
    config: KeystrokeConfig,
    status: SensorStatus,
    running: Arc<AtomicBool>,
}

impl KeystrokeSensor {
    pub fn new(config: KeystrokeConfig) -> Self {
        Self {
            config,
            status: SensorStatus::Uninitialized,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if accessibility permissions are granted
    pub fn check_permission() -> Result<bool> {
        // In production, check OS-specific permissions
        Ok(true)
    }
}

#[async_trait]
impl Sensor for KeystrokeSensor {
    fn name(&self) -> &str {
        "keystroke"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing keystroke sensor");
        
        if !Self::check_permission()? {
            self.status = SensorStatus::Error("Accessibility permission denied".to_string());
            return Err(CognipaceError::PermissionDenied("Accessibility access required".to_string()));
        }
        
        self.status = SensorStatus::Ready;
        info!("Keystroke sensor initialized");
        Ok(())
    }

    async fn start(&mut self, sender: mpsc::Sender<SensorData>) -> Result<()> {
        if self.status != SensorStatus::Ready {
            return Err(CognipaceError::SensorError("Keystroke sensor not initialized".to_string()));
        }

        info!("Starting keystroke sensor");
        self.status = SensorStatus::Running;
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        
        tokio::spawn(async move {
            // In production, this would use OS-level hooks
            // For now, we simulate keystroke events
            let mut last_key_time = std::time::Instant::now();
            
            while running.load(Ordering::SeqCst) {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // Simulate keystroke data
                let now = std::time::Instant::now();
                let flight_time = now.duration_since(last_key_time).as_millis() as f64;
                last_key_time = now;
                
                let keystroke = KeystrokeData {
                    timestamp: Utc::now(),
                    key_down: 0.0,
                    key_up: 0.0,
                    flight_time,
                    dwell_time: 80.0 + (flight_time * 0.1),
                    is_error: false,
                };

                debug!("Keystroke: flight_time={:.1}ms, dwell_time={:.1}ms", 
                       keystroke.flight_time, keystroke.dwell_time);
                
                if sender.send(SensorData::Keystroke(keystroke)).await.is_err() {
                    warn!("Keystroke sensor: receiver dropped");
                    break;
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping keystroke sensor");
        self.running.store(false, Ordering::SeqCst);
        self.status = SensorStatus::Ready;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down keystroke sensor");
        self.running.store(false, Ordering::SeqCst);
        self.status = SensorStatus::Uninitialized;
        Ok(())
    }
}
