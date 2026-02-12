use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create Plots Table
        manager
            .create_table(
                Table::create()
                    .table(Plots::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Plots::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Plots::Name).string_len(64).not_null())
                    .col(ColumnDef::new(Plots::Description).text().null())
                    .col(ColumnDef::new(Plots::CompanyId).uuid().not_null())
                    .col(
                        ColumnDef::new(Plots::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Plots::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-plots-company_id")
                            .from(Plots::Table, Plots::CompanyId)
                            .to(Companies::Table, Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create Index on company_id for query performance
        manager
            .create_index(
                Index::create()
                    .table(Plots::Table)
                    .name("idx_plots_company_id")
                    .col(Plots::CompanyId)
                    .to_owned(),
            )
            .await?;

        // 3. Add Foreign Key from predictions.plot_id to plots.id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-predictions-plot_id")
                    .from(Predictions::Table, Predictions::PlotId)
                    .to(Plots::Table, Plots::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // 4. Create Index on predictions.plot_id for query performance
        manager
            .create_index(
                Index::create()
                    .table(Predictions::Table)
                    .name("idx_predictions_plot_id")
                    .col(Predictions::PlotId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop Index on predictions.plot_id
        manager
            .drop_index(
                Index::drop()
                    .name("idx_predictions_plot_id")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Foreign Key from predictions.plot_id
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk-predictions-plot_id")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Index on plots.company_id
        manager
            .drop_index(
                Index::drop()
                    .name("idx_plots_company_id")
                    .table(Plots::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Plots Table
        manager
            .drop_table(Table::drop().table(Plots::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Plots {
    Table,
    Id,
    Name,
    Description,
    CompanyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Companies {
    Table,
    Id,
}

#[derive(Iden)]
enum Predictions {
    Table,
    PlotId,
}
