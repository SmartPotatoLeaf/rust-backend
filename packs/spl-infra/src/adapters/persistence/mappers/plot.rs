use crate::adapters::persistence::entities::plot::{ActiveModel, Model};
use crate::adapters::persistence::repositories::plot::{DetailedPlotQueryResult, PlotQueryResult};
use spl_domain::entities::plot::Plot;
use spl_domain::ports::repositories::plot::DetailedPlot;
use spl_shared::{map_mirror, maps_set};

map_mirror!(Model, Plot {
   id, company_id, name, description,
   #into [ created_at, updated_at ]
});

maps_set!(ActiveModel {
  id, company_id, name, description,
  #into [ created_at, updated_at ]
} #from [ Plot ]);

impl From<DetailedPlotQueryResult> for DetailedPlot {
    fn from(value: DetailedPlotQueryResult) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            created_at: value.created_at,
            total_diagnosis: value.total_diagnosis.unwrap_or(0),
            last_diagnosis: value.last_diagnosis,
            matching_diagnosis: value.matching_diagnosis.unwrap_or(0),
        }
    }
}

impl From<PlotQueryResult> for Plot {
    fn from(value: PlotQueryResult) -> Self {
        Self {
            id: value.id.unwrap_or_default(),
            company_id: value.company_id,
            name: value.name,
            description: value.description,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}
