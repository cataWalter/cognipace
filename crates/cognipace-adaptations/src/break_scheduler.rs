//! Break Scheduler - schedules breaks based on ultradian rhythms and cognitive state

use cognipace_core::types::{AttentionState, AdaptationDecision, AdaptationType, BreakActivity};
use cognipace_core::error::Result;
use tracing::info;
use chrono::{Utc, Duration, DateTime};
use uuid::Uuid;
use std::time::Instant;

/// Break suggestion
#[derive(Debug, Clone)]
pub struct BreakSuggestion {
    pub activity: BreakActivity,
    pub duration_minutes: u64,
    pub urgency: BreakUrgency,
    pub reason: String,
}

/// Break urgency level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BreakUrgency {
    /// Soft suggestion - user can continue if in flow
    Soft,
    /// Firm suggestion - strongly recommended
    Firm,
    /// Enforced - break should be taken
    Enforced,
}

/// Break scheduler
pub struct BreakScheduler {
    enabled: bool,
    interval_minutes: u64,
    duration_minutes: u64,
    last_break: Option<DateTime<Utc>>,
    break_in_progress: bool,
    consecutive_focus_time: Duration,
    focus_start: Option<Instant>,
}

impl BreakScheduler {
    pub fn new(enabled: bool, interval_minutes: u64, duration_minutes: u64) -> Self {
        Self {
            enabled,
            interval_minutes,
            duration_minutes,
            last_break: None,
            break_in_progress: false,
            consecutive_focus_time: Duration::zero(),
            focus_start: None,
        }
    }

    /// Check if a break should be suggested
    pub fn check_break(&mut self, state: &AttentionState, load_score: f64) -> Option<(BreakSuggestion, AdaptationDecision)> {
        if !self.enabled {
            return None;
        }

        let now = Utc::now();
        
        // Track focus time
        if *state == AttentionState::DeepFocus || *state == AttentionState::FlowBuilding {
            if self.focus_start.is_none() {
                self.focus_start = Some(Instant::now());
            }
        } else {
            if let Some(start) = self.focus_start {
                self.consecutive_focus_time += start.elapsed();
                self.focus_start = None;
            }
        }

        // Check if it's time for a scheduled break
        let time_since_last_break = self.last_break
            .map(|last| now.signed_duration_since(last))
            .unwrap_or(Duration::minutes(self.interval_minutes as i64));

        let scheduled_break_due = time_since_last_break.num_minutes() >= self.interval_minutes as i64;

        // Determine if break is needed
        let (should_suggest, urgency, activity) = if self.break_in_progress {
            return None;  // Already in break
        } else if *state == AttentionState::Overloaded {
            (true, BreakUrgency::Enforced, BreakActivity::RestEyes)
        } else if *state == AttentionState::Fatigued {
            (true, BreakUrgency::Firm, BreakActivity::Walk)
        } else if scheduled_break_due {
            if *state == AttentionState::DeepFocus {
                // Don't interrupt deep focus
                (false, BreakUrgency::Soft, BreakActivity::Stretch)
            } else {
                (true, BreakUrgency::Soft, BreakActivity::Stretch)
            }
        } else if load_score > 0.7 {
            (true, BreakUrgency::Firm, BreakActivity::Breathe)
        } else {
            (false, BreakUrgency::Soft, BreakActivity::Hydrate)
        };

        if should_suggest {
            let reason = match urgency {
                BreakUrgency::Soft => format!("Scheduled break due ({} min interval)", self.interval_minutes),
                BreakUrgency::Firm => format!("High cognitive load detected ({:.0}%)", load_score * 100.0),
                BreakUrgency::Enforced => "Cognitive overload - break required".to_string(),
            };

            let suggestion = BreakSuggestion {
                activity,
                duration_minutes: self.duration_minutes,
                urgency: urgency.clone(),
                reason: reason.clone(),
            };

            let adaptation_type = match urgency {
                BreakUrgency::Soft | BreakUrgency::Firm => AdaptationType::SuggestBreak,
                BreakUrgency::Enforced => AdaptationType::EnforceBreak,
            };

            let decision = AdaptationDecision {
                id: Uuid::new_v4(),
                timestamp: now,
                adaptation_type,
                reason,
                cognitive_state: *state,
                load_score,
                user_override: false,
            };

            Some((suggestion, decision))
        } else {
            None
        }
    }

    /// Mark that a break has started
    pub fn start_break(&mut self) {
        self.break_in_progress = true;
        info!("Break started");
    }

    /// Mark that a break has ended
    pub fn end_break(&mut self) {
        self.break_in_progress = false;
        self.last_break = Some(Utc::now());
        self.consecutive_focus_time = Duration::zero();
        self.focus_start = None;
        info!("Break ended");
    }

    /// Get time until next scheduled break
    pub fn time_until_next_break(&self) -> Duration {
        if let Some(last) = self.last_break {
            let elapsed = Utc::now().signed_duration_since(last);
            let remaining = Duration::minutes(self.interval_minutes as i64) - elapsed;
            if remaining.num_seconds() > 0 {
                remaining
            } else {
                Duration::zero()
            }
        } else {
            Duration::minutes(self.interval_minutes as i64)
        }
    }

    pub fn is_break_in_progress(&self) -> bool {
        self.break_in_progress
    }
}
