//! App usage monitoring sensor
//!
//! Tracks active applications and window switching for cognitive load analysis.

use async_trait::async_trait;
use cognipace_core::types::{SensorData, AppUsageData, AppCategory};
use cognipace_core::error::{CognipaceError, Result};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, debug};
use chrono::Utc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::traits::{Sensor, SensorStatus};

/// App usage sensor configuration
#[derive(Debug, Clone)]
pub struct AppUsageConfig {
    pub enabled: bool,
    pub sample_rate_hz: u32,
}

impl Default for AppUsageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sample_rate_hz: 1,
        }
    }
}

/// App usage monitoring sensor
pub struct AppUsageSensor {
    config: AppUsageConfig,
    status: SensorStatus,
    running: Arc<AtomicBool>,
}

impl AppUsageSensor {
    pub fn new(config: AppUsageConfig) -> Self {
        Self {
            config,
            status: SensorStatus::Uninitialized,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Categorize an app by its name
    fn categorize_app(app_name: &str) -> AppCategory {
        let lower = app_name.to_lowercase();
        
        if lower.contains("code") || lower.contains("vim") || lower.contains("terminal") 
            || lower.contains("idea") || lower.contains("vscode") {
            AppCategory::Development
        } else if lower.contains("slack") || lower.contains("teams") || lower.contains("discord")
            || lower.contains("zoom") {
            AppCategory::Communication
        } else if lower.contains("chrome") || lower.contains("firefox") || lower.contains("safari")
            || lower.contains("edge") {
            AppCategory::Productivity
        } else if lower.contains("netflix") || lower.contains("youtube") || lower.contains("spotify")
            || lower.contains("steam") {
            AppCategory::Entertainment
        } else if lower.contains("twitter") || lower.contains("facebook") || lower.contains("instagram")
            || lower.contains("reddit") {
            AppCategory::Social
        } else if lower.contains("news") || lower.contains("cnn") || lower.contains("bbc") {
            AppCategory::News
        } else {
            AppCategory::Other
        }
    }
}

#[async_trait]
impl Sensor for AppUsageSensor {
    fn name(&self) -> &str {
        "app_usage"
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing app usage sensor");
        self.status = SensorStatus::Ready;
        info!("App usage sensor initialized");
        Ok(())
    }

    async fn start(&mut self, sender: mpsc::Sender<SensorData>) -> Result<()> {
        if self.status != SensorStatus::Ready {
            return Err(CognipaceError::SensorError("App usage sensor not initialized".to_string()));
        }

        info!("Starting app usage sensor at {} Hz", self.config.sample_rate_hz);
        self.status = SensorStatus::Running;
        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let sample_rate = self.config.sample_rate_hz;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(1000 / sample_rate as u64));
            let mut last_app = String::from("unknown");
            
            while running.load(Ordering::SeqCst) {
                interval.tick().await;
                
                // In production, this would query OS for active window
                // For now, simulate with sample data
                let app_name = "code".to_string();
                let window_title = "main.rs - Visual Studio Code".to_string();
                let category = Self::categorize_app(&app_name);
                
                let app_data = AppUsageData {
                    timestamp: Utc::now(),
                    app_name: app_name.clone(),
                    window_title,
                    is_active: true,
                    category,
                };

                if app_name != last_app {
                    debug!("Active app changed to: {}", app_name);
                    last_app = app_name;
                }
                
                if sender.send(SensorData::AppUsage(app_data)).await.is_err() {
                    warn!("App usage sensor: receiver dropped");
                    break;
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping app usage sensor");
        self.running.store(false, Ordering::SeqCst);
        self.status = SensorStatus::Ready;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down app usage sensor");
        self.running.store(false, Ordering::SeqCst);
        self.status = SensorStatus::Uninitialized;
        Ok(())
    }
}
