use crate::adapters::persistence::entities::diagnostics::{label, prediction};
use crate::adapters::persistence::entities::user;
use crate::adapters::persistence::repositories::DbPredictionRepository;
use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::*;
use spl_domain::entities::dashboard::{
    DashboardDistribution, DashboardLabelCount, DashboardSummary,
};
use spl_domain::ports::repositories::dashboard::DashboardSummaryRepository;
use spl_shared::error::{AppError, Result};
use std::collections::HashMap;
use uuid::Uuid;

pub struct DbDashboardSummaryRepository {
    db: DatabaseConnection,
}

impl DbDashboardSummaryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[derive(FromQueryResult)]
pub struct LabelQueryResult {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub min: f32,
    pub max: f32,
    pub weight: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub count: i64,
}

#[derive(FromQueryResult)]
pub struct DistributionQueryResult {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub min: f32,
    pub max: f32,
    pub weight: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub count: i64,
    pub month: String,
}

#[async_trait::async_trait]
impl DashboardSummaryRepository for DbDashboardSummaryRepository {
    async fn get_summary(
        &self,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<DashboardSummary> {
        if users_ids.is_empty() {
            return Err(AppError::NoContent(
                "Cannot generate a dashboard without users".to_string(),
            ));
        }

        let query = DbPredictionRepository::build_filter_query(
            users_ids.clone(),
            labels.clone(),
            Some(plot_ids.clone()),
            min_date,
            max_date,
        );

        let labels_select = DbPredictionRepository::add_filter_query(
            label::Entity::find()
                .join(JoinType::LeftJoin, label::Relation::Prediction.def())
                .column_as(prediction::Column::Id.count(), "count"),
            users_ids.clone(),
            labels,
            Some(plot_ids.clone()),
            min_date,
            max_date,
        )
        .group_by(label::Column::Id)
        .group_by(label::Column::Name);

        let plot_conditions = DbPredictionRepository::build_plots_condition(plot_ids);

        let monthly_expr = Expr::cust("TO_CHAR(\"predictions\".\"created_at\", 'YYYY-MM')");

        let (total, plot_count, labels, distributions) = tokio::try_join!(
            query.clone().select_only().count(&self.db),
            prediction::Entity::find()
                .inner_join(user::Entity)
                .filter(plot_conditions)
                .filter(user::Column::Id.is_in(users_ids))
                .select_only()
                .column_as(
                    Expr::col(prediction::Column::PlotId).count_distinct().add(
                        Expr::case(
                            Func::sum(
                                Expr::col(prediction::Column::PlotId)
                                    .is_null()
                                    .cast_as("int")
                            )
                            .gt(0),
                            Expr::val(1),
                        )
                        .finally(Expr::val(0))
                    ),
                    "count"
                )
                .into_tuple::<i64>()
                .one(&self.db),
            labels_select
                .clone()
                .into_model::<LabelQueryResult>()
                .all(&self.db),
            labels_select
                .column_as(monthly_expr.clone(), "month",)
                .group_by(monthly_expr)
                .into_model::<DistributionQueryResult>()
                .all(&self.db),
        )?;

        // Calculate mean severity
        let labels_count: f64 = labels.len() as f64;

        let mut mean = labels
            .into_iter()
            .map(|l| l.weight as i64 * l.count)
            .sum::<i64>() as f64;

        mean /= labels_count;

        // Group monthly distribution by month
        let mut monthly_map: HashMap<String, Vec<LabelQueryResult>> = HashMap::new();

        for distribution in distributions {
            monthly_map
                .entry(distribution.month.clone())
                .or_insert_with(Vec::new)
                .push(LabelQueryResult::from(distribution));
        }

        Ok(DashboardSummary {
            total,
            plots: plot_count.unwrap_or(0) as u64,
            mean_severity: mean as f32,
            distribution: Some(
                monthly_map
                    .into_iter()
                    .map(|(month, labels)| DashboardDistribution {
                        month,
                        labels: labels.into_iter().map(DashboardLabelCount::from).collect(),
                    })
                    .collect(),
            ),
        })
    }
}
