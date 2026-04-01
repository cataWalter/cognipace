//! Sensor manager - coordinates all sensors

use cognipace_core::types::{SensorData, SensorConfig, FeatureVector};
use cognipace_core::error::Result;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::traits::Sensor;
use crate::eye_tracker::{EyeTracker, EyeTrackerConfig};
use crate::keystroke::{KeystrokeSensor, KeystrokeConfig};
use crate::app_usage::{AppUsageSensor, AppUsageConfig};

/// Feature aggregator - combines sensor data into feature vectors
pub struct FeatureAggregator {
    gaze_data: Vec<f64>,
    keystroke_data: Vec<(f64, f64)>,  // (flight_time, dwell_time)
    app_switches: u32,
    last_app: String,
    window_start: std::time::Instant,
    window_duration_secs: u64,
}

impl FeatureAggregator {
    pub fn new(window_duration_secs: u64) -> Self {
        Self {
            gaze_data: Vec::new(),
            keystroke_data: Vec::new(),
            app_switches: 0,
            last_app: String::new(),
            window_start: std::time::Instant::now(),
            window_duration_secs,
        }
    }

    /// Process incoming sensor data
    pub fn process_sensor_data(&mut self, data: &SensorData) {
        match data {
            SensorData::Gaze(gaze) => {
                self.gaze_data.push(gaze.x);
                self.gaze_data.push(gaze.y);
            }
            SensorData::Keystroke(ks) => {
                self.keystroke_data.push((ks.flight_time, ks.dwell_time));
            }
            SensorData::AppUsage(app) => {
                if app.app_name != self.last_app && !self.last_app.is_empty() {
                    self.app_switches += 1;
                }
                self.last_app = app.app_name.clone();
            }
            _ => {}
        }
    }

    /// Check if window is complete and extract features
    pub fn extract_features(&mut self) -> Option<FeatureVector> {
        let elapsed = self.window_start.elapsed().as_secs();
        if elapsed < self.window_duration_secs {
            return None;
        }

        let features = self.compute_features();
        self.reset();
        Some(features)
    }

    fn compute_features(&self) -> FeatureVector {
        let mut features = FeatureVector::new(chrono::Utc::now());

        // Calculate gaze variance
        if self.gaze_data.len() >= 2 {
            let x_values: Vec<f64> = self.gaze_data.iter().step_by(2).copied().collect();
            let mean_x: f64 = x_values.iter().sum::<f64>() / x_values.len() as f64;
            let variance: f64 = x_values.iter()
                .map(|x| (x - mean_x).powi(2))
                .sum::<f64>() / x_values.len() as f64;
            features.gaze_variance = variance.min(1.0);
        }

        // Calculate keystroke metrics
        if !self.keystroke_data.is_empty() {
            let avg_flight: f64 = self.keystroke_data.iter()
                .map(|(f, _)| f)
                .sum::<f64>() / self.keystroke_data.len() as f64;
            let avg_dwell: f64 = self.keystroke_data.iter()
                .map(|(_, d)| d)
                .sum::<f64>() / self.keystroke_data.len() as f64;
            
            // Typing speed (words per minute estimate)
            features.typing_speed = (60.0 / (avg_flight / 1000.0) * 5.0).min(200.0);
            features.typing_hesitation = avg_flight;
        }

        // App switching rate (per minute)
        let minutes = self.window_duration_secs as f64 / 60.0;
        features.app_switching_rate = self.app_switches as f64 / minutes;

        features.time_of_day = chrono::Utc::now().hour() as f64 / 24.0;

        features
    }

    fn reset(&mut self) {
        self.gaze_data.clear();
        self.keystroke_data.clear();
        self.app_switches = 0;
        self.window_start = std::time::Instant::now();
    }
}

