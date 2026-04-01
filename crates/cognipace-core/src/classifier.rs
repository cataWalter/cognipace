//! Attention State Classifier - Classifies cognitive state from features

use crate::types::{AttentionState, FeatureVector, LoadScore};
use tracing::debug;

/// Thresholds for attention state classification
pub struct ClassificationThresholds {
    pub deep_focus_load: f64,
    pub flow_building_load: f64,
    pub fragmented_load: f64,
    pub fatigued_load: f64,
    pub overloaded_load: f64,
}

impl Default for ClassificationThresholds {
    fn default() -> Self {
        Self {
            deep_focus_load: 0.2,
            flow_building_load: 0.35,
            fragmented_load: 0.55,
            fatigued_load: 0.75,
            overloaded_load: 0.9,
        }
    }
}

/// Attention State Classifier
pub struct AttentionStateClassifier {
    thresholds: ClassificationThresholds,
}

impl AttentionStateClassifier {
    pub fn new() -> Self {
        Self {
            thresholds: ClassificationThresholds::default(),
        }
    }

    pub fn with_thresholds(thresholds: ClassificationThresholds) -> Self {
        Self { thresholds }
    }

    /// Classify attention state from feature vector and load score
    pub fn classify(&self, features: &FeatureVector, load_score: LoadScore) -> AttentionState {
        // Check for overload conditions first
        if self.is_overloaded(features, load_score) {
            return AttentionState::Overloaded;
        }

        // Check for fatigue indicators
        if self.is_fatigued(features) {
            return AttentionState::Fatigued;
        }

        // Check for fragmentation
        if self.is_fragmented(features) {
            return AttentionState::Fragmented;
        }

        // Check for deep focus
        if self.is_deep_focus(features, load_score) {
            return AttentionState::DeepFocus;
        }

        // Check for flow building
        if self.is_flow_building(features, load_score) {
            return AttentionState::FlowBuilding;
        }

        // Default to fragmented if uncertain
        AttentionState::Fragmented
    }

    fn is_overloaded(&self, features: &FeatureVector, load_score: LoadScore) -> bool {
        load_score >= self.thresholds.overloaded_load
            || (features.app_switching_rate > 20.0 && features.typing_error_rate > 0.5)
    }

    fn is_fatigued(&self, features: &FeatureVector) -> bool {
        // Low typing speed + poor posture + low HRV = fatigue
        let low_typing_speed = features.typing_speed < 30.0;
        let poor_posture = features.posture_score.map_or(false, |p| p < 0.3);
        let low_hrv = features.hrv_score.map_or(false, |h| h < 0.3);

        (low_typing_speed && poor_posture) || (low_typing_speed && low_hrv)
    }

    fn is_fragmented(&self, features: &FeatureVector) -> bool {
        features.app_switching_rate > 10.0
            || features.gaze_variance > 0.6
            || features.typing_error_rate > 0.3
    }

    fn is_deep_focus(&self, features: &FeatureVector, load_score: LoadScore) -> bool {
        load_score <= self.thresholds.deep_focus_load
            && features.app_switching_rate < 3.0
            && features.gaze_variance < 0.2
    }

    fn is_flow_building(&self, features: &FeatureVector, load_score: LoadScore) -> bool {
        load_score <= self.thresholds.flow_building_load
            && features.app_switching_rate < 5.0
            && features.gaze_variance < 0.4
    }
}

/// Load Predictor - forecasts future cognitive load
pub struct LoadPredictor {
    horizon_seconds: u64,
}

impl LoadPredictor {
    pub fn new(horizon_seconds: u64) -> Self {
        Self { horizon_seconds }
    }

    /// Predict future load based on current trend
    /// This is a simplified version - in production, this would use ML
    pub fn predict(&self, current_load: LoadScore, recent_loads: &[LoadScore]) -> LoadScore {
        if recent_loads.is_empty() {
            return current_load;
        }

        // Calculate trend
        let avg_recent: f64 = recent_loads.iter().sum::<f64>() / recent_loads.len() as f64;
        let trend = current_load - avg_recent;

        // Extrapolate with damping
        let damping = 0.5;
        let predicted = current_load + trend * damping;

        predicted.clamp(0.0, 1.0)
    }

    /// Calculate probability of overload within horizon
    pub fn overload_probability(&self, current_load: LoadScore, trend: f64) -> f64 {
        if trend <= 0.0 {
            return 0.0;
        }

        // Estimate time to overload
        let time_to_overload = (1.0 - current_load) / trend;
        
        if time_to_overload <= 0.0 {
            return 1.0;
        }

        // Probability decreases as time to overload increases
        let horizon_hours = self.horizon_seconds as f64 / 3600.0;
        (1.0 - (time_to_overload / horizon_hours)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_deep_focus_classification() {
        let classifier = AttentionStateClassifier::new();
        let mut features = FeatureVector::new(Utc::now());
        features.app_switching_rate = 1.0;
        features.gaze_variance = 0.1;
        features.typing_speed = 80.0;
        features.typing_error_rate = 0.02;

        let state = classifier.classify(&features, 0.15);
        assert_eq!(state, AttentionState::DeepFocus);
    }

    #[test]
    fn test_overloaded_classification() {
        let classifier = AttentionStateClassifier::new();
        let mut features = FeatureVector::new(Utc::now());
        features.app_switching_rate = 25.0;
        features.gaze_variance = 0.9;
        features.typing_speed = 40.0;
        features.typing_error_rate = 0.6;

        let state = classifier.classify(&features, 0.95);
        assert_eq!(state, AttentionState::Overloaded);
    }

    #[test]
    fn test_load_prediction() {
        let predictor = LoadPredictor::new(900); // 15 minutes
        let current = 0.5;
        let recent = vec![0.4, 0.45, 0.48];
        
        let predicted = predictor.predict(current, &recent);
        assert!(predicted >= current, "Predicted load should be >= current with upward trend");
    }
}
