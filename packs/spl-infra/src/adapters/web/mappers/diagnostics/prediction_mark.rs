use crate::adapters::web::models::diagnostics::prediction_mark::PredictionMarkResponse;
use spl_domain::entities::diagnostics::PredictionMark;
use spl_shared::maps_to;

maps_to!(PredictionMarkResponse {
    id, data, prediction_id, created_at,
    #into [ mark_type ]
} #from [ PredictionMark ]);