/// Sensor manager that coordinates all sensors
pub struct SensorManager {
    sensors: HashMap<String, Box<dyn Sensor>>,
    aggregator: Arc<RwLock<FeatureAggregator>>,
    sender: Option<mpsc::Sender<FeatureVector>>,
    handles: Vec<JoinHandle<()>>,
    running: bool,
}

impl SensorManager {
    pub fn new(config: &SensorConfig) -> Self {
        let mut sensors = HashMap::new();

        if config.eye_tracking.enabled {
            sensors.insert(
                "eye_tracker".to_string(),
                Box::new(EyeTracker::new(EyeTrackerConfig {
                    enabled: true,
                    sample_rate_hz: config.eye_tracking.sample_rate,
                })) as Box<dyn Sensor>,
            );
        }

        if config.keystroke.enabled {
            sensors.insert(
                "keystroke".to_string(),
                Box::new(KeystrokeSensor::new(KeystrokeConfig {
                    enabled: true,
                })) as Box<dyn Sensor>,
            );
        }

        if config.app_usage.enabled {
            sensors.insert(
                "app_usage".to_string(),
                Box::new(AppUsageSensor::new(AppUsageConfig {
                    enabled: true,
                    sample_rate_hz: config.app_usage.sample_rate,
                })) as Box<dyn Sensor>,
            );
        }

        Self {
            sensors,
            aggregator: Arc::new(RwLock::new(FeatureAggregator::new(30))), // 30 second windows
            sender: None,
            handles: Vec::new(),
            running: false,
        }
    }

    /// Initialize all sensors
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing sensor manager with {} sensors", self.sensors.len());
        
        for (name, sensor) in self.sensors.iter_mut() {
            info!("Initializing sensor: {}", name);
            sensor.initialize().await?;
        }

        Ok(())
    }

    /// Start all sensors and begin feature aggregation
    pub async fn start(&mut self, sender: mpsc::Sender<FeatureVector>) -> Result<()> {
        if self.running {
            return Ok(());
        }

        self.sender = Some(sender);
        self.running = true;

        let (sensor_tx, mut sensor_rx) = mpsc::channel::<SensorData>(100);
        let aggregator = self.aggregator.clone();

        // Start feature extraction task
        let feature_sender = self.sender.as_ref().unwrap().clone();
        let extract_handle = tokio::spawn(async move {
            while let Some(data) = sensor_rx.recv().await {
                let mut agg = aggregator.write().await;
                agg.process_sensor_data(&data);
                
                if let Some(features) = agg.extract_features() {
                    if feature_sender.send(features).await.is_err() {
                        warn!("Feature receiver dropped");
                        break;
                    }
                }
            }
        });
        self.handles.push(extract_handle);

        // Start all sensors
        for (name, sensor) in self.sensors.iter_mut() {
            info!("Starting sensor: {}", name);
            let tx = sensor_tx.clone();
            
            // Clone sender for each sensor task
            let sensor_tx_clone = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = sensor.start(sensor_tx_clone).await {
                    error!("Failed to start sensor {}: {}", name, e);
                }
            });
        }

        info!("All sensors started");
        Ok(())
    }

    /// Stop all sensors
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping all sensors");
        
        for (name, sensor) in self.sensors.iter_mut() {
            if let Err(e) = sensor.stop().await {
                error!("Error stopping sensor {}: {}", name, e);
            }
        }

        self.running = false;
        Ok(())
    }

    /// Shutdown all sensors and cleanup
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down sensor manager");
        
        for (name, sensor) in self.sensors.iter_mut() {
            if let Err(e) = sensor.shutdown().await {
                error!("Error shutting down sensor {}: {}", name, e);
            }
        }

        for handle in self.handles.drain(..) {
            handle.abort();
        }

        self.running = false;
        Ok(())
    }

    /// Get list of available sensors
    pub fn available_sensors(&self) -> Vec<&str> {
        self.sensors.keys().map(|s| s.as_str()).collect()
    }

    /// Check if sensor manager is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}
