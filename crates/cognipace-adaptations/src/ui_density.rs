//! UI Density Controller - adapts interface complexity based on cognitive capacity

use cognipace_core::types::{AttentionState, AdaptationDecision, AdaptationType};
use cognipace_core::error::Result;
use tracing::info;
use chrono::Utc;
use uuid::Uuid;

/// UI density levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIDensity {
    /// Full detail, rich visualizations, advanced options
    Full,
    /// Standard view, key information highlighted
    Standard,
    /// Minimal view, only essential actions
    Minimal,
    /// Emergency mode: single task, everything else hidden
    Emergency,
}

impl UIDensity {
    pub fn description(&self) -> &'static str {
        match self {
            UIDensity::Full => "Full detail with all metrics and options",
            UIDensity::Standard => "Standard view with main metrics visible",
            UIDensity::Minimal => "Minimal view showing only top 3 items",
            UIDensity::Emergency => "Emergency mode - current task only",
        }
    }
}

/// UI Density Controller
pub struct UIDensityController {
    current_density: UIDensity,
    enabled: bool,
}

impl UIDensityController {
    pub fn new(enabled: bool) -> Self {
        Self {
            current_density: UIDensity::Full,
            enabled,
        }
    }

    /// Determine appropriate UI density based on cognitive state
    pub fn adapt(&mut self, state: &AttentionState, load_score: f64) -> (UIDensity, AdaptationDecision) {
        if !self.enabled {
            return (UIDensity::Full, self.create_decision(AdaptationType::Normal, state, load_score, "UI adaptation disabled"));
        }

        let new_density = match state {
            AttentionState::DeepFocus | AttentionState::FlowBuilding => {
                // User is focused, can handle full UI
                UIDensity::Full
            }
            AttentionState::Fragmented => {
                // Reduce complexity to help focus
                UIDensity::Standard
            }
            AttentionState::Fatigued => {
                // Simplify to reduce cognitive burden
                UIDensity::Minimal
            }
            AttentionState::Overloaded => {
                // Emergency simplification
                UIDensity::Emergency
            }
            AttentionState::Recovering => {
                // Gradually reintroduce complexity
                UIDensity::Standard
            }
        };

        let adaptation_type = match new_density {
            UIDensity::Full => AdaptationType::Normal,
            UIDensity::Standard => AdaptationType::ReduceUIDensity,
            UIDensity::Minimal => AdaptationType::MinimalUI,
            UIDensity::Emergency => AdaptationType::EmergencyUI,
        };

        let reason = format!(
            "UI density changed from {:?} to {:?} due to {:?}",
            self.current_density, new_density, state
        );

        let decision = self.create_decision(adaptation_type, state, load_score, &reason);
        
        if self.current_density != new_density {
            info!("UI density changed: {:?} -> {:?}", self.current_density, new_density);
            self.current_density = new_density;
        }

        (new_density, decision)
    }

    pub fn current_density(&self) -> UIDensity {
        self.current_density
    }

    fn create_decision(
        &self,
        adaptation_type: AdaptationType,
        state: &AttentionState,
        load_score: f64,
        reason: &str,
    ) -> AdaptationDecision {
        AdaptationDecision {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            adaptation_type,
            reason: reason.to_string(),
            cognitive_state: *state,
            load_score,
            user_override: false,
        }
    }
}
