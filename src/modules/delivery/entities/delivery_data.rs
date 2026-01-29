use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "user_delivery_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub user_id: i32,
    pub recipient_name: String,
    pub phone_number: String,
    pub zip_code: String,
    pub address: String,
    pub detail_address: Option<String>,
    pub entrance_password: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub shipping_memo: Option<String>,
    #[serde(skip_deserializing)]
    pub created_at: DateTime,
    #[serde(skip_deserializing)]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "crate::modules::users::entities::user::Entity",
        from = "Column::UserId",
        to = "crate::modules::users::entities::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<crate::modules::users::entities::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
