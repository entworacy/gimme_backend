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

    #[sea_orm(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<verification::Model>,

    #[sea_orm(ignore)]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub socials: Vec<super::social::Model>,

    #[sea_orm(ignore)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery: Option<crate::modules::delivery::entities::delivery_data::Model>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "verification::Entity")]
    UserVerification,
    #[sea_orm(has_many = "super::social::Entity")]
    UserSocials,
    #[sea_orm(has_one = "crate::modules::delivery::entities::delivery_data::Entity")]
    UserDeliveryData,
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

impl Related<crate::modules::delivery::entities::delivery_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserDeliveryData.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
