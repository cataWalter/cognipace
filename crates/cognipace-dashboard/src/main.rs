//! CogniPace Dashboard - Tauri-based dashboard for cognitive state visualization

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cognipace_core::types::{EngineConfig, FeatureVector};
use cognipace_core::config::load_config;
use cognipace_adaptations::AdaptationEngine;
use cognipace_sensors::SensorManager;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tauri::State;

/// Application state shared with the frontend
pub struct AppState {
    pub engine: Mutex<AdaptationEngine>,
    pub sensor_manager: Mutex<Option<SensorManager>>,
    pub current_state: Mutex<Option<cognipace_core::types::CognitiveState>>,
}

/// Get current cognitive state
#[tauri::command]
async fn get_cognitive_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let current = state.current_state.lock().await;
    
    if let Some(ref cs) = *current {
        Ok(serde_json::to_value(cs).map_err(|e| e.to_string())?)
    } else {
        Ok(serde_json::json!({
            "status": "waiting",
            "message": "Waiting for sensor data..."
        }))
    }
}

/// Get system configuration
#[tauri::command]
async fn get_config() -> Result<serde_json::Value, String> {
    let config = load_config().map_err(|e| e.to_string())?;
    Ok(serde_json::to_value(config).map_err(|e| e.to_string())?)
}

/// Get database statistics
#[tauri::command]
async fn get_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let engine = state.engine.lock().await;
    
    if let Some(db) = engine.database() {
        let stats = db.get_stats().map_err(|e| e.to_string())?;
        Ok(serde_json::json!({
            "feature_count": stats.feature_count,
            "state_count": stats.state_count,
            "adaptation_count": stats.adaptation_count,
            "retention_days": stats.retention_days,
        }))
    } else {
        Ok(serde_json::json!({
            "feature_count": 0,
            "state_count": 0,
            "adaptation_count": 0,
            "retention_days": 0,
        }))
    }
}

/// Purge all data
#[tauri::command]
async fn purge_data(state: State<'_, AppState>) -> Result<(), String> {
    let engine = state.engine.lock().await;
    
    if let Some(db) = engine.database() {
        db.purge_old_data().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize in background
            let handle = app.handle();
            
            tauri::async_runtime::spawn(async move {
                initialize_system(handle).await;
            });
            
            Ok(())
        })
        .manage(AppState {
            engine: Mutex::new(AdaptationEngine::new(EngineConfig::default())),
            sensor_manager: Mutex::new(None),
            current_state: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_cognitive_state,
            get_config,
            get_stats,
            purge_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn initialize_system(handle: tauri::AppHandle) {
    // Load configuration
    let config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            EngineConfig::default()
        }
    };

    // Initialize sensor manager
    let mut sensor_manager = SensorManager::new(&config.sensors);
    if let Err(e) = sensor_manager.initialize().await {
        eprintln!("Failed to initialize sensors: {}", e);
        return;
    }

    // Initialize adaptation engine
    let mut engine = AdaptationEngine::new(config.clone());
    
    let db_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("cognipace")
        .join("cognipace.db");
    
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    
    if let Ok(e) = engine.with_database(&db_path).await {
        engine = e;
    }

    // Create channel for features
    let (tx, mut rx) = mpsc::channel::<FeatureVector>(32);

    // Start sensors
    if let Err(e) = sensor_manager.start(tx).await {
        eprintln!("Failed to start sensors: {}", e);
        return;
    }

    // Update app state
    {
        let state: State<'_, AppState> = handle.state();
        let mut engine_guard = state.engine.lock().await;
        *engine_guard = engine;
        
        let mut sm_guard = state.sensor_manager.lock().await;
        *sm_guard = Some(sensor_manager);
    }

    // Process features in a loop
    loop {
        if let Some(features) = rx.recv().await {
            let state: State<'_, AppState> = handle.state();
            let mut engine_guard = state.engine.lock().await;
            let cognitive_state = engine_guard.process(&features);
            
            let mut current_guard = state.current_state.lock().await;
            *current_guard = Some(cognitive_state);
        }
    }
}
