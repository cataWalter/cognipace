//! Database storage for feature vectors and cognitive state history

use crate::types::{CognitiveState, FeatureVector, AdaptationDecision, UserFeedback, AttentionState, AdaptationType};
use crate::error::{CognipaceError, Result};
use chrono::{DateTime, Utc, Duration};
use rusqlite::{Connection, params};
use tracing::{info, debug};
use std::path::Path;

/// Database manager for storing feature vectors and cognitive state history
pub struct Database {
    conn: Connection,
    retention_days: u64,
}

impl Database {
    /// Create a new database connection
    pub fn new(path: &Path, retention_days: u64) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;
        
        let db = Self { conn, retention_days };
        db.initialize()?;
        
        info!("Database initialized at {:?}", path);
        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn memory(retention_days: u64) -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;
        
        let db = Self { conn, retention_days };
        db.initialize()?;
        
        info!("In-memory database initialized");
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS feature_vectors (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                gaze_variance REAL,
                reading_speed REAL,
                typing_speed REAL,
                typing_error_rate REAL,
                typing_hesitation REAL,
                app_switching_rate REAL,
                task_complexity REAL,
                notification_count REAL,
                hrv_score REAL,
                posture_score REAL,
                voice_stress REAL,
                time_of_day REAL
            );

            CREATE TABLE IF NOT EXISTS cognitive_states (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                attention_state TEXT NOT NULL,
                load_score REAL NOT NULL,
                intrinsic_load REAL NOT NULL,
                extraneous_load REAL NOT NULL,
                physiological_load REAL NOT NULL,
                forecast_predicted_load REAL,
                forecast_confidence REAL,
                forecast_horizon_seconds INTEGER,
                forecast_overload_probability REAL
            );

