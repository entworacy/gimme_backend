use sea_orm::{DeriveActiveEnum, EnumIter, prelude::StringLen};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum AccountStatus {
    #[sea_orm(string_value = "ACTIVE")]
    Active,
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "BANNED")]
    Banned,
    #[sea_orm(string_value = "PERM_BANNED")]
    PermBanned,
}
