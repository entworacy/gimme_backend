use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users Table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Uuid).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null())
                    .col(ColumnDef::new(Users::CountryCode).string().not_null())
                    .col(ColumnDef::new(Users::PhoneNumber).string().not_null())
                    .col(ColumnDef::new(Users::AccountStatus).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Users::LastLoginAt).timestamp())
                    .to_owned(),
            )
            .await?;

        // User Verifications Table
        manager
            .create_table(
                Table::create()
                    .table(UserVerifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserVerifications::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserVerifications::UserId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserVerifications::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(UserVerifications::EmailVerifiedAt).timestamp())
                    .col(
                        ColumnDef::new(UserVerifications::PhoneVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(UserVerifications::PhoneVerifiedAt).timestamp())
                    .col(
                        ColumnDef::new(UserVerifications::BusinessVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(UserVerifications::BusinessInfo).text())
                    .col(ColumnDef::new(UserVerifications::VerificationCode).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_verifications_user")
                            .from(UserVerifications::Table, UserVerifications::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // User Socials Table
        manager
            .create_table(
                Table::create()
                    .table(UserSocials::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSocials::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserSocials::UserId).integer().not_null())
                    .col(ColumnDef::new(UserSocials::Provider).string().not_null())
                    .col(ColumnDef::new(UserSocials::ProviderId).string().not_null())
                    .col(
                        ColumnDef::new(UserSocials::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_socials_user")
                            .from(UserSocials::Table, UserSocials::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for user_socials provider_id
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_socials_provider_id")
                    .table(UserSocials::Table)
                    .col(UserSocials::ProviderId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserSocials::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserVerifications::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Uuid,
    Username,
    Email,
    CountryCode,
    PhoneNumber,
    AccountStatus,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
}

#[derive(DeriveIden)]
enum UserVerifications {
    Table,
    Id,
    UserId,
    EmailVerified,
    EmailVerifiedAt,
    PhoneVerified,
    PhoneVerifiedAt,
    BusinessVerified,
    BusinessInfo,
    VerificationCode,
}

#[derive(DeriveIden)]
enum UserSocials {
    Table,
    Id,
    UserId,
    Provider,
    ProviderId,
    CreatedAt,
}
