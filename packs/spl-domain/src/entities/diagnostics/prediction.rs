use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Label, PredictionMark};
use crate::entities::feedback::Feedback;
use crate::entities::image::Image;
use crate::entities::user::User;

/// Represents a disease prediction result from the ML model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: Uuid,
    /// User who owns this prediction
    pub user: User,
    /// The image used for prediction
    pub image: Image,
    /// The assigned severity label
    pub label: Label,
    pub marks: Vec<PredictionMark>,
    /// Associated plot (optional)
    pub plot_id: Option<Uuid>,
    /// Confidence level that disease is present (0.0 - 1.0)
    pub presence_confidence: f32,
    /// Confidence level that disease is absent (0.0 - 1.0)
    pub absence_confidence: f32,
    /// Severity percentage of the disease (0.0 - 100.0)
    pub severity: f32,
    /// Feedback associated with this prediction (optional, one-to-one)
    pub feedback: Option<Feedback>,
    /// When the prediction was created
    pub created_at: DateTime<Utc>,
}
