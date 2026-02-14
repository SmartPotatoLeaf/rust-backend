use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create Labels Table
        manager
            .create_table(
                Table::create()
                    .table(Labels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Labels::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Labels::Name)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Labels::Description).text().null())
                    .col(ColumnDef::new(Labels::Min).float().not_null())
                    .col(ColumnDef::new(Labels::Max).float().not_null())
                    .col(ColumnDef::new(Labels::Weight).integer().not_null())
                    .col(
                        ColumnDef::new(Labels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Labels::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Create Predictions Table
        manager
            .create_table(
                Table::create()
                    .table(Predictions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Predictions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Predictions::UserId).uuid().not_null())
                    .col(ColumnDef::new(Predictions::ImageId).uuid().not_null())
                    .col(ColumnDef::new(Predictions::LabelId).integer().not_null())
                    .col(ColumnDef::new(Predictions::PlotId).uuid().null())
                    .col(
                        ColumnDef::new(Predictions::PresenceConfidence)
                            .float()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Predictions::AbsenceConfidence)
                            .float()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Predictions::Severity).float().not_null())
                    .col(
                        ColumnDef::new(Predictions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-predictions-label_id")
                            .from(Predictions::Table, Predictions::LabelId)
                            .to(Labels::Table, Labels::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Create Mark Types Table
        manager
            .create_table(
                Table::create()
                    .table(MarkTypes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MarkTypes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MarkTypes::Name)
                            .string_len(32)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(MarkTypes::Description).text().null())
                    .col(
                        ColumnDef::new(MarkTypes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Create Prediction Marks Table
        manager
            .create_table(
                Table::create()
                    .table(PredictionMarks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PredictionMarks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PredictionMarks::Data)
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PredictionMarks::PredictionId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PredictionMarks::MarkTypeId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PredictionMarks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-prediction_marks-prediction_id")
                            .from(PredictionMarks::Table, PredictionMarks::PredictionId)
                            .to(Predictions::Table, Predictions::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-prediction_marks-mark_type_id")
                            .from(PredictionMarks::Table, PredictionMarks::MarkTypeId)
                            .to(MarkTypes::Table, MarkTypes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // 4. Create Index on prediction_id for better query performance
        manager
            .create_index(
                Index::create()
                    .table(PredictionMarks::Table)
                    .name("idx_prediction_marks_prediction_id")
                    .col(PredictionMarks::PredictionId)
                    .to_owned(),
            )
            .await?;

        // 5. Create Index on mark_type_id
        manager
            .create_index(
                Index::create()
                    .table(PredictionMarks::Table)
                    .name("idx_prediction_marks_mark_type_id")
                    .col(PredictionMarks::MarkTypeId)
                    .to_owned(),
            )
            .await?;

        // 6. Create Index on weight for labels ordering
        manager
            .create_index(
                Index::create()
                    .table(Labels::Table)
                    .name("idx_labels_weight")
                    .col(Labels::Weight)
                    .to_owned(),
            )
            .await?;

        // 6b. Create Index on labels severity range (min, max, weight)
        manager
            .create_index(
                Index::create()
                    .table(Labels::Table)
                    .name("idx_labels_severity_range")
                    .col(Labels::Min)
                    .col(Labels::Max)
                    .col(Labels::Weight)
                    .to_owned(),
            )
            .await?;

        // 6c. Create composite index on predictions (user_id, created_at DESC)
        manager
            .create_index(
                Index::create()
                    .table(Predictions::Table)
                    .name("idx_predictions_user_created")
                    .col(Predictions::UserId)
                    .col((Predictions::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        // 6d. Create index on predictions.image_id
        manager
            .create_index(
                Index::create()
                    .table(Predictions::Table)
                    .name("idx_predictions_image_id")
                    .col(Predictions::ImageId)
                    .to_owned(),
            )
            .await?;

        // 6e. Create index on predictions.label_id
        manager
            .create_index(
                Index::create()
                    .table(Predictions::Table)
                    .name("idx_predictions_label_id")
                    .col(Predictions::LabelId)
                    .to_owned(),
            )
            .await?;

        // 6f. Create composite index for dashboard queries (user_id, label_id, created_at DESC)
        manager
            .create_index(
                Index::create()
                    .table(Predictions::Table)
                    .name("idx_predictions_user_label_date")
                    .col(Predictions::UserId)
                    .col(Predictions::LabelId)
                    .col((Predictions::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        // 6g. Create partial index for unassigned predictions (WHERE plot_id IS NULL)
        let sql_partial_index = r#"
            CREATE INDEX idx_predictions_unassigned_user 
            ON predictions(user_id) 
            WHERE plot_id IS NULL
        "#;
        manager.get_connection().execute_unprepared(sql_partial_index).await?;

        // 6h. Create composite index on prediction_marks (prediction_id, mark_type_id)
        manager
            .create_index(
                Index::create()
                    .table(PredictionMarks::Table)
                    .name("idx_prediction_marks_pred_type")
                    .col(PredictionMarks::PredictionId)
                    .col(PredictionMarks::MarkTypeId)
                    .to_owned(),
            )
            .await?;

        // 7. Insert Default Labels
        let insert_labels = Query::insert()
            .into_table(Labels::Table)
            .columns([
                Labels::Id,
                Labels::Name,
                Labels::Description,
                Labels::Min,
                Labels::Max,
                Labels::Weight,
            ])
            .values_panic([
                1.into(),
                "healthy".into(),
                "The plant shows no visible signs of disease or lesions. Leaf color, texture, and structure remain normal, indicating a fully healthy condition.".into(),
                (0.0_f32).into(),
                (0.0_f32).into(),
                0.into(),
            ])
            .values_panic([
                2.into(),
                "low".into(),
                "Early signs of disease begin to appear as small, isolated spots or minor discoloration. The damage is minimal and does not significantly affect the plant's overall health or growth.".into(),
                (0.0_f32).into(),
                (10.0_f32).into(),
                1.into(),
            ])
            .values_panic([
                3.into(),
                "mild".into(),
                "Visible lesions or affected areas cover a moderate portion of the leaf. The disease is active, and symptoms may expand if conditions remain favorable, but the plant still retains good structural integrity.".into(),
                (10.0_f32).into(),
                (30.0_f32).into(),
                2.into(),
            ])
            .values_panic([
                4.into(),
                "severe".into(),
                "Large sections of the leaf surface are damaged, with widespread lesions, decay, or necrosis. The disease severely compromises the plant's health, and immediate intervention is required to prevent further loss.".into(),
                (30.0_f32).into(),
                (100.0_f32).into(),
                3.into(),
            ])
            .to_owned();

        manager.exec_stmt(insert_labels).await?;

        // 8. Insert Default Mark Types
        let insert_mark_types = Query::insert()
            .into_table(MarkTypes::Table)
            .columns([MarkTypes::Id, MarkTypes::Name, MarkTypes::Description])
            .values_panic([
                1.into(),
                "leaf_mask".into(),
                "Binary image mask indicating the presence of leaf regions within the corresponding image. Pixels belonging to detected leaf areas are typically assigned a value of 1 (or 255), while background or non-leaf areas are assigned a value of 0.".into(),
            ])
            .values_panic([
                2.into(),
                "lt_blg_lesion_mask".into(),
                "Binary image mask highlighting regions affected by late blight disease on the leaf surface. Pixels corresponding to infected or lesion areas are assigned a value of 1 (or 255), while healthy or unaffected areas are assigned a value of 0.".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert_mark_types).await?;

        // 9. Add CHECK constraints for data validation
        let check_constraints = vec![
            r#"ALTER TABLE labels ADD CONSTRAINT chk_labels_severity_range 
               CHECK (min >= 0 AND max <= 100 AND min <= max)"#,
            r#"ALTER TABLE predictions ADD CONSTRAINT chk_predictions_severity_range 
               CHECK (severity >= 0 AND severity <= 100)"#,
            r#"ALTER TABLE predictions ADD CONSTRAINT chk_predictions_confidence_range 
               CHECK (presence_confidence >= 0 AND presence_confidence <= 1 
                  AND absence_confidence >= 0 AND absence_confidence <= 1)"#,
        ];

        for sql in check_constraints {
            manager.get_connection().execute_unprepared(sql).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop CHECK constraints
        let drop_constraints = vec![
            "ALTER TABLE predictions DROP CONSTRAINT IF EXISTS chk_predictions_confidence_range",
            "ALTER TABLE predictions DROP CONSTRAINT IF EXISTS chk_predictions_severity_range",
            "ALTER TABLE labels DROP CONSTRAINT IF EXISTS chk_labels_severity_range",
        ];

        for sql in drop_constraints {
            manager.get_connection().execute_unprepared(sql).await?;
        }

        // Drop new indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_prediction_marks_pred_type")
                    .table(PredictionMarks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_predictions_unassigned_user")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_predictions_user_label_date")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_predictions_label_id")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_predictions_image_id")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_predictions_user_created")
                    .table(Predictions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_labels_severity_range")
                    .table(Labels::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_prediction_marks_prediction_id")
                    .table(PredictionMarks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_prediction_marks_mark_type_id")
                    .table(PredictionMarks::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_labels_weight")
                    .table(Labels::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Prediction Marks Table (FK will be dropped automatically)
        manager
            .drop_table(Table::drop().table(PredictionMarks::Table).to_owned())
            .await?;

        // Drop Mark Types Table
        manager
            .drop_table(Table::drop().table(MarkTypes::Table).to_owned())
            .await?;

        // Drop Predictions Table
        manager
            .drop_table(Table::drop().table(Predictions::Table).to_owned())
            .await?;

        // Drop Labels Table
        manager
            .drop_table(Table::drop().table(Labels::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Labels {
    Table,
    Id,
    Name,
    Description,
    Min,
    Max,
    Weight,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum MarkTypes {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
}

#[derive(Iden)]
enum Predictions {
    Table,
    Id,
    UserId,
    ImageId,
    LabelId,
    PlotId,
    PresenceConfidence,
    AbsenceConfidence,
    Severity,
    CreatedAt,
}

#[derive(Iden)]
enum PredictionMarks {
    Table,
    Id,
    Data,
    PredictionId,
    MarkTypeId,
    CreatedAt,
}
