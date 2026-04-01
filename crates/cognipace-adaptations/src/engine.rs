//! Adaptation Engine - coordinates all adaptation mechanisms

use cognipace_core::types::{
    AttentionState, CognitiveState, AdaptationDecision, FeatureVector,
    EngineConfig, LoadForecast,
};
use cognipace_core::model::CognitiveLoadModel;
use cognipace_core::classifier::{AttentionStateClassifier, LoadPredictor};
use cognipace_core::database::Database;
use cognipace_core::error::Result;
use tracing::{info, debug, warn};
use chrono::Utc;
use std::path::Path;

use crate::notification::NotificationRouter;
use crate::ui_density::UIDensityController;
use crate::app_manager::AppAvailabilityManager;
use crate::break_scheduler::BreakScheduler;

/// Main adaptation engine that coordinates all components
pub struct AdaptationEngine {
    config: EngineConfig,
    load_model: CognitiveLoadModel,
    classifier: AttentionStateClassifier,
    predictor: LoadPredictor,
    notification_router: NotificationRouter,
    ui_controller: UIDensityController,
    app_manager: AppAvailabilityManager,
    break_scheduler: BreakScheduler,
    database: Option<Database>,
    recent_loads: Vec<f64>,
}

impl AdaptationEngine {
    /// Create a new adaptation engine
    pub fn new(config: EngineConfig) -> Self {
        let window_size = config.model.window_size_seconds;
        
        Self {
            config: config.clone(),
            load_model: CognitiveLoadModel::new(),
            classifier: AttentionStateClassifier::new(),
            predictor: LoadPredictor::new(config.model.prediction_horizon_seconds),
            notification_router: NotificationRouter::new(
                config.adaptations.notifications.mode.clone(),
                config.adaptations.notifications.enabled,
            ),
            ui_controller: UIDensityController::new(config.adaptations.ui_density.enabled),
            app_manager: AppAvailabilityManager::new(
                config.adaptations.app_blocking.enabled,
                config.adaptations.app_blocking.blocked_apps.clone(),
            ),
            break_scheduler: BreakScheduler::new(
                config.adaptations.breaks.enabled,
                config.adaptations.breaks.interval_minutes,
                config.adaptations.breaks.duration_minutes,
            ),
            database: None,
            recent_loads: Vec::new(),
        }
    }

    /// Initialize the engine with database
    pub async fn with_database(mut self, db_path: &Path) -> Result<Self> {
        let retention_days = self.config.privacy.data_retention_days;
        self.database = Some(Database::new(db_path, retention_days)?);
        Ok(self)
    }

    /// Process feature vector and produce cognitive state with adaptations
    pub fn process(&mut self, features: &FeatureVector) -> CognitiveState {
        // Calculate cognitive load
        let load_score = self.load_model.calculate_load(features);
        
        // Track recent loads for prediction
        self.recent_loads.push(load_score);
        if self.recent_loads.len() > 10 {
            self.recent_loads.remove(0);
        }

        // Classify attention state
        let attention_state = self.classifier.classify(features, load_score);

        // Predict future load
        let forecast = if self.recent_loads.len() >= 2 {
            let predicted = self.predictor.predict(load_score, &self.recent_loads);
            let trend = load_score - self.recent_loads[0];
            let overload_prob = self.predictor.overload_probability(load_score, trend);
            
            Some(LoadForecast {
                predicted_load: predicted,
                confidence: 0.7,
                horizon_seconds: self.config.model.prediction_horizon_seconds,
                overload_probability: overload_prob,
            })
        } else {
            None
        };

        // Create cognitive state
        let mut state = CognitiveState::new(
            attention_state,
            load_score,
            features.task_complexity * 0.3,
            (features.app_switching_rate / 20.0 + features.notification_count / 10.0) * 0.5,
            features.hrv_score.map_or(0.5, |h| (1.0 - h) * 0.2),
        );
        state.forecast = forecast;

        // Apply adaptations
        self.apply_adaptations(&state, features);

        // Store in database
        if let Some(ref db) = self.database {
            if let Err(e) = db.store_feature_vector(features) {
                warn!("Failed to store feature vector: {}", e);
            }
            if let Err(e) = db.store_cognitive_state(&state) {
                warn!("Failed to store cognitive state: {}", e);
            }
        }

        state
    }

    fn apply_adaptations(&mut self, state: &CognitiveState, features: &FeatureVector) {
        debug!(
            "Applying adaptations for state: {:?}, load: {:.2}",
            state.attention_state, state.load_score
        );

        // Notification routing
        // UI density
        let (_, ui_decision) = self.ui_controller.adapt(&state.attention_state, state.load_score);
        self.log_decision(&ui_decision);

        // App availability
        let app_decision = self.app_manager.adapt(&state.attention_state, state.load_score);
        self.log_decision(&app_decision);

        // Break scheduling
        if let Some((suggestion, break_decision)) = self.break_scheduler.check_break(
            &state.attention_state,
            state.load_score,
        ) {
            info!(
                "Break suggestion: {:?} ({:?}) - {}",
                suggestion.activity, suggestion.urgency, suggestion.reason
            );
            self.log_decision(&break_decision);
        }
    }

    fn log_decision(&self, decision: &AdaptationDecision) {
        debug!(
            "Adaptation: {} - {} (load: {:.2}, state: {:?})",
            decision.adaptation_type,
            decision.reason,
            decision.load_score,
            decision.cognitive_state
        );

        if let Some(ref db) = self.database {
            if let Err(e) = db.store_adaptation_decision(decision) {
                warn!("Failed to store adaptation decision: {}", e);
            }
        }
    }

    /// Get the notification router
    pub fn notification_router(&self) -> &NotificationRouter {
        &self.notification_router
    }

    /// Get the UI controller
    pub fn ui_controller(&self) -> &UIDensityController {
        &self.ui_controller
    }

    /// Get the app manager
    pub fn app_manager(&self) -> &AppAvailabilityManager {
        &self.app_manager
    }

    /// Get the break scheduler
    pub fn break_scheduler(&self) -> &BreakScheduler {
        &self.break_scheduler
    }

    /// Get database reference
    pub fn database(&self) -> Option<&Database> {
        self.database.as_ref()
    }
}
