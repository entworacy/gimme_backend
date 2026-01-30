use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "place_parent", rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub place_name: String,
    pub fulfillment_type: PlaceFulfillmentType,
    #[sea_orm(default = PlaceFulfillmentStatus::Active)]
    pub fulfillment_status: PlaceFulfillmentStatus,
    pub open_time: Vec<u32>,
    pub close_time: Vec<u32>,
    #[sea_orm(default = false)]
    pub is_public: bool,
    #[sea_orm(default = false)]
    pub fc_able_split_shipping: bool,
    #[sea_orm(default = None)]
    pub min_shipping_amount_krw: Option<u32>,
    pub base_currency_code: Option<String>,
    pub base_currency_rate: Option<f64>,
    pub post_code: String,
    pub address: String,
    pub address_detail: String,
    #[sea_orm(default = None)]
    pub sub: Option<String>,
    pub live_detail_id: String,
}
impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::None)",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
pub enum PlaceFulfillmentType {
    SelfEmployed, // 자영업 FC
    Subdivision,  // 소분 FC
    Distribution, // 중앙 FC
    Customer,     // 고객
    Indirect,     // 택배 대신 FC
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum PlaceFulfillmentStatus {
    Active = 0,
    Closed = -1,
    Suspended = -2,
    LawEnforcementSanction = -3,
    Breakdown = -4,
    InventoryQuantityMismatch = -5,
    ExceededQuantityLimit = 1,
    ExtremeWeatherProblem = 2,
    Delayed = 3,
    DelayedOverOneHour = 4,
}
