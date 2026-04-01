//! Core types for cognitive load measurement and adaptation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Cognitive load score (0.0 to 1.0)
pub type LoadScore = f64;

/// Attention state classifier output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionState {
    /// Deep focus - stable gaze, low app switching, fast typing
    DeepFocus,
    /// Flow building - increasing focus metrics, single app usage
    FlowBuilding,
    /// Fragmented - high app switching, gaze drift, errors
    Fragmented,
    /// Fatigued - slow typing, posture slump, HRV drop
    Fatigued,
    /// Overloaded - all signals elevated, rapid context switches
    Overloaded,
    /// Recovering - improving metrics post-break
    Recovering,
}

impl AttentionState {
    pub fn emoji(&self) -> &'static str {
        match self {
            AttentionState::DeepFocus => "🟢",
            AttentionState::FlowBuilding => "🟡",
            AttentionState::Fragmented => "🟠",
            AttentionState::Fatigued => "🔴",
            AttentionState::Overloaded => "⚫",
            AttentionState::Recovering => "🔵",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AttentionState::DeepFocus => "Deep focus detected. All systems optimized for concentration.",
            AttentionState::FlowBuilding => "Flow state building. Gradually reducing interruptions.",
            AttentionState::Fragmented => "Attention fragmented. Consider taking a break.",
            AttentionState::Fatigued => "Fatigue detected. Strongly recommend a break.",
            AttentionState::Overloaded => "Cognitive overload! Emergency simplification activated.",
            AttentionState::Recovering => "Recovery in progress. Gradually re-introducing stimuli.",
        }
    }

    pub fn recommended_action(&self) -> &'static str {
        match self {
            AttentionState::DeepFocus => "Continue current work. Notifications blocked.",
            AttentionState::FlowBuilding => "Maintain focus. Reducing interruptions.",
            AttentionState::Fragmented => "Suggest break. Batching notifications.",
            AttentionState::Fatigued => "Enforce break. Reducing complexity.",
            AttentionState::Overloaded => "Emergency mode. All non-essential blocked.",
            AttentionState::Recovering => "Gradual return to normal operations.",
        }
    }
}

impl std::fmt::Display for AttentionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AttentionState::DeepFocus => "Deep Focus",
            AttentionState::FlowBuilding => "Flow Building",
            AttentionState::Fragmented => "Fragmented",
            AttentionState::Fatigued => "Fatigued",
            AttentionState::Overloaded => "Overloaded",
            AttentionState::Recovering => "Recovering",
        };
        write!(f, "{} {}", self.emoji(), name)
    }
}

/// Feature vector extracted from sensor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub timestamp: DateTime<Utc>,
    pub gaze_variance: f64,
    pub reading_speed: f64,
    pub typing_speed: f64,
    pub typing_error_rate: f64,
    pub typing_hesitation: f64,
    pub app_switching_rate: f64,
    pub task_complexity: f64,
    pub notification_count: f64,
    pub hrv_score: Option<f64>,
    pub posture_score: Option<f64>,
    pub voice_stress: Option<f64>,
    pub time_of_day: f64,
}

impl FeatureVector {
    pub fn new(timestamp: DateTime<Utc>) -> Self {
        Self {
            timestamp,
            gaze_variance: 0.0,
            reading_speed: 0.0,
            typing_speed: 0.0,
            typing_error_rate: 0.0,
            typing_hesitation: 0.0,
            app_switching_rate: 0.0,
            task_complexity: 0.0,
            notification_count: 0.0,
            hrv_score: None,
            posture_score: None,
            voice_stress: None,
            time_of_day: 0.0,
        }
    }

    /// Convert to array for ML model input
    pub fn to_array(&self) -> Vec<f64> {
        vec![
            self.gaze_variance,
            self.reading_speed,
            self.typing_speed,
            self.typing_error_rate,
            self.typing_hesitation,
            self.app_switching_rate,
            self.task_complexity,
            self.notification_count,
            self.hrv_score.unwrap_or(0.5),
            self.posture_score.unwrap_or(0.5),
            self.voice_stress.unwrap_or(0.5),
            self.time_of_day,
        ]
    }
}

