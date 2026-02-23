use crate::adapters::persistence::entities::user::user;
use crate::adapters::persistence::entities::{diagnostics::label, diagnostics::prediction, plot};
use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::sea_query::{
    Expr, Func, IntoColumnRef, Query, SelectStatement, SimpleExpr, UnionType,
};
use sea_orm::JoinType;
use sea_orm::*;
use spl_domain::entities::plot::{DetailedPlot, Plot};
use spl_domain::ports::repositories::crud::CrudRepository;
use spl_domain::ports::repositories::plot::{PlotRepository};
use spl_shared::adapters::persistence::repository::crud;
use spl_shared::error::{AppError, Result};
use uuid::Uuid;
pub struct DbPlotRepository {
    db: DatabaseConnection,
}

// Auxiliary structure to receive raw data from the DB in a single query
#[derive(Debug, Clone, FromQueryResult)]
pub struct DetailedPlotQueryResult {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    // Aggregations
    pub total_diagnosis: Option<i64>, // Count returns i64 (or i32 depending on driver), we use Option for safety
    pub last_diagnosis: Option<DateTime<Utc>>,
    pub matching_diagnosis: Option<i64>, // Result of conditional SUM
}

#[derive(Debug, Clone, FromQueryResult)]
pub struct PlotQueryResult {
    pub id: Option<Uuid>,
    pub company_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl DbPlotRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn get_total<E, M>(&self, query: Select<E>) -> Result<i64>
    where
        E: EntityTrait<Model = M>,
        M: FromQueryResult + Sized + Send + Sync,
    {
        let total_diagnosis = query.count(&self.db).await.map_err(AppError::from)?;

        Ok(total_diagnosis as i64)
    }

    fn find_detailed<E>(select: Select<E>, labels: &Vec<String>) -> Select<E>
    where
        E: EntityTrait,
    {
        let matching_condition = if labels.is_empty() {
            // If no labels, count where severity is 0.0 (healthy)
            Expr::col((prediction::Entity, prediction::Column::Severity)).eq(0.0)
        } else {
            // If labels exist, count where label.name matches any of the labels
            Expr::col((label::Entity, label::Column::Name)).is_in(labels.clone())
        };

        select
            // Aggregation 1: Total Diagnosis (COUNT(prediction.id))
            // Since it's a LEFT JOIN, if there are no predictions, count returns 0, which is correct
            .column_as(
                Expr::col((prediction::Entity, prediction::Column::Id)).count(),
                "total_diagnosis",
            )
            // Aggregation 2: Last Diagnosis (MAX(prediction.created_at))
            .column_as(
                Expr::col((prediction::Entity, prediction::Column::CreatedAt)).max(),
                "last_diagnosis",
            )
            // Aggregation 3: Matching Diagnosis (SUM(CASE WHEN condition THEN 1 ELSE 0))
            // This counts only predictions matching label condition, keeping the plot visible
            .expr_as(
                Func::sum(Expr::case(matching_condition, 1).finally(0)),
                "matching_diagnosis",
            )
    }

    fn get_default_fields() -> Vec<(&'static str, SimpleExpr)> {
        vec![
            ("id", Expr::value(None::<Uuid>)), // Default plot has no ID
            ("name", Expr::value("Default Plot")),
            (
                "description",
                Expr::value("Default plot for unassigned predictions"),
            ),
            ("created_at", Expr::value(Utc::now())), // Use the current time for created_at of the default plot
            ("updated_at", Expr::value(Utc::now())), // Use current time for updated_at of the default plot
        ]
    }

    fn find_default_detailed(
        company_id: Uuid,
        labels: &Vec<String>,
    ) -> (Select<prediction::Entity>, Select<prediction::Entity>) {
        // Base query for unassigned predictions for the company
        let base_select = prediction::Entity::find()
            .join(JoinType::InnerJoin, prediction::Relation::User.def())
            .left_join(label::Entity)
            .filter(prediction::Column::PlotId.is_null())
            .filter(user::Column::CompanyId.eq(company_id));

        // Build the default select with aggregations, similar to the detailed plot query but for unassigned predictions
        let mut default_select = base_select.clone().select_only();
        for (name, field) in DbPlotRepository::get_default_fields() {
            if name != "updated_at" {
                default_select = default_select.expr_as(field, name);
            }
        }

        let query = DbPlotRepository::find_detailed(default_select, &labels);
        (base_select, query)
    }