            CREATE TABLE IF NOT EXISTS adaptation_decisions (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                adaptation_type TEXT NOT NULL,
                reason TEXT NOT NULL,
                cognitive_state TEXT NOT NULL,
                load_score REAL NOT NULL,
                user_override INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS user_feedback (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                actual_state TEXT NOT NULL,
                predicted_state TEXT NOT NULL,
                was_correct INTEGER NOT NULL,
                notes TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_features_timestamp ON feature_vectors(timestamp);
            CREATE INDEX IF NOT EXISTS idx_states_timestamp ON cognitive_states(timestamp);
            CREATE INDEX IF NOT EXISTS idx_adaptations_timestamp ON adaptation_decisions(timestamp);
        ").map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store a feature vector
    pub fn store_feature_vector(&self, features: &FeatureVector) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO feature_vectors (
                id, timestamp, gaze_variance, reading_speed, typing_speed,
                typing_error_rate, typing_hesitation, app_switching_rate,
                task_complexity, notification_count, hrv_score, posture_score,
                voice_stress, time_of_day
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                features.timestamp.to_rfc3339(),
                features.timestamp.to_rfc3339(),
                features.gaze_variance,
                features.reading_speed,
                features.typing_speed,
                features.typing_error_rate,
                features.typing_hesitation,
                features.app_switching_rate,
                features.task_complexity,
                features.notification_count,
                features.hrv_score,
                features.posture_score,
                features.voice_stress,
                features.time_of_day,
            ],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store a cognitive state
    pub fn store_cognitive_state(&self, state: &CognitiveState) -> Result<()> {
        let forecast = &state.forecast;
        
        self.conn.execute(
            "INSERT OR REPLACE INTO cognitive_states (
                id, timestamp, attention_state, load_score,
                intrinsic_load, extraneous_load, physiological_load,
                forecast_predicted_load, forecast_confidence,
                forecast_horizon_seconds, forecast_overload_probability
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                state.id.to_string(),
                state.timestamp.to_rfc3339(),
                serde_json::to_string(&state.attention_state).unwrap_or_default(),
                state.load_score,
                state.intrinsic_load,
                state.extraneous_load,
                state.physiological_load,
                forecast.as_ref().map(|f| f.predicted_load),
                forecast.as_ref().map(|f| f.confidence),
                forecast.as_ref().map(|f| f.horizon_seconds as i64),
                forecast.as_ref().map(|f| f.overload_probability),
            ],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store an adaptation decision
    pub fn store_adaptation_decision(&self, decision: &AdaptationDecision) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO adaptation_decisions (
                id, timestamp, adaptation_type, reason,
                cognitive_state, load_score, user_override
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                decision.id.to_string(),
                decision.timestamp.to_rfc3339(),
                serde_json::to_string(&decision.adaptation_type).unwrap_or_default(),
                decision.reason,
                serde_json::to_string(&decision.cognitive_state).unwrap_or_default(),
                decision.load_score,
                if decision.user_override { 1 } else { 0 },
            ],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store user feedback
    pub fn store_user_feedback(&self, feedback: &UserFeedback) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_feedback (
                id, timestamp, actual_state, predicted_state,
                was_correct, notes
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                feedback.id.to_string(),
                feedback.timestamp.to_rfc3339(),
                serde_json::to_string(&feedback.actual_state).unwrap_or_default(),
                serde_json::to_string(&feedback.predicted_state).unwrap_or_default(),
                if feedback.was_correct { 1 } else { 0 },
                feedback.notes,
            ],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get recent feature vectors
    pub fn get_recent_features(&self, count: usize) -> Result<Vec<FeatureVector>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM feature_vectors ORDER BY timestamp DESC LIMIT ?1"
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let rows = stmt.query_map(params![count as i64], |row| {
            Ok(FeatureVector {
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                gaze_variance: row.get(2)?,
                reading_speed: row.get(3)?,
                typing_speed: row.get(4)?,
                typing_error_rate: row.get(5)?,
                typing_hesitation: row.get(6)?,
                app_switching_rate: row.get(7)?,
                task_complexity: row.get(8)?,
                notification_count: row.get(9)?,
                hrv_score: row.get(10)?,
                posture_score: row.get(11)?,
                voice_stress: row.get(12)?,
                time_of_day: row.get(13)?,
            })
        }).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let mut features = Vec::new();
        for row in rows {
            features.push(row.map_err(|e| CognipaceError::DatabaseError(e.to_string()))?);
        }

        Ok(features)
    }

    /// Get recent cognitive states
    pub fn get_recent_states(&self, count: usize) -> Result<Vec<CognitiveState>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM cognitive_states ORDER BY timestamp DESC LIMIT ?1"
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let rows = stmt.query_map(params![count as i64], |row| {
            let attention_state: String = row.get(2)?;
            let forecast = if let Some(predicted_load) = row.get::<_, Option<f64>>(7)? {
                Some(crate::types::LoadForecast {
                    predicted_load,
                    confidence: row.get(8)?,
                    horizon_seconds: row.get::<_, Option<i64>>(9)?.unwrap_or(0) as u64,
                    overload_probability: row.get(10)?,
                })
            } else {
                None
            };

            Ok(CognitiveState {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                attention_state: serde_json::from_str(&attention_state).unwrap_or(AttentionState::Fragmented),
                load_score: row.get(3)?,
                intrinsic_load: row.get(4)?,
                extraneous_load: row.get(5)?,
                physiological_load: row.get(6)?,
                forecast,
            })
        }).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let mut states = Vec::new();
        for row in rows {
            states.push(row.map_err(|e| CognipaceError::DatabaseError(e.to_string()))?);
        }

        Ok(states)
    }

    /// Get recent adaptation decisions
    pub fn get_recent_adaptations(&self, count: usize) -> Result<Vec<AdaptationDecision>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM adaptation_decisions ORDER BY timestamp DESC LIMIT ?1"
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let rows = stmt.query_map(params![count as i64], |row| {
            let adaptation_type: String = row.get(2)?;
            let cognitive_state: String = row.get(4)?;

            Ok(AdaptationDecision {
                id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                adaptation_type: serde_json::from_str(&adaptation_type).unwrap_or(AdaptationType::Normal),
                reason: row.get(3)?,
                cognitive_state: serde_json::from_str(&cognitive_state).unwrap_or(AttentionState::Fragmented),
                load_score: row.get(5)?,
                user_override: row.get::<_, i32>(6)? != 0,
            })
        }).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let mut decisions = Vec::new();
        for row in rows {
            decisions.push(row.map_err(|e| CognipaceError::DatabaseError(e.to_string()))?);
        }

        Ok(decisions)
    }

    /// Purge old data based on retention policy
    pub fn purge_old_data(&self) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        self.conn.execute(
            "DELETE FROM feature_vectors WHERE timestamp < ?1",
            params![cutoff_str],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        self.conn.execute(
            "DELETE FROM cognitive_states WHERE timestamp < ?1",
            params![cutoff_str],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        self.conn.execute(
            "DELETE FROM adaptation_decisions WHERE timestamp < ?1",
            params![cutoff_str],
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        debug!("Purged data older than {}", cutoff_str);
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let feature_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM feature_vectors",
            [],
            |row| row.get(0),
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let state_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cognitive_states",
            [],
            |row| row.get(0),
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        let adaptation_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM adaptation_decisions",
            [],
            |row| row.get(0),
        ).map_err(|e| CognipaceError::DatabaseError(e.to_string()))?;

        Ok(DatabaseStats {
            feature_count: feature_count as usize,
            state_count: state_count as usize,
            adaptation_count: adaptation_count as usize,
            retention_days: self.retention_days,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub feature_count: usize,
    pub state_count: usize,
    pub adaptation_count: usize,
    pub retention_days: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FeatureVector;

    #[test]
    fn test_store_and_retrieve_features() {
        let db = Database::memory(7).unwrap();
        let features = FeatureVector::new(Utc::now());
        
        db.store_feature_vector(&features).unwrap();
        let retrieved = db.get_recent_features(10).unwrap();
        
        assert_eq!(retrieved.len(), 1);
    }
}
