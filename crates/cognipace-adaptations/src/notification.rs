//! Notification router - adapts notification delivery based on cognitive state

use cognipace_core::types::{AttentionState, AdaptationDecision, AdaptationType, NotificationMode};
use cognipace_core::error::Result;
use tracing::{info, debug, warn};
use chrono::Utc;
use uuid::Uuid;
use std::collections::VecDeque;

/// Notification priority levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationPriority {
    /// Critical - must be delivered immediately
    Critical,
    /// High - from important contacts or time-sensitive
    High,
    /// Normal - regular notifications
    Normal,
    /// Low - newsletters, updates, etc.
    Low,
}

/// A notification to be routed
#[derive(Debug, Clone)]
pub struct PendingNotification {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub source: String,
    pub priority: NotificationPriority,
    pub timestamp: chrono::DateTime<Utc>,
}

/// Notification router that adapts delivery based on cognitive state
pub struct NotificationRouter {
    mode: NotificationMode,
    queued: VecDeque<PendingNotification>,
    batch_size: usize,
    enabled: bool,
}

impl NotificationRouter {
    pub fn new(mode: NotificationMode, enabled: bool) -> Self {
        Self {
            mode,
            queued: VecDeque::new(),
            batch_size: 5,
            enabled,
        }
    }

    /// Route a notification based on current cognitive state
    pub fn route_notification(
        &mut self,
        notification: PendingNotification,
        state: &AttentionState,
    ) -> AdaptationDecision {
        if !self.enabled {
            return self.create_decision(AdaptationType::Normal, state, "Notifications disabled");
        }

        match state {
            AttentionState::DeepFocus => {
                // Queue all non-critical notifications
                if notification.priority != NotificationPriority::Critical {
                    self.queued.push_back(notification);
                    self.create_decision(AdaptationType::QueueNotifications, state, 
                        "Deep focus detected - notification queued")
                } else {
                    self.deliver_immediate(&notification);
                    self.create_decision(AdaptationType::PriorityNotifications, state,
                        "Critical notification delivered during deep focus")
                }
            }
            AttentionState::FlowBuilding => {
                // Queue low priority, deliver others
                if notification.priority == NotificationPriority::Low {
                    self.queued.push_back(notification);
                    self.create_decision(AdaptationType::QueueNotifications, state,
                        "Flow building - low priority notification queued")
                } else {
                    self.deliver_immediate(&notification);
                    self.create_decision(AdaptationType::Normal, state,
                        "Notification delivered during flow building")
                }
            }
            AttentionState::Fragmented => {
                // Batch and summarize
                self.queued.push_back(notification);
                if self.queued.len() >= self.batch_size {
                    self.deliver_batch();
                }
                self.create_decision(AdaptationType::BatchNotifications, state,
                    "Fragmented attention - notification batched")
            }
            AttentionState::Fatigued => {
                // Block non-urgent
                if notification.priority == NotificationPriority::Critical 
                    || notification.priority == NotificationPriority::High {
                    self.deliver_immediate(&notification);
                    self.create_decision(AdaptationType::PriorityNotifications, state,
                        "Fatigued state - high priority notification delivered")
                } else {
                    self.queued.push_back(notification);
                    self.create_decision(AdaptationType::BlockNotifications, state,
                        "Fatigued state - non-urgent notification blocked")
                }
            }
            AttentionState::Overloaded => {
                // Block all non-critical
                if notification.priority == NotificationPriority::Critical {
                    self.deliver_immediate(&notification);
                    self.create_decision(AdaptationType::PriorityNotifications, state,
                        "Overloaded - critical notification only")
                } else {
                    self.queued.push_back(notification);
                    self.create_decision(AdaptationType::BlockNotifications, state,
                        "Overloaded - notification blocked")
                }
            }
            AttentionState::Recovering => {
                // Deliver priority only, queue rest
                if notification.priority == NotificationPriority::High 
                    || notification.priority == NotificationPriority::Critical {
                    self.deliver_immediate(&notification);
                    self.create_decision(AdaptationType::PriorityNotifications, state,
                        "Recovering - priority notification delivered")
                } else {
                    self.queued.push_back(notification);
                    self.create_decision(AdaptationType::QueueNotifications, state,
                        "Recovering - notification queued for later")
                }
            }
        }
    }

    /// Deliver queued notifications when state improves
    pub fn deliver_queued(&mut self) -> Vec<PendingNotification> {
        let mut delivered = Vec::new();
        while let Some(notification) = self.queued.pop_front() {
            delivered.push(notification);
        }
        delivered
    }

    fn deliver_immediate(&self, notification: &PendingNotification) {
        info!("Delivering notification: {} - {}", notification.source, notification.title);
        // In production, this would call OS notification API
    }

    fn deliver_batch(&mut self) {
        let batch_size = self.batch_size.min(self.queued.len());
        let batch: Vec<_> = self.queued.drain(..batch_size).collect();
        
        if !batch.is_empty() {
            let summary = batch.iter()
                .map(|n| format!("{}: {}", n.source, n.title))
                .collect::<Vec<_>>()
                .join("\n");
            
            info!("Delivering batch notification:\n{}", summary);
            // In production, deliver as a summarized notification
        }
    }

    fn create_decision(
        &self,
        adaptation_type: AdaptationType,
        state: &AttentionState,
        reason: &str,
    ) -> AdaptationDecision {
        AdaptationDecision {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            adaptation_type,
            reason: reason.to_string(),
            cognitive_state: *state,
            load_score: 0.0,  // Will be set by caller
            user_override: false,
        }
    }

    pub fn set_mode(&mut self, mode: NotificationMode) {
        self.mode = mode;
        info!("Notification mode changed to: {:?}", mode);
    }

    pub fn queued_count(&self) -> usize {
        self.queued.len()
    }
}
