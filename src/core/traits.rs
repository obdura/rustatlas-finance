use super::meta::{ExchangeRateRequest, MarketRequest};
use crate::{currencies::enums::Currency, models::traits::Model, time::date, utils::errors::Result};


/// # HasCurrency
/// A trait for objects that have a currency.
/// 
/// * This trait is used to get the currency of an object.
/// * It can also be used to get the exchange rate between the currency of the object and another currency using a model.
pub trait HasCurrency {
    fn currency(&self) -> Result<Currency>;
    fn fx_fwd_change<T: Model>(&self, second_currency: Currency, date: date::Date, model: &T) -> Result<f64> {
        let first_currency = self.currency()?;
        if first_currency == second_currency {
            return Ok(1.0);
        }
        let request = ExchangeRateRequest::new(first_currency, Some(second_currency), Some(date));
        let fx= model.gen_fx_data(request)?; 
        Ok(fx)
    }    
}

/// # HasDiscountCurveId
/// A trait for objects that have a discount curve id.
/// 
/// * This trait is used to get the discount curve id of an object.
/// 
pub trait HasDiscountCurveId {
    fn discount_curve_id(&self) -> Result<usize>;
}

/// # HasForecastCurveId
/// A trait for objects that have a forecast curve id.
/// 
/// * This trait is used to get the forecast curve id of an object.
pub trait HasForecastCurveId {
    fn forecast_curve_id(&self) -> Result<usize>;
}

/// # Registrable
/// A trait for objects that can be registered for market data.
/// 
/// * This trait is used to get the id of an object, set the id of an object, and get the market request of an object.
pub trait Registrable: HasDiscountCurveId + HasForecastCurveId + HasCurrency {
    fn id(&self) -> Result<usize>;
    fn set_id(&mut self, id: usize);
    fn market_request(&self) -> Result<MarketRequest>;
}
