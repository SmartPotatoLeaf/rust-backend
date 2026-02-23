use crate::adapters::persistence::entities::diagnostics::{label, prediction};
use crate::adapters::persistence::entities::user;
use crate::adapters::persistence::repositories::DbPredictionRepository;
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use itertools::Itertools;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::{Expr, ExprTrait, Func};
use sea_orm::*;
use spl_domain::entities::dashboard::{
    DashboardCounts, DashboardDetailedPlot, DashboardDistribution, DashboardLabelCount,
    DashboardSummary,
};
use spl_domain::ports::repositories::dashboard::DashboardSummaryRepository;
use spl_domain::ports::repositories::diagnostics::PredictionRepository;
use spl_domain::ports::repositories::plot::PlotRepository;
use spl_shared::error::{AppError, Result};
use spl_shared::traits::IntoWithContext;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, FromQueryResult)]
struct SummaryRow {
    pub plot_id: Option<Uuid>,
    pub label_id: Option<Uuid>,
    pub label_name: Option<String>,
    pub label_weight: Option<i32>,
    pub count: i64,
    pub month: Option<chrono::NaiveDateTime>, // DATE_TRUNC devuelve timestamp
}

pub struct DbDashboardSummaryRepository {
    db: DatabaseConnection,
    prediction_repository: Arc<dyn PredictionRepository>,
    plot_repository: Arc<dyn PlotRepository>,
}

impl DbDashboardSummaryRepository {
    pub fn new(
        db: DatabaseConnection,
        prediction_repository: Arc<dyn PredictionRepository>,
        plot_repository: Arc<dyn PlotRepository>,
    ) -> Self {
        Self {
            db,
            prediction_repository,
            plot_repository,
        }
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

        let mut mean = labels
            .into_iter()
            .map(|l| l.weight as i64 * l.count)
            .sum::<i64>() as f64;

        if total > 0 {
            mean /= total as f64;
        }

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

    async fn get_counts(
        &self,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
        last_n: u64,
    ) -> Result<DashboardCounts> {
        // Get summary statistics by reusing get_summary
        let (summary, predictions) = tokio::try_join!(
            self.get_summary(
                users_ids.clone(),
                min_date,
                max_date,
                plot_ids.clone(),
                labels.clone()
            ),
            self.prediction_repository.filter(
                users_ids,
                labels,
                Some(plot_ids),
                min_date,
                max_date,
                0,
                last_n,
            )
        )?;

        Ok(DashboardCounts {
            total: summary.total,
            plots: summary.plots,
            mean_severity: summary.mean_severity,
            distribution: summary.distribution,
            last_predictions: predictions.1,
        })
    }

    async fn get_summary_detailed_plot_by_id(
        &self,
        company_id: Uuid,
        plot_id: Uuid,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        labels: Option<Vec<String>>,
    ) -> Result<Option<DashboardDetailedPlot>> {
        let (detailed, summary) = tokio::try_join!(
            self.plot_repository.get_detailed_by_id(
                company_id,
                plot_id,
                labels.clone().unwrap_or(vec![])
            ),
            self.get_summary(users_ids, min_date, max_date, vec![Some(plot_id)], labels)
        )?;

        let detailed = detailed
            .ok_or_else(|| AppError::NotFound(format!("Plot with id {} not found", plot_id)))?;

        Ok(Some(detailed.into_with_context(summary)?))
    }

    async fn get_default_summary_detailed_plot(
        &self,
        company_id: Uuid,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<Option<DashboardDetailedPlot>> {
        let (detailed, summary) = tokio::try_join!(
            self.plot_repository
                .get_default_detailed(company_id, labels.clone().unwrap_or(vec![])),
            self.get_summary(users_ids, min_date, max_date, plot_ids, labels)
        )?;

        let detailed = detailed
            .ok_or_else(|| AppError::NotFound("Company has no default plot".to_string()))?;

        Ok(Some(detailed.into_with_context(summary)?))
    }

    async fn get_compare(
        &self,
        users_ids: Vec<Uuid>,
        min_date: Option<DateTime<Utc>>,
        max_date: Option<DateTime<Utc>>,
        plot_ids: Vec<Option<Uuid>>,
        labels: Option<Vec<String>>,
    ) -> Result<Vec<DashboardSummary>> {
        let futures = plot_ids
            .into_iter()
            .unique()
            .map(|el| {
                self.get_summary(
                    users_ids.clone(),
                    min_date,
                    max_date,
                    vec![el],
                    labels.clone(),
                )
            })
            .collect::<Vec<_>>();

        try_join_all(futures).await
    }
}
