//! Cognitive Load Model - Calculates cognitive load scores

use crate::types::{FeatureVector, LoadScore};
use tracing::debug;

/// Weights for cognitive load calculation
/// Based on Sweller's Cognitive Load Theory
pub struct CognitiveLoadWeights {
    pub intrinsic_weight: f64,
    pub extraneous_weight: f64,
    pub physiological_weight: f64,
}

impl Default for CognitiveLoadWeights {
    fn default() -> Self {
        Self {
            intrinsic_weight: 0.3,
            extraneous_weight: 0.5,
            physiological_weight: 0.2,
        }
    }
}

/// Cognitive Load Calculator
pub struct CognitiveLoadModel {
    weights: CognitiveLoadWeights,
}

impl CognitiveLoadModel {
    pub fn new() -> Self {
        Self {
            weights: CognitiveLoadWeights::default(),
        }
    }

    pub fn with_weights(weights: CognitiveLoadWeights) -> Self {
        Self { weights }
    }

    /// Calculate cognitive load score from feature vector
    /// 
    /// cognitive_load_score = (
    ///     intrinsic_load * 0.3 +      // Task complexity (from app context)
    ///     extraneous_load * 0.5 +     // Environmental factors (notifications, UI)
    ///     physiological_load * 0.2    // Body signals (HRV, posture, fatigue)
    /// )
    pub fn calculate_load(&self, features: &FeatureVector) -> LoadScore {
        // Intrinsic load: based on task complexity and typing speed (proxy for difficulty)
        let intrinsic_load = self.calculate_intrinsic_load(features);
        
        // Extraneous load: based on app switching, notifications, gaze variance
        let extraneous_load = self.calculate_extraneous_load(features);
        
        // Physiological load: based on HRV, posture, voice stress
        let physiological_load = self.calculate_physiological_load(features);

        let score = (
            intrinsic_load * self.weights.intrinsic_weight
            + extraneous_load * self.weights.extraneous_weight
            + physiological_load * self.weights.physiological_weight
        ).clamp(0.0, 1.0);

        debug!(
            "Cognitive load calculated: {:.2} (intrinsic: {:.2}, extraneous: {:.2}, physiological: {:.2})",
            score, intrinsic_load, extraneous_load, physiological_load
        );

        score
    }

    fn calculate_intrinsic_load(&self, features: &FeatureVector) -> f64 {
        // Task complexity is the primary indicator
        let task_complexity = features.task_complexity.clamp(0.0, 1.0);
        
        // Higher typing hesitation can indicate task difficulty
        let hesitation_factor = (features.typing_hesitation / 1000.0).clamp(0.0, 1.0);
        
        (task_complexity * 0.7 + hesitation_factor * 0.3).clamp(0.0, 1.0)
    }

    fn calculate_extraneous_load(&self, features: &FeatureVector) -> f64 {
        // High app switching indicates fragmentation
        let switching_factor = (features.app_switching_rate / 20.0).clamp(0.0, 1.0);
        
        // Notifications add cognitive burden
        let notification_factor = (features.notification_count / 10.0).clamp(0.0, 1.0);
        
        // Gaze variance indicates distraction
        let gaze_factor = features.gaze_variance.clamp(0.0, 1.0);
        
        // Typing error rate indicates frustration
        let error_factor = features.typing_error_rate.clamp(0.0, 1.0);

        (switching_factor * 0.35 
            + notification_factor * 0.25 
            + gaze_factor * 0.25 
            + error_factor * 0.15).clamp(0.0, 1.0)
    }

    fn calculate_physiological_load(&self, features: &FeatureVector) -> f64 {
        let mut load = 0.0;
        let mut weight_sum = 0.0;

        // HRV (lower HRV = higher stress)
        if let Some(hrv) = features.hrv_score {
            // Invert HRV so higher = more load
            let hrv_load = (1.0 - hrv.clamp(0.0, 1.0));
            load += hrv_load * 0.4;
            weight_sum += 0.4;
        }

        // Posture (poor posture = fatigue)
        if let Some(posture) = features.posture_score {
            let posture_load = (1.0 - posture.clamp(0.0, 1.0));
            load += posture_load * 0.3;
            weight_sum += 0.3;
        }

        // Voice stress
        if let Some(voice) = features.voice_stress {
            load += voice.clamp(0.0, 1.0) * 0.3;
            weight_sum += 0.3;
        }

        // If no physiological data, return moderate load
        if weight_sum == 0.0 {
            return 0.5;
        }

        (load / weight_sum).clamp(0.0, 1.0)
    }

    /// Update weights based on user feedback
    pub fn adapt_weights(&mut self, feedback_score: f64) {
        // Simple gradient descent towards better weights
        let learning_rate = 0.01;
        let error = feedback_score - self.calculate_load_from_weights();
        
        self.weights.intrinsic_weight += learning_rate * error;
        self.weights.extraneous_weight += learning_rate * error;
        self.weights.physiological_weight += learning_rate * error;
        
        // Normalize weights to sum to 1.0
        let total = self.weights.intrinsic_weight 
            + self.weights.extraneous_weight 
            + self.weights.physiological_weight;
        
        self.weights.intrinsic_weight /= total;
        self.weights.extraneous_weight /= total;
        self.weights.physiological_weight /= total;
    }

    fn calculate_load_from_weights(&self) -> f64 {
        self.weights.intrinsic_weight * 0.5
            + self.weights.extraneous_weight * 0.7
            + self.weights.physiological_weight * 0.3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_low_cognitive_load() {
        let model = CognitiveLoadModel::new();
        let mut features = FeatureVector::new(Utc::now());
        features.task_complexity = 0.2;
        features.app_switching_rate = 1.0;
        features.notification_count = 0.0;
        features.gaze_variance = 0.1;
        features.typing_error_rate = 0.05;
        features.typing_hesitation = 100.0;
        features.hrv_score = Some(0.8);
        features.posture_score = Some(0.9);
        
        let score = model.calculate_load(&features);
        assert!(score < 0.4, "Low load scenario should score below 0.4, got {}", score);
    }

    #[test]
    fn test_high_cognitive_load() {
        let model = CognitiveLoadModel::new();
        let mut features = FeatureVector::new(Utc::now());
        features.task_complexity = 0.9;
        features.app_switching_rate = 15.0;
        features.notification_count = 8.0;
        features.gaze_variance = 0.8;
        features.typing_error_rate = 0.4;
        features.typing_hesitation = 800.0;
        features.hrv_score = Some(0.2);
        features.posture_score = Some(0.3);
        
        let score = model.calculate_load(&features);
        assert!(score > 0.6, "High load scenario should score above 0.6, got {}", score);
    }
}
