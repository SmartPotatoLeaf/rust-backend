use crate::adapters::web::models::diagnostics::prediction_mark::{
    PredictionMarkResponse, RawPredictionMarkResponse,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use spl_domain::entities::diagnostics::{PredictionMark, RawPredictionMark};
use spl_shared::maps_to;

maps_to!(PredictionMarkResponse {
    id, data, prediction_id, created_at,
    #into [ mark_type ]
} #from [ PredictionMark ]);

impl From<RawPredictionMark> for RawPredictionMarkResponse {
    fn from(value: RawPredictionMark) -> Self {
        let data = BASE64.encode(&value.data);
        Self {
            mark_type: value.mark_type,
            data,
        }
    }
}
