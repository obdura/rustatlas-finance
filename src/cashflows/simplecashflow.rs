use serde::{Deserialize, Serialize};

use crate::{
    core::{
        meta::{DiscountFactorRequest, ExchangeRateRequest, MarketRequest},
        traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId, Registrable},
    },
    currencies::enums::Currency,
    time::date::Date,
    utils::errors::{AtlasError, Result},
};

use super::{
    side::Side,
    traits::{Expires, Payable, Scalable},
};

/// # SimpleCashflow
/// A simple cashflow that is payable at a given date.
///
/// ## Example
/// ```
/// use rustatlas::prelude::*;
/// let payment_date = Date::new(2020, 1, 1);
/// let cashflow = SimpleCashflow::new(payment_date, Currency::USD, Side::Receive).with_amount(100.0);
/// assert_eq!(cashflow.side(), Side::Receive);
/// assert_eq!(cashflow.payment_date(), payment_date);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimpleCashflow {
    payment_date: Date,
    exchange_fixing_date: Option<Date>,
    currency: Currency, // currency of amount
    payment_currency: Option<Currency>,
    side: Side,
    amount: Option<f64>,
    discount_curve_id: Option<usize>,
    id: Option<usize>,
}

impl SimpleCashflow {
    pub fn new(payment_date: Date, currency: Currency, side: Side) -> SimpleCashflow {
        SimpleCashflow {
            payment_date,
            exchange_fixing_date: Some(payment_date),
            currency,
            payment_currency: None,
            side,
            amount: None,
            discount_curve_id: None,
            id: None,
        }
    }

    pub fn with_amount(mut self, amount: f64) -> SimpleCashflow {
        self.amount = Some(amount);
        self
    }

    pub fn with_exchange_fixing_date(mut self, date: Date) -> SimpleCashflow {
        self.exchange_fixing_date = Some(date);
        self
    }

    pub fn with_discount_curve_id(mut self, discount_curve_id: usize) -> SimpleCashflow {
        self.discount_curve_id = Some(discount_curve_id);
        self
    }

    pub fn with_id(mut self, registry_id: usize) -> SimpleCashflow {
        self.id = Some(registry_id);
        self
    }

    pub fn with_payment_currency(mut self, currency: Currency) -> SimpleCashflow {
        self.payment_currency = Some(currency);
        self
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        self.discount_curve_id = Some(id);
    }

    pub fn set_amount(&mut self, amount: f64) {
        self.amount = Some(amount);
    }

    pub fn set_payment_currency(&mut self, currency: Currency) {
        self.payment_currency = Some(currency);
    }
}

impl HasCurrency for SimpleCashflow {
    fn currency(&self) -> Result<Currency> {
        return Ok(self.currency);
    }
}

impl HasDiscountCurveId for SimpleCashflow {
    fn discount_curve_id(&self) -> Result<usize> {
        return self
            .discount_curve_id
            .ok_or(AtlasError::ValueNotSetErr("Discount curve id".to_string()));
    }
}

impl HasForecastCurveId for SimpleCashflow {
    fn forecast_curve_id(&self) -> Result<usize> {
        return Err(AtlasError::InvalidValueErr(
            "No forecast curve id for simple cashflow".to_string(),
        ));
    }
}

impl Registrable for SimpleCashflow {
    fn id(&self) -> Result<usize> {
        return self.id.ok_or(AtlasError::ValueNotSetErr("Id".to_string()));
    }

    fn set_id(&mut self, id: usize) {
        self.id = Some(id);
    }

    fn market_request(&self) -> Result<MarketRequest> {
        let id = self.id()?;
        let currency_request = ExchangeRateRequest::new(
            self.payment_currency()?,
            None, 
            None,
        );
        let currency_fwd_request = ExchangeRateRequest::new(
            self.payment_currency()?, 
            Some(self.currency()?), 
            Some(self.exchange_fixing_date()?),
        );
        let mut discount_request = DiscountFactorRequest::new(
            self.discount_curve_id()?, 
            self.payment_date
        );
        discount_request.set_discount_currency(self.payment_currency()?);

        return Ok(MarketRequest::new(
            id,
            Some(discount_request),
            None,
            Some(currency_request),
            Some(currency_fwd_request),
        ));
    }
}

impl Payable for SimpleCashflow {
    fn amount(&self) -> Result<f64> {
        return self.amount.ok_or(AtlasError::ValueNotSetErr(
            "Amount not set for simple cashflow".to_string(),
        ));
    }
    fn side(&self) -> Side {
        return self.side;
    }
    fn payment_date(&self) -> Date {
        return self.payment_date;
    }
    fn payment_currency(&self) -> Result<Currency> {
        return Ok(self.payment_currency.unwrap_or(self.currency));
    }
    fn exchange_fixing_date(&self) -> Result<Date> {
        return Ok(self.exchange_fixing_date.unwrap_or(self.payment_date));
    }
}

impl Expires for SimpleCashflow {
    fn is_expired(&self, date: Date) -> bool {
        return self.payment_date < date;
    }
}

impl Scalable for SimpleCashflow {
    fn scale(&mut self, factor: f64) -> Result<()> {
        let amount = self.amount.clone();
        match amount {
            Some(am) => self.set_amount(am * factor),
            None => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_trait_for_simple_cashflow() -> Result<()> {
        let date = Date::new(1993, 8, 6);
        let currency = Currency::CLP;
        let side = Side::Receive;
        let mut cashflow = SimpleCashflow::new(date, currency, side).with_amount(1_000_000.0);

        cashflow.scale(2.5)?;

        assert!((cashflow.amount()? - 2.5 * 1_000_000.0).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_scale_trait_for_simple_cashflow_with_none_amount() -> Result<()> {
        let date = Date::new(1993, 8, 6);
        let currency = Currency::CLP;
        let side = Side::Receive;
        let mut cashflow = SimpleCashflow::new(date, currency, side);

        cashflow.scale(2.5)?;

        assert!(cashflow.amount == None);
        Ok(())
    }
}
