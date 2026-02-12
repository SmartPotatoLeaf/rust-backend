pub mod label;
pub mod mark_type;
pub mod prediction;

pub use label::{CreateLabelDto, UpdateLabelDto};
pub use mark_type::{CreateMarkTypeDto, UpdateMarkTypeDto};
pub use prediction::{CreatePredictionDto, FilterPredictionDto, UpdatePredictionDto};