    fn create_plot_detailed_query(
        query: Select<plot::Entity>,
        labels: &Vec<String>,
    ) -> Select<plot::Entity> {
        DbPlotRepository::find_detailed(
            query
                .select_only()
                // Select Plot columns
                .column(plot::Column::Id)
                .column(plot::Column::Name)
                .column(plot::Column::Description)
                .column(plot::Column::CreatedAt),
            labels,
        )
        // Group by Plot ID for aggregations to work
        .group_by(plot::Column::Id)
        // Sorting and Pagination
        .order_by_desc(plot::Column::CreatedAt)
    }
    async fn fetch_detailed_plots(
        &self,
        detailed_select: Select<plot::Entity>,
        default_select: Select<prediction::Entity>,
    ) -> Result<Vec<DetailedPlot>> {
        // Creates query statements and combines them with UNION ALL to get both plots and default plot in a single query
        let mut query = SelectStatement::new();
        let query = query
            .from_subquery(detailed_select.clone().into_query(), "detailed_plots")
            .columns(vec![
                "id".into_column_ref(),
                "name".into_column_ref(),
                "description".into_column_ref(),
                "created_at".into_column_ref(),
                "total_diagnosis".into_column_ref(),
                "last_diagnosis".into_column_ref(),
                "matching_diagnosis".into_column_ref(),
            ]);

        let default_query = default_select.into_query();

        let union = query.union(UnionType::All, default_query);

        // get the database backend to execute the union query
        let backend = self.db.get_database_backend();

        let stmt = backend.build(union);

        // Run the combined query to get both plots and default plot in a single round trip
        let results: Vec<DetailedPlot> = DetailedPlotQueryResult::find_by_statement(stmt)
            .all(&self.db)
            .await
            .map_err(AppError::from)?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(results)
    }
}

#[async_trait::async_trait]
impl CrudRepository<Plot, Uuid> for DbPlotRepository {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<Plot>> {
        crud::get_by_id::<plot::Entity, Plot, Uuid>(&self.db, id).await
    }

    async fn create(&self, entity: Plot) -> Result<Plot> {
        crud::create::<plot::Entity, Plot>(&self.db, entity).await
    }

    async fn update(&self, entity: Plot) -> Result<Plot> {
        crud::update::<plot::Entity, Plot>(&self.db, entity).await
    }

    async fn delete(&self, id: Uuid) -> Result<Plot> {
        crud::delete::<plot::Entity, Plot, Uuid>(&self.db, id).await
    }
}

