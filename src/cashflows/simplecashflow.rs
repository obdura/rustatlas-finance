use serde::{Deserialize, Serialize};

use crate::{
    core::{
        meta::{DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest},
        traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId, Registrable},
    },
    currencies::{
        enums::Currency,
        exchangerategeneration::{ExchangeGenerationMethod, SingleDate},
    },
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimpleCashflow {
    payment_date: Date,
    exchange_fixing_method: Option<ExchangeGenerationMethod>,
    currency: Currency,
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
            exchange_fixing_method: None,
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

    pub fn with_exchange_fixing_method(
        mut self,
        method: ExchangeGenerationMethod,
    ) -> SimpleCashflow {
        self.exchange_fixing_method = Some(method);
        self
    }

    pub fn with_exchange_fixing_date(mut self, date: Date) -> SimpleCashflow {
        self.exchange_fixing_method =
            Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(date)));
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

    pub fn set_exchange_fixing_method(&mut self, method: ExchangeGenerationMethod) {
        self.exchange_fixing_method = Some(method);
    }

    pub fn set_exchange_fixing_date(&mut self, date: Date) {
        self.exchange_fixing_method =
            Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(date)));
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

    fn df_request(&self) -> Result<Option<DiscountFactorRequest>> {
        let mut discount_request =
            DiscountFactorRequest::new(self.discount_curve_id()?, self.payment_date);

        discount_request.set_discount_currency(self.payment_currency()?);
        Ok(Some(discount_request))
    }

    fn fwd_request(&self) -> Result<Option<ForwardRateRequest>> {
        Ok(None)
    }

    fn fx_request(&self) -> Result<Option<ExchangeRateRequest>> {
        let currency_request =
            ExchangeRateRequest::new_with_method(self.payment_currency()?, None, None);
        Ok(Some(currency_request))
    }

    fn fx_fwd_request(&self) -> Result<Option<ExchangeRateRequest>> {
        let currency_fwd_request = ExchangeRateRequest::new_with_method(
            self.payment_currency()?,
            Some(self.currency()?),
            self.exchange_fixing_method()?.clone(),
        );
        Ok(Some(currency_fwd_request))
    }

    fn fx_fixing_request(&self) -> Result<Option<ExchangeRateRequest>> {
        Ok(None)
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
    fn exchange_fixing_method(&self) -> Result<&Option<ExchangeGenerationMethod>> {
        return Ok(&self.exchange_fixing_method);
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

    #[test]
    fn test_with_methods_chain() -> Result<()> {
        let date = Date::new(2022, 12, 31);
        let currency = Currency::USD;
        let side = Side::Pay;
        let payment_currency = Currency::EUR;
        let exchange_fixing_date = Date::new(2022, 12, 15);
        let discount_curve_id = 42;
        let id = 99;
        let amount = 12345.67;

        let cashflow = SimpleCashflow::new(date, currency, side)
            .with_amount(amount)
            .with_payment_currency(payment_currency)
            .with_exchange_fixing_date(exchange_fixing_date)
            .with_discount_curve_id(discount_curve_id)
            .with_id(id);

        assert_eq!(cashflow.payment_date(), date);
        assert_eq!(cashflow.currency()?, currency);
        assert_eq!(cashflow.side(), side);
        assert_eq!(cashflow.amount()?, amount);
        assert_eq!(cashflow.payment_currency()?, payment_currency);
        assert_eq!(
            cashflow.exchange_fixing_method()?,
            &Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(
                exchange_fixing_date
            )))
        );
        assert_eq!(cashflow.discount_curve_id()?, discount_curve_id);
        assert_eq!(cashflow.id()?, id);
        Ok(())
    }

    #[test]
    fn test_setters() -> Result<()> {
        let date = Date::new(2023, 1, 1);
        let mut cashflow = SimpleCashflow::new(date, Currency::CLP, Side::Receive);

        cashflow.set_amount(500.0);
        cashflow.set_discount_curve_id(7);
        cashflow.set_payment_currency(Currency::USD);
        let fixing_date = Date::new(2023, 1, 2);
        cashflow.set_exchange_fixing_date(fixing_date);

        assert_eq!(cashflow.amount()?, 500.0);
        assert_eq!(cashflow.discount_curve_id()?, 7);
        assert_eq!(cashflow.payment_currency()?, Currency::USD);
        assert_eq!(
            cashflow.exchange_fixing_method()?,
            &Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(
                fixing_date
            )))
        );
        Ok(())
    }

    #[test]
    fn test_is_expired() {
        let date = Date::new(2020, 5, 20);
        let cashflow = SimpleCashflow::new(date, Currency::USD, Side::Pay);

        assert!(!cashflow.is_expired(Date::new(2020, 5, 20)));
        assert!(!cashflow.is_expired(Date::new(2020, 5, 19)));
        assert!(cashflow.is_expired(Date::new(2020, 5, 21)));
    }

    #[test]
    fn test_forecast_curve_id_returns_error() {
        let date = Date::new(2021, 7, 15);
        let cashflow = SimpleCashflow::new(date, Currency::USD, Side::Pay);
        let result = cashflow.forecast_curve_id();
        assert!(result.is_err());
    }

    #[test]
    fn test_discount_curve_id_error() {
        let date = Date::new(2021, 7, 15);
        let cashflow = SimpleCashflow::new(date, Currency::USD, Side::Pay);
        let result = cashflow.discount_curve_id();
        assert!(result.is_err());
    }

    #[test]
    fn test_id_error() {
        let date = Date::new(2021, 7, 15);
        let cashflow = SimpleCashflow::new(date, Currency::USD, Side::Pay);
        let result = cashflow.id();
        assert!(result.is_err());
    }

    #[test]
    fn test_amount_error() {
        let date = Date::new(2021, 7, 15);
        let cashflow = SimpleCashflow::new(date, Currency::USD, Side::Pay);
        let result = cashflow.amount();
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_currency_defaults_to_currency() -> Result<()> {
        let date = Date::new(2022, 2, 2);
        let cashflow = SimpleCashflow::new(date, Currency::EUR, Side::Receive).with_amount(100.0);
        assert_eq!(cashflow.payment_currency()?, Currency::EUR);
        Ok(())
    }

    #[test]
    fn test_exchange_fixing_date_defaults_to_payment_date() -> Result<()> {
        let date = Date::new(2022, 2, 2);
        let cashflow = SimpleCashflow::new(date, Currency::EUR, Side::Receive).with_amount(100.0);
        assert_eq!(cashflow.exchange_fixing_method()?, &None);
        Ok(())
    }
}
