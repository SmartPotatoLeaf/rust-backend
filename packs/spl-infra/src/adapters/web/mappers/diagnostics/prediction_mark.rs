use crate::adapters::web::models::diagnostics::prediction_mark::{
    PredictionMarkResponse,
};
use spl_domain::entities::diagnostics::PredictionMark;

impl From<PredictionMark> for PredictionMarkResponse {
    fn from(param: PredictionMark) -> Self {
        Self {
            id: param.id,
            data: param.data,
            mark_type: param.mark_type.into(),
            created_at: param.created_at,
        }
    }
}

