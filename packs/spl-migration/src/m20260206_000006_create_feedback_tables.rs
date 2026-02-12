use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create Feedback Status Table
        manager
            .create_table(
                Table::create()
                    .table(FeedbackStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FeedbackStatus::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FeedbackStatus::Name)
                            .string_len(16)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(FeedbackStatus::Description).text().null())
                    .col(
                        ColumnDef::new(FeedbackStatus::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(FeedbackStatus::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create Feedbacks Table
        manager
            .create_table(
                Table::create()
                    .table(Feedbacks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Feedbacks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Feedbacks::Comment).text().null())
                    .col(
                        ColumnDef::new(Feedbacks::StatusId)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Feedbacks::CorrectLabelId).integer().null())
                    .col(
                        ColumnDef::new(Feedbacks::PredictionId)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Feedbacks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Feedbacks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feedbacks-status_id")
                            .from(Feedbacks::Table, Feedbacks::StatusId)
                            .to(FeedbackStatus::Table, FeedbackStatus::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feedbacks-correct_label_id")
                            .from(Feedbacks::Table, Feedbacks::CorrectLabelId)
                            .to(Labels::Table, Labels::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feedbacks-prediction_id")
                            .from(Feedbacks::Table, Feedbacks::PredictionId)
                            .to(Predictions::Table, Predictions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Create Index on status_id for better query performance
        manager
            .create_index(
                Index::create()
                    .table(Feedbacks::Table)
                    .name("idx_feedbacks_status_id")
                    .col(Feedbacks::StatusId)
                    .to_owned(),
            )
            .await?;

        // 4. Create Index on prediction_id
        manager
            .create_index(
                Index::create()
                    .table(Feedbacks::Table)
                    .name("idx_feedbacks_prediction_id")
                    .col(Feedbacks::PredictionId)
                    .to_owned(),
            )
            .await?;

        // 5. Insert Default Feedback Statuses
        let insert = Query::insert()
            .into_table(FeedbackStatus::Table)
            .columns([FeedbackStatus::Name, FeedbackStatus::Description])
            .values_panic([
                "pending".into(),
                "The request is awaiting a decision or further action before it can proceed.".into(),
            ])
            .values_panic([
                "accepted".into(),
                "The request or item has been formally received and acknowledged as valid.".into(),
            ])
            .values_panic([
                "rejected".into(),
                "The request has been reviewed and explicitly declined, with no further action expected.".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop Indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_feedbacks_status_id")
                    .table(Feedbacks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_feedbacks_prediction_id")
                    .table(Feedbacks::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Feedbacks Table (FKs will be dropped automatically)
        manager
            .drop_table(Table::drop().table(Feedbacks::Table).to_owned())
            .await?;

        // Drop Feedback Status Table
        manager
            .drop_table(Table::drop().table(FeedbackStatus::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum FeedbackStatus {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Feedbacks {
    Table,
    Id,
    Comment,
    StatusId,
    CorrectLabelId,
    PredictionId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Labels {
    Table,
    Id,
}

#[derive(Iden)]
enum Predictions {
    Table,
    Id,
}
