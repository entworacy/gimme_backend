use crate::modules::place::entities::place_parent::PlaceFulfillmentStatus;
use crate::modules::place::entities::place_parent::PlaceFulfillmentType;

pub struct PlaceDomain {
    pub id: i32,
    pub place_name: String,
    pub fulfillment_type: PlaceFulfillmentType,
    pub fulfillment_status: PlaceFulfillmentStatus,
    pub open_time: Vec<u32>,
    pub close_time: Vec<u32>,
    pub is_public: bool,
    pub fc_able_split_shipping: bool,
    pub min_shipping_amount_krw: Option<u32>,
    pub base_currency_code: Option<String>,
    pub base_currency_rate: Option<f64>,
    pub post_code: String,
    pub address: String,
    pub address_detail: String,
    pub sub: Option<String>,
}

impl PlaceDomain {
    pub fn can_order(&self) -> bool {
        (self.fulfillment_status == PlaceFulfillmentStatus::Active
            || self.fulfillment_status == PlaceFulfillmentStatus::Delayed
            || self.fulfillment_status == PlaceFulfillmentStatus::DelayedOverOneHour
            || self.fulfillment_status == PlaceFulfillmentStatus::ExtremeWeatherProblem)
            && self.is_public
            && self.fulfillment_type != PlaceFulfillmentType::Customer
    }

    pub fn set_amount(&self, amount: i32) -> Result<(), String> {
        if amount < 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        let mut amount: f64 = amount as f64;
        if self.base_currency_code.is_some() && self.base_currency_code.as_ref().unwrap() != "KRW" {
            // krw기반계산이 아니면 별도 처리를 해줘야한다.
            // Ex) 1$ = 1,300이면 base_currency_rate = 1300
            // amount는 해당 currency의 금액으로 들어오기 때문에
            // min_currency_krw = 1300이면 min_currency의 실제 코드는
            // min_curreny_real = min_currency_krw / base_currency_rate
            let min_currency_real =
                self.min_shipping_amount_krw.unwrap() as f64 / self.base_currency_rate.unwrap();
        }

        Ok(())
    }
}
