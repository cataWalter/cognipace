//! CogniPace CLI - Command-line interface for real-time cognitive state display

use cognipace_core::types::EngineConfig;
use cognipace_core::config::load_config;
use cognipace_core::database::Database;
use cognipace_adaptations::AdaptationEngine;
use cognipace_sensors::SensorManager;
use clap::Parser;
use tokio::sync::mpsc;
use tracing::{info, error};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod tui;

/// CogniPace CLI - Cognitive Load & Attention Operating System
#[derive(Parser, Debug)]
#[command(name = "cognipace-cli")]
#[command(about = "Monitor and adapt to your cognitive state in real-time")]
struct Args {
    /// Show current cognitive state once and exit
    #[arg(short, long)]
    once: bool,

    /// Enable specific sensors (comma-separated)
    #[arg(short, long, default_value = "all")]
    sensors: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Export stored data
    #[arg(long)]
    export: bool,

    /// Purge all stored data
    #[arg(long)]
    purge: bool,

    /// Show sensor status
    #[arg(long)]
    sensor_status: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
        )
        .init();

    // Handle special commands
    if args.purge {
        return handle_purge().await;
    }

    if args.export {
        return handle_export().await;
    }

    if args.sensor_status {
        return handle_sensor_status().await;
    }

    // Load configuration
    let config = load_config()?;
    info!("Configuration loaded");

    // Run the main application
    if args.once {
        run_once(&config).await
    } else {
        run_tui(&config).await
    }
}

async fn run_once(config: &EngineConfig) -> anyhow::Result<()> {
    println!("CogniPace - Cognitive State Snapshot");
    println!("=====================================");
    println!();
    println!("Sensors:");
    println!("  Eye Tracking: {}", if config.sensors.eye_tracking.enabled { "Enabled" } else { "Disabled" });
    println!("  Keystroke: {}", if config.sensors.keystroke.enabled { "Enabled" } else { "Disabled" });
    println!("  App Usage: {}", if config.sensors.app_usage.enabled { "Enabled" } else { "Disabled" });
    println!();
    println!("Adaptations:");
    println!("  Notifications: {}", if config.adaptations.notifications.enabled { "Enabled" } else { "Disabled" });
    println!("  UI Density: {}", if config.adaptations.ui_density.enabled { "Enabled" } else { "Disabled" });
    println!("  App Blocking: {}", if config.adaptations.app_blocking.enabled { "Enabled" } else { "Disabled" });
    println!("  Breaks: {}", if config.adaptations.breaks.enabled { "Enabled" } else { "Disabled" });
    println!();
    println!("Privacy:");
    println!("  Data Retention: {} days", config.privacy.data_retention_days);
    println!("  Telemetry: {}", if config.privacy.telemetry { "Enabled" } else { "Disabled" });
    
    Ok(())
}

async fn run_tui(config: &EngineConfig) -> anyhow::Result<()> {
    // Initialize sensor manager
    let mut sensor_manager = SensorManager::new(&config.sensors);
    sensor_manager.initialize().await?;

    // Initialize adaptation engine
    let mut engine = AdaptationEngine::new(config.clone());
    
    // Try to initialize database
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cognipace")
        .join("cognipace.db");
    
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    engine = engine.with_database(&db_path).await?;

    // Create channel for features
    let (tx, mut rx) = mpsc::channel::<cognipace_core::types::FeatureVector>(32);

    // Start sensors
    sensor_manager.start(tx).await?;

    // Run TUI
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    tui::run_tui(&mut rx, &mut engine, running).await?;

    // Cleanup
    sensor_manager.shutdown().await?;
    
    Ok(())
}

async fn handle_purge() -> anyhow::Result<()> {
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cognipace")
        .join("cognipace.db");
    
    if db_path.exists() {
        let db = Database::new(&db_path, 0)?;
        db.purge_old_data()?;
        std::fs::remove_file(&db_path)?;
        println!("All data purged successfully");
    } else {
        println!("No data to purge");
    }
    
    Ok(())
}

async fn handle_export() -> anyhow::Result<()> {
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cognipace")
        .join("cognipace.db");
    
    if db_path.exists() {
        let db = Database::new(&db_path, 365)?;
        let states = db.get_recent_states(100)?;
        
        println!("{{\"cognitive_states\": [");
        for (i, state) in states.iter().enumerate() {
            let json = serde_json::to_string(state)?;
            if i < states.len() - 1 {
                println!("  {},", json);
            } else {
                println!("  {}", json);
            }
        }
        println!("]}}");
    } else {
        println!("No data to export");
    }
    
    Ok(())
}

async fn handle_sensor_status() -> anyhow::Result<()> {
    println!("Sensor Status:");
    println!("==============");
    println!("Eye Tracking: Available (webcam required)");
    println!("Keystroke: Available (accessibility permissions required)");
    println!("App Usage: Available");
    println!();
    println!("Note: Sensors require appropriate OS permissions to function");
    Ok(())
}
