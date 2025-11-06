
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadingSession {
    pub date: DateTime<Utc>,
    pub file_path: String,
    pub words_read: usize,
    pub reading_time: f64,
    pub avg_speed: u64,
}
