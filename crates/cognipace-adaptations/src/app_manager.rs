//! App Availability Manager - controls app access based on cognitive state

use cognipace_core::types::{AttentionState, AdaptationDecision, AdaptationType, AppCategory};
use cognipace_core::error::Result;
use tracing::info;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashSet;

/// App availability mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAvailabilityMode {
    /// All apps available
    Normal,
    /// Distracting apps blocked
    Focused,
    /// Only essential apps available
    Strict,
    /// Emergency - only current task
    Emergency,
}

/// App availability manager
pub struct AppAvailabilityManager {
    mode: AppAvailabilityMode,
    blocked_apps: HashSet<String>,
    allowed_apps: HashSet<String>,
    enabled: bool,
}

impl AppAvailabilityManager {
    pub fn new(enabled: bool, blocked_apps: Vec<String>) -> Self {
        Self {
            mode: AppAvailabilityMode::Normal,
            blocked_apps: blocked_apps.into_iter().collect(),
            allowed_apps: HashSet::new(),
            enabled,
        }
    }

    /// Determine app availability based on cognitive state
    pub fn adapt(&mut self, state: &AttentionState, load_score: f64) -> AdaptationDecision {
        if !self.enabled {
            return self.create_decision(AdaptationType::Normal, state, load_score, "App blocking disabled");
        }

        let (new_mode, blocked) = match state {
            AttentionState::DeepFocus | AttentionState::FlowBuilding => {
                // Block distracting apps
                (AppAvailabilityMode::Focused, self.get_distracting_apps())
            }
            AttentionState::Fragmented => {
                // Block more aggressively
                (AppAvailabilityMode::Focused, self.get_distracting_apps())
            }
            AttentionState::Fatigued => {
                // Only allow essential apps
                (AppAvailabilityMode::Strict, self.get_non_essential_apps())
            }
            AttentionState::Overloaded => {
                // Emergency - block everything except current app
                (AppAvailabilityMode::Emergency, self.get_all_apps())
            }
            AttentionState::Recovering => {
                // Gradually restore access
                (AppAvailabilityMode::Normal, HashSet::new())
            }
        };

        let adaptation_type = if blocked.is_empty() {
            AdaptationType::Normal
        } else {
            AdaptationType::BlockApps(blocked.iter().cloned().collect())
        };

        let reason = format!(
            "App mode changed from {:?} to {:?} - {} apps blocked",
            self.mode, new_mode, blocked.len()
        );

        let decision = self.create_decision(adaptation_type, state, load_score, &reason);
        
        if self.mode != new_mode {
            info!("App availability mode changed: {:?} -> {:?}", self.mode, new_mode);
            self.mode = new_mode;
            self.blocked_apps = blocked;
        }

        decision
    }

    /// Check if an app should be blocked
    pub fn is_app_blocked(&self, app_name: &str) -> bool {
        self.blocked_apps.contains(app_name)
    }

    /// Get distracting app categories
    fn get_distracting_apps(&self) -> HashSet<String> {
        // These would be determined by app category in production
        HashSet::from([
            "twitter".to_string(),
            "facebook".to_string(),
            "instagram".to_string(),
            "reddit".to_string(),
            "youtube".to_string(),
            "netflix".to_string(),
        ])
    }

    fn get_non_essential_apps(&self) -> HashSet<String> {
        let mut apps = self.get_distracting_apps();
        apps.extend([
            "news".to_string(),
            "email".to_string(),  // Batch mode instead
        ]);
        apps
    }

    fn get_all_apps(&self) -> HashSet<String> {
        // In production, this would query running apps
        let mut apps = self.get_distracting_apps();
        apps.extend([
            "slack".to_string(),
            "teams".to_string(),
            "discord".to_string(),
        ]);
        apps
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

    pub fn current_mode(&self) -> &AppAvailabilityMode {
        &self.mode
    }
}
