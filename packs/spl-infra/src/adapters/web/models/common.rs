use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SimplifiedQuery {
    /// Return a simplified response of entities containing only essential fields.
    #[serde(default)]
    pub simplified: bool,
}
