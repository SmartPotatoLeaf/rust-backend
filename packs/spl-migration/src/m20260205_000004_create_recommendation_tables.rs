use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create Recommendation Categories Table
        manager
            .create_table(
                Table::create()
                    .table(RecommendationCategories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RecommendationCategories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RecommendationCategories::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(RecommendationCategories::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(RecommendationCategories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(RecommendationCategories::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create Recommendations Table
        manager
            .create_table(
                Table::create()
                    .table(Recommendations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Recommendations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Recommendations::Description).text().null())
                    .col(
                        ColumnDef::new(Recommendations::CategoryId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Recommendations::MinSeverity)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Recommendations::MaxSeverity)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Recommendations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Recommendations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-recommendations-category_id")
                            .from(Recommendations::Table, Recommendations::CategoryId)
                            .to(
                                RecommendationCategories::Table,
                                RecommendationCategories::Id,
                            )
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Create Index on category_id for better query performance
        manager
            .create_index(
                Index::create()
                    .table(Recommendations::Table)
                    .name("idx_recommendations_category_id")
                    .col(Recommendations::CategoryId)
                    .to_owned(),
            )
            .await?;

        // 3b. Create composite index for severity range queries
        manager
            .create_index(
                Index::create()
                    .table(Recommendations::Table)
                    .name("idx_recommendations_severity_range")
                    .col(Recommendations::MinSeverity)
                    .col(Recommendations::MaxSeverity)
                    .col(Recommendations::CategoryId)
                    .to_owned(),
            )
            .await?;

        // 4. Insert Default Recommendation Categories
        let insert = Query::insert()
            .into_table(RecommendationCategories::Table)
            .columns([
                RecommendationCategories::Name,
                RecommendationCategories::Description,
            ])
            .values_panic([
                "preventive".into(),
                "Recommendations focused on avoiding the initial development or early spread of Phytophthora infestans. These actions are designed to reduce disease risk before significant symptoms appear, through field preparation, leaf sanitation, environmental management, and proactive guarding against favorable conditions for the pathogen.".into(),
            ])
            .values_panic([
                "monitoring".into(),
                "Recommendations aimed at continuous observation and follow-up of the crop when mild symptoms or early warning signs are present. They include schedules for field inspection, detection of new lesions, weather-triggered alerts, and instructions for identifying changes in severity that may require stronger interventions.".into(),
            ])
            .values_panic([
                "cultural".into(),
                "Agronomic and management recommendations that help reduce disease progression by modifying cultivation practices—such as improving aeration, optimizing irrigation, adjusting planting density, removing infected debris, or managing soil moisture in a way that suppresses disease spread without chemical inputs.".into(),
            ])
            .values_panic([
                "chemical_control".into(),
                "Recommendations involving the responsible and safe use of fungicides to control moderate to severe infections. These include guidance on active ingredients, application intervals, resistance-management strategies, protective equipment, and regulatory considerations for safe and effective chemical treatment.".into(),
            ])
            .values_panic([
                "corrective".into(),
                "Actions intended to immediately contain or reverse disease progression once infection is already noticeable. These recommendations combine rapid interventions, emergency measures, and treatment protocols to protect unaffected tissue and prevent crop losses under worsening conditions.".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        // 5. Add CHECK constraint for severity range validation
        let check_constraint = r#"ALTER TABLE recommendations ADD CONSTRAINT chk_recommendations_severity_range 
               CHECK (min_severity >= 0 AND max_severity <= 100 AND min_severity <= max_severity)"#;
        manager.get_connection().execute_unprepared(check_constraint).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop CHECK constraint
        let drop_constraint = "ALTER TABLE recommendations DROP CONSTRAINT IF EXISTS chk_recommendations_severity_range";
        manager.get_connection().execute_unprepared(drop_constraint).await?;

        // Drop Index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_recommendations_severity_range")
                    .table(Recommendations::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_recommendations_category_id")
                    .table(Recommendations::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Recommendations Table (FK will be dropped automatically)
        manager
            .drop_table(Table::drop().table(Recommendations::Table).to_owned())
            .await?;

        // Drop Recommendation Categories Table
        manager
            .drop_table(
                Table::drop()
                    .table(RecommendationCategories::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum RecommendationCategories {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Recommendations {
    Table,
    Id,
    Description,
    CategoryId,
    MinSeverity,
    MaxSeverity,
    CreatedAt,
    UpdatedAt,
}