#[async_trait::async_trait]
impl PlotRepository for DbPlotRepository {
    async fn get_by_company_id(&self, company_id: Uuid) -> Result<Vec<Plot>> {
        let models = plot::Entity::find()
            .filter(plot::Column::CompanyId.eq(company_id))
            .order_by_desc(plot::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn get_all_by_company_id(&self, company_id: Uuid) -> Result<Vec<Plot>> {
        // Creates the query for create the default plot with a subquery to check
        // if there are any unassigned predictions for the company
        let exists_default_query = Query::select()
            .expr(Expr::val(1))
            .from(prediction::Entity)
            .inner_join(
                user::Entity,
                Expr::col((user::Entity, user::Column::Id))
                    .equals((prediction::Entity, prediction::Column::UserId)),
            )
            .and_where(user::Column::CompanyId.eq(company_id))
            .and_where(prediction::Column::PlotId.is_null())
            .to_owned();

        let mut default_query = Query::select();

        for (name, field) in Self::get_default_fields() {
            default_query.expr_as(field, name);
        }

        let default_select = default_query
            .expr_as(Expr::val(company_id), "company_id") // Add company_id to the default plot select
            .and_where(Expr::exists(exists_default_query))
            .to_owned();

        // Filter to get all plots for the company
        let plots_select = plot::Entity::find().filter(plot::Column::CompanyId.eq(company_id));

        // Maps and sorts the plot query to select the necessary fields and combines
        // it with the default plot query using UNION ALL
        let mut query = SelectStatement::new();
        let query = query
            .from_subquery(
                plots_select
                    .clone()
                    .order_by_desc(plot::Column::CreatedAt)
                    .into_query(),
                "plots",
            )
            .columns(vec![
                "id".into_column_ref(),
                "name".into_column_ref(),
                "description".into_column_ref(),
                "created_at".into_column_ref(),
                "updated_at".into_column_ref(),
                "company_id".into_column_ref(),
            ]);

        // Combines the plots query with the default plot query using UNION ALL to ensure the default plot is included if there are unassigned predictions
        let union = query.union(UnionType::All, default_select);
        let backend = self.db.get_database_backend();
        let stmt = backend.build(union);

        let plots = plot::Model::find_by_statement(stmt)
            .into_model::<PlotQueryResult>()
            .all(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(plots.into_iter().map(Into::into).collect())
    }

    async fn get_by_company_id_and_id(&self, company_id: Uuid, id: Uuid) -> Result<Option<Plot>> {
        let model = plot::Entity::find()
            .filter(plot::Column::Id.eq(id))
            .filter(plot::Column::CompanyId.eq(company_id))
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(model.map(Into::into))
    }

    async fn get_detailed(
        &self,
        company_id: Uuid,
        offset: u64,
        limit: u64,
        labels: Vec<String>,
    ) -> Result<(i64, Vec<DetailedPlot>)> {
        // Base Query: plot left join prediction left join label
        // Use LEFT JOIN to ensure plots without predictions are included.
        // DO NOT apply WHERE filters on predictions/labels to ensure ALL company plots are listed.
        let query = plot::Entity::find()
            .filter(plot::Column::CompanyId.eq(company_id))
            .left_join(prediction::Entity)
            .join(JoinType::LeftJoin, prediction::Relation::Label.def());

        // The count query must count ALL company plots, regardless of label filters
        let count_query = plot::Entity::find().filter(plot::Column::CompanyId.eq(company_id));

        // Creates detailed select for plots with aggregations
        let detailed_select = DbPlotRepository::create_plot_detailed_query(query, &labels)
            .offset(offset)
            .limit(limit);

        // Creates detailed select for default plot (unassigned predictions) with aggregations
        let (_, default_select) = DbPlotRepository::find_default_detailed(company_id, &labels);

        // 3. Execute Total Count (Query 1) and Fetch Detailed Plots (Query 2) concurrently
        let (mut total, mut results) = tokio::try_join!(
            self.get_total(count_query),
            self.fetch_detailed_plots(detailed_select, default_select)
        )?;

        // Check if the default plot is included in the results and adjust total count accordingly
        if !results.is_empty() {
            if let Some(may_default) = results.last() {
                if may_default.id.is_none() {
                    total += 1;
                }
            }
        }

        // If we have more results than the limit, we need to remove the default plot
        let size = results.len() as u64;
        if size > limit {
            results = results
                .into_iter()
                .take((size - 1) as usize)
                .collect::<Vec<_>>();
        }

        Ok((total, results))
    }

    async fn get_detailed_by_id(
        &self,
        company_id: Uuid,
        plot_id: Uuid,
        labels: Vec<String>,
    ) -> Result<Option<DetailedPlot>> {
        let query = plot::Entity::find()
            .filter(plot::Column::Id.eq(plot_id))
            .filter(plot::Column::CompanyId.eq(company_id))
            .left_join(prediction::Entity)
            .join(JoinType::LeftJoin, prediction::Relation::Label.def());

        let first = DbPlotRepository::find_detailed(query, &labels)
            .group_by(plot::Column::Id)
            .into_model::<DetailedPlotQueryResult>()
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(first.map(Into::into))
    }

    async fn get_default_detailed(
        &self,
        company_id: Uuid,
        labels: Vec<String>,
    ) -> Result<Option<DetailedPlot>> {
        // Base query for unassigned predictions for the company
        let (base_query, query) = DbPlotRepository::find_default_detailed(company_id, &labels);

        let total_diagnosis = self.get_total(base_query.clone()).await?;

        if total_diagnosis == 0 {
            return Ok(None);
        }

        let default = query
            .into_model::<DetailedPlotQueryResult>()
            .one(&self.db)
            .await
            .map_err(AppError::from)?;

        Ok(default.map(Into::into))
    }
}
