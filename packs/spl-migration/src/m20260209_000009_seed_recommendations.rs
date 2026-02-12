use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DbBackend, Statement};
use sea_orm_migration::sea_query::{MysqlQueryBuilder, PostgresQueryBuilder, SqliteQueryBuilder};
use serde::Deserialize;
use serde_json::from_str;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Deserialize)]
struct SeedRecommendation {
    description: String,
    recommendation_type_id: i32,
    min_severity: f32,
    max_severity: f32,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let json_content = include_str!("../seeds/recommendations.json");
        let seeds: Vec<SeedRecommendation> = from_str(json_content)
            .map_err(|e| DbErr::Custom(format!("Failed to parse recommendations.json: {}", e)))?;

        if seeds.is_empty() {
            return Ok(());
        }

        // 1. Check if table is empty to avoid duplicate seeds
        let db = manager.get_connection();
        let query = Query::select()
            .expr(Expr::col(Recommendations::Id).count())
            .from(Recommendations::Table)
            .to_owned();

        let builder = manager.get_database_backend();
        let (sql, values) = match builder {
            DbBackend::MySql => query.build(MysqlQueryBuilder),
            DbBackend::Postgres => query.build(PostgresQueryBuilder),
            DbBackend::Sqlite => query.build(SqliteQueryBuilder),
        };

        let count_res = db
            .query_one(Statement::from_sql_and_values(builder, sql, values))
            .await?;

        let count: i64 = match count_res {
            Some(res) => res.try_get_by_index(0).unwrap_or(0),
            None => 0,
        };

        if count > 0 {
            return Ok(());
        }

        // 2. Batch Insert
        let mut insert = Query::insert();
        insert.into_table(Recommendations::Table).columns([
            Recommendations::Id,
            Recommendations::Description,
            Recommendations::CategoryId,
            Recommendations::MinSeverity,
            Recommendations::MaxSeverity,
        ]);

        for seed in seeds {
            insert.values_panic([
                uuid::Uuid::new_v4().into(),
                seed.description.into(),
                seed.recommendation_type_id.into(),
                seed.min_severity.into(),
                seed.max_severity.into(),
            ]);
        }

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Since IDs are generated randomly, we cannot easily revert specific rows without tracking them.
        // For seed data, it is often acceptable to leave data or truncate table (if safe).
        // Here we choose to do nothing, as deleting ALL recommendations might be destructive to user data.
        Ok(())
    }
}

#[derive(Iden)]
enum Recommendations {
    Table,
    Id,
    Description,
    CategoryId,
    MinSeverity,
    MaxSeverity,
}
