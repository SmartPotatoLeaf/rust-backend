use crate::adapters::persistence::repositories::dashboard::{
    DistributionQueryResult, LabelQueryResult,
};
use spl_domain::entities::dashboard::DashboardLabelCount;
use spl_domain::entities::diagnostics::Label;
use spl_shared::maps_to;

maps_to!(Label {
    id, name, description, min, max, weight,
    #into [ created_at, updated_at ]
} #from [ LabelQueryResult, DistributionQueryResult ]);

maps_to!(LabelQueryResult {
    id, name, description, min, max, weight,
    created_at, updated_at, count,
} #from [ DistributionQueryResult ]);

impl From<LabelQueryResult> for DashboardLabelCount {
    fn from(value: LabelQueryResult) -> Self {
        Self {
            count: value.count as u64,
            label: value.into(),
        }
    }
}
