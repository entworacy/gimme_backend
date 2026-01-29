use sea_orm::{DeriveActiveEnum, EnumIter, prelude::StringLen};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum AccountStatus {
    #[sea_orm(string_value = "ACTIVE")]
    #[serde(rename = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "PENDING")]
    #[serde(rename = "PENDING")]
    Pending,
    #[sea_orm(string_value = "BANNED")]
    #[serde(rename = "BANNED")]
    Banned,
    #[sea_orm(string_value = "PERM_BANNED")]
    #[serde(rename = "PERM_BANNED")]
    PermBanned,
}
