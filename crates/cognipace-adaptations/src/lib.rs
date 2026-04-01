//! CogniPace Adaptations - Adaptation mechanisms
//!
//! This crate provides the adaptation layer that modifies system behavior
//! based on cognitive state predictions.

pub mod notification;
pub mod ui_density;
pub mod app_manager;
pub mod break_scheduler;
pub mod engine;

pub use notification::*;
pub use ui_density::*;
pub use app_manager::*;
pub use break_scheduler::*;
pub use engine::*;