/// Cognitive state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveState {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub attention_state: AttentionState,
    pub load_score: LoadScore,
    pub intrinsic_load: f64,
    pub extraneous_load: f64,
    pub physiological_load: f64,
    pub forecast: Option<LoadForecast>,
}

impl CognitiveState {
    pub fn new(
        attention_state: AttentionState,
        load_score: LoadScore,
        intrinsic_load: f64,
        extraneous_load: f64,
        physiological_load: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            attention_state,
            load_score,
            intrinsic_load,
            extraneous_load,
            physiological_load,
            forecast: None,
        }
    }
}

/// Load forecast for future time horizon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadForecast {
    pub predicted_load: LoadScore,
    pub confidence: f64,
    pub horizon_seconds: u64,
    pub overload_probability: f64,
}

/// Adaptation decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationDecision {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub adaptation_type: AdaptationType,
    pub reason: String,
    pub cognitive_state: AttentionState,
    pub load_score: LoadScore,
    pub user_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdaptationType {
    /// Queue notifications for later delivery
    QueueNotifications,
    /// Batch and summarize notifications
    BatchNotifications,
    /// Block all non-urgent notifications
    BlockNotifications,
    /// Allow priority notifications only
    PriorityNotifications,
    /// Reduce UI density
    ReduceUIDensity,
    /// Simplify UI to minimal view
    MinimalUI,
    /// Emergency single-task mode
    EmergencyUI,
    /// Block distracting apps
    BlockApps(Vec<String>),
    /// Suggest break
    SuggestBreak,
    /// Enforce break
    EnforceBreak,
    /// Summarize content
    SummarizeContent,
    /// Normal operation
    Normal,
}

impl std::fmt::Display for AdaptationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AdaptationType::QueueNotifications => "Queue notifications",
            AdaptationType::BatchNotifications => "Batch notifications",
            AdaptationType::BlockNotifications => "Block notifications",
            AdaptationType::PriorityNotifications => "Priority notifications only",
            AdaptationType::ReduceUIDensity => "Reduce UI density",
            AdaptationType::MinimalUI => "Minimal UI",
            AdaptationType::EmergencyUI => "Emergency UI mode",
            AdaptationType::BlockApps(_) => "Block distracting apps",
            AdaptationType::SuggestBreak => "Suggest break",
            AdaptationType::EnforceBreak => "Enforce break",
            AdaptationType::SummarizeContent => "Summarize content",
            AdaptationType::Normal => "Normal operation",
        };
        write!(f, "{}", s)
    }
}

/// Sensor data types
#[derive(Debug, Clone)]
pub enum SensorData {
    Gaze(GazeData),
    Keystroke(KeystrokeData),
    AppUsage(AppUsageData),
    Calendar(CalendarData),
    Wearable(WearableData),
    Voice(VoiceData),
}

/// Gaze tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GazeData {
    pub timestamp: DateTime<Utc>,
    pub x: f64,
    pub y: f64,
    pub pupil_size: Option<f64>,
    pub blink_rate: Option<f64>,
    pub fixation_duration: f64,
}

/// Keystroke dynamics data (timing only, no content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeData {
    pub timestamp: DateTime<Utc>,
    pub key_down: f64,
    pub key_up: f64,
    pub flight_time: f64,  // time between key releases and next press
    pub dwell_time: f64,   // time key is held down
    pub is_error: bool,
}

/// App usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageData {
    pub timestamp: DateTime<Utc>,
    pub app_name: String,
    pub window_title: String,
    pub is_active: bool,
    pub category: AppCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    Productivity,
    Communication,
    Development,
    Entertainment,
    Social,
    News,
    Other,
}

/// Calendar context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarData {
    pub timestamp: DateTime<Utc>,
    pub meeting_density: f64,
    pub deadline_proximity: Option<u64>,
    pub free_time_remaining: f64,
}

/// Wearable data (HRV, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WearableData {
    pub timestamp: DateTime<Utc>,
    pub heart_rate: Option<u32>,
    pub hrv: Option<f64>,
    pub stress_level: Option<f64>,
    pub recovery_score: Option<f64>,
}

