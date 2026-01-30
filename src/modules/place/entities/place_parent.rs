use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "place_parent")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub place_name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
