use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct SimplifiedQuery {
    #[serde(default)]
    pub simplified: bool,
}
