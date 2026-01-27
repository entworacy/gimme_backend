use crate::modules::users::entities::verification;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_serializing)]
    pub id: i32,
    #[sea_orm(unique, index)]
    pub uuid: String, // Decimal string representation of UUIDv4
    pub username: String,
    pub email: String,
    pub country_code: String,
    pub phone_number: String,

    pub account_status: super::enums::AccountStatus,
    #[serde(skip_deserializing)]
    pub created_at: DateTime,
    #[serde(skip_deserializing)]
    pub updated_at: DateTime,
    pub last_login_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "verification::Entity")]
    UserVerification,
    #[sea_orm(has_many = "super::social::Entity")]
    UserSocials,
}

impl Related<verification::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserVerification.def()
    }
}

impl Related<super::social::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSocials.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
