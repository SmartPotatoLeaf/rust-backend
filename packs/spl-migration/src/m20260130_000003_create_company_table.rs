use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create Companies Table
        manager
            .create_table(
                Table::create()
                    .table(Companies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Companies::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Companies::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Companies::Description).string().null())
                    .col(
                        ColumnDef::new(Companies::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Companies::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Alter Users Table: Add company_id FK
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::CompanyId).uuid().null(), // Nullable for Admin
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-users-company_id")
                            .from_tbl(Users::Table)
                            .from_col(Users::CompanyId)
                            .to_tbl(Companies::Table)
                            .to_col(Companies::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Add Composite Unique Key (username, company_id)
        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .name("idx_users_username_company")
                    .col(Users::Username)
                    .col(Users::CompanyId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 3b. Add Index on company_id for multi-tenant queries
        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .name("idx_users_company_id")
                    .col(Users::CompanyId)
                    .to_owned(),
            )
            .await?;

        // 3c. Add Index on role_id for permission queries
        manager
            .create_index(
                Index::create()
                    .table(Users::Table)
                    .name("idx_users_role_id")
                    .col(Users::RoleId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop Indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_role_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_company_id")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Index
        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_username_company")
                    .table(Users::Table)
                    .to_owned(),
            )
            .await?;

        // Drop Company FK and Column
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::CompanyId)
                    // Constraint dropped automatically usually with column drop or might persist?
                    .to_owned(),
            )
            .await?;

        // Restore Username unique?
        // manager.create_index(...).await?;

        manager
            .drop_table(Table::drop().table(Companies::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Companies {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    CompanyId,
    Username,
    RoleId,
}
