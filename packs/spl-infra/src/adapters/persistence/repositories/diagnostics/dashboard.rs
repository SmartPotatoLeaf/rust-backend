use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Func;
use sea_orm::*;
use spl_domain::ports::repositories::diagnostics::dashboard::{
    DashboardRepository, DashboardSummaryData, LabelCount, MonthlyLabelCount,
};
use spl_shared::error::Result;
use std::collections::HashMap;
use uuid::Uuid;

use crate::adapters::persistence::entities::diagnostics::{label, prediction};

pub struct DbDashboardRepository {
    db: DatabaseConnection,
}

impl DbDashboardRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl DashboardRepository for DbDashboardRepository {
    async fn get_summary(
        &self,
        user_id: Uuid,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Option<Vec<Option<Uuid>>>,
        labels: Option<Vec<String>>,
    ) -> Result<DashboardSummaryData> {
        // Base conditions
        let mut conditions = Condition::all().add(prediction::Column::UserId.eq(user_id));

        // Date filters
        if let Some(min) = min_date {
            conditions = conditions.add(prediction::Column::PredictedAt.gte(min));
        }
        if let Some(max) = max_date {
            conditions = conditions.add(prediction::Column::PredictedAt.lte(max));
        }

        // Plot IDs filter
        if let Some(ref pids) = plot_ids {
            let mut plot_condition = Condition::any();
            for pid in pids {
                match pid {
                    Some(id) => plot_condition = plot_condition.add(prediction::Column::PlotId.eq(*id)),
                    None => plot_condition = plot_condition.add(prediction::Column::PlotId.is_null()),
                }
            }
            conditions = conditions.add(plot_condition);
        }

        // 1. Get total count of predictions
        let total_query = prediction::Entity::find()
            .filter(conditions.clone())
            .count(&self.db);

        // 2. Get count of distinct plots
        let plot_count_query = async {
            if plot_ids.is_none() {
                // Count all distinct plots for user
                prediction::Entity::find()
                    .filter(prediction::Column::UserId.eq(user_id))
                    .filter(prediction::Column::PlotId.is_not_null())
                    .select_only()
                    .column(prediction::Column::PlotId)
                    .distinct()
                    .count(&self.db)
                    .await
            } else {
                // Count plots matching the filter
                let pids = plot_ids.as_ref().unwrap();
                let non_null_plots: Vec<Uuid> = pids.iter().filter_map(|p| *p).collect();
                Ok(non_null_plots.len() as u64)
            }
        };

        // 3. Get label counts with weights
        let labels_count_query = async {
            let mut query = prediction::Entity::find()
                .filter(conditions.clone())
                .join(JoinType::InnerJoin, prediction::Relation::Label.def())
                .select_only()
                .column_as(label::Column::Id, "label_id")
                .column_as(label::Column::Name, "label_name")
                .column_as(label::Column::Weight, "label_weight")
                .column_as(prediction::Column::Id.count(), "count")
                .group_by(label::Column::Id)
                .group_by(label::Column::Name)
                .group_by(label::Column::Weight);

            // Label name filter
            if let Some(ref label_names) = labels {
                let mut label_condition = Condition::any();
                for name in label_names {
                    label_condition = label_condition.add(label::Column::Name.eq(name.as_str()));
                }
                query = query.filter(label_condition);
            }

            let results: Vec<(i32, String, i32, i64)> = query
                .into_tuple()
                .all(&self.db)
                .await?;

            Ok::<Vec<LabelCount>, DbErr>(
                results
                    .into_iter()
                    .map(|(label_id, label_name, label_weight, count)| LabelCount {
                        label_id,
                        label_name,
                        label_weight,
                        count,
                    })
                    .collect(),
            )
        };

        // 4. Get monthly distribution
        let monthly_distribution_query = async {
            // Extract year and month from predicted_at
            let mut query = prediction::Entity::find()
                .filter(conditions.clone())
                .join(JoinType::InnerJoin, prediction::Relation::Label.def())
                .select_only()
                .expr_as(
                    Expr::cust("TO_CHAR(predicted_at, 'YYYY-MM')"),
                    "month"
                )
                .column_as(label::Column::Id, "label_id")
                .column_as(label::Column::Name, "label_name")
                .column_as(prediction::Column::Id.count(), "count")
                .group_by(Expr::cust("TO_CHAR(predicted_at, 'YYYY-MM')"))
                .group_by(label::Column::Id)
                .group_by(label::Column::Name)
                .order_by_asc(Expr::cust("TO_CHAR(predicted_at, 'YYYY-MM')"));

            // Label name filter
            if let Some(ref label_names) = labels {
                let mut label_condition = Condition::any();
                for name in label_names {
                    label_condition = label_condition.add(label::Column::Name.eq(name.as_str()));
                }
                query = query.filter(label_condition);
            }

            let results: Vec<(String, i32, String, i64)> = query
                .into_tuple()
                .all(&self.db)
                .await?;

            Ok::<Vec<MonthlyLabelCount>, DbErr>(
                results
                    .into_iter()
                    .map(|(month, label_id, label_name, count)| MonthlyLabelCount {
                        month,
                        label_id,
                        label_name,
                        count,
                    })
                    .collect(),
            )
        };

        // Execute all queries in parallel
        let (total, plot_count, labels_count, monthly_distribution) = tokio::try_join!(
            total_query,
            plot_count_query,
            labels_count_query,
            monthly_distribution_query
        )?;

        Ok(DashboardSummaryData {
            total: total as i64,
            plot_count: plot_count as i64,
            labels_count,
            monthly_distribution,
        })
    }
}