/// Voice stress data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceData {
    pub timestamp: DateTime<Utc>,
    pub stress_score: f64,
    pub fatigue_indicator: f64,
    pub pitch_variance: f64,
}

/// User feedback for model personalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actual_state: AttentionState,
    pub predicted_state: AttentionState,
    pub was_correct: bool,
    pub notes: Option<String>,
}

/// Break activity suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakActivity {
    Walk,
    Stretch,
    Breathe,
    Hydrate,
    RestEyes,
    Custom(String),
}

impl BreakActivity {
    pub fn emoji(&self) -> &'static str {
        match self {
            BreakActivity::Walk => "🚶",
            BreakActivity::Stretch => "🧘",
            BreakActivity::Breathe => "🫁",
            BreakActivity::Hydrate => "💧",
            BreakActivity::RestEyes => "👁️",
            BreakActivity::Custom(_) => "✨",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            BreakActivity::Walk => "Take a short walk",
            BreakActivity::Stretch => "Do some stretches",
            BreakActivity::Breathe => "Practice deep breathing",
            BreakActivity::Hydrate => "Drink some water",
            BreakActivity::RestEyes => "Rest your eyes (20-20-20 rule)",
            BreakActivity::Custom(_) => "Custom activity",
        }
    }
}

/// Configuration for the cognitive engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub sensors: SensorConfig,
    pub adaptations: AdaptationConfig,
    pub privacy: PrivacyConfig,
    pub model: ModelConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sensors: SensorConfig::default(),
            adaptations: AdaptationConfig::default(),
            privacy: PrivacyConfig::default(),
            model: ModelConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub eye_tracking: SensorEnableConfig,
    pub keystroke: SensorEnableConfig,
    pub app_usage: SensorEnableConfig,
    pub calendar: CalendarSensorConfig,
    pub wearables: SensorEnableConfig,
    pub voice: SensorEnableConfig,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            eye_tracking: SensorEnableConfig { enabled: true, sample_rate: 30 },
            keystroke: SensorEnableConfig { enabled: true, sample_rate: 0 },
            app_usage: SensorEnableConfig { enabled: true, sample_rate: 1 },
            calendar: CalendarSensorConfig { enabled: false, source: CalendarSource::Local },
            wearables: SensorEnableConfig { enabled: false, sample_rate: 60 },
            voice: SensorEnableConfig { enabled: false, sample_rate: 10 },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEnableConfig {
    pub enabled: bool,
    pub sample_rate: u32, // Hz, 0 for event-driven
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSensorConfig {
    pub enabled: bool,
    pub source: CalendarSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarSource {
    Google,
    Local,
    Outlook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationConfig {
    pub notifications: NotificationAdaptationConfig,
    pub ui_density: UIAdaptationConfig,
    pub app_blocking: AppBlockingConfig,
    pub breaks: BreakConfig,
}

impl Default for AdaptationConfig {
    fn default() -> Self {
        Self {
            notifications: NotificationAdaptationConfig::default(),
            ui_density: UIAdaptationConfig::default(),
            app_blocking: AppBlockingConfig::default(),
            breaks: BreakConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAdaptationConfig {
    pub enabled: bool,
    pub mode: NotificationMode,
}

impl Default for NotificationAdaptationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: NotificationMode::Adaptive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationMode {
    Adaptive,
    Aggressive,
    Gentle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAdaptationConfig {
    pub enabled: bool,
}

impl Default for UIAdaptationConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBlockingConfig {
    pub enabled: bool,
    pub blocked_apps: Vec<String>,
}

impl Default for AppBlockingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            blocked_apps: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub duration_minutes: u64,
}

impl Default for BreakConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_minutes: 90,
            duration_minutes: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub data_retention_days: u64,
    pub telemetry: bool,
    pub raw_data_storage: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            data_retention_days: 7,
            telemetry: false,
            raw_data_storage: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: Option<String>,
    pub window_size_seconds: u64,
    pub prediction_horizon_seconds: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            window_size_seconds: 300,
            prediction_horizon_seconds: 900,
        }
    }
}
