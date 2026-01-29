use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserDeliveryData::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserDeliveryData::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserDeliveryData::UserId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(UserDeliveryData::RecipientName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserDeliveryData::PhoneNumber)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserDeliveryData::ZipCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserDeliveryData::Address)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserDeliveryData::DetailAddress).string())
                    .col(ColumnDef::new(UserDeliveryData::EntrancePassword).string())
                    .col(ColumnDef::new(UserDeliveryData::ShippingMemo).text())
                    .col(
                        ColumnDef::new(UserDeliveryData::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UserDeliveryData::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_delivery_data_users")
                            .from(UserDeliveryData::Table, UserDeliveryData::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserDeliveryData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserDeliveryData {
    Table,
    Id,
    UserId,
    RecipientName,
    PhoneNumber,
    ZipCode,
    Address,
    DetailAddress,
    EntrancePassword, // Added
    ShippingMemo,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
