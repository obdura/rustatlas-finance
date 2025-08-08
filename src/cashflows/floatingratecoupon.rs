use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    core::{
        meta::{ForwardRateRequest, MarketRequest},
        traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId, Registrable},
    },
    currencies::enums::Currency,
    rates::interestrate::{InterestRate, RateDefinition},
    time::{date::Date, enums::TimeUnit, period::Period},
    utils::errors::{AtlasError, Result},
};

use super::{
    side::Side,
    simplecashflow::SimpleCashflow,
    traits::{Expires, InterestAccrual, Payable, RequiresFixingRate, Scalable},
};

/// # FloatingRateCoupon
/// A floating rate coupon is a cashflow that pays a floating rate of interest on a notional amount.
///
/// ## Parameters
/// * `notional` - The notional amount of the coupon
/// * `spread` - The spread over the floating rate
/// * `accrual_start_date` - The date from which the coupon accrues interest
/// * `accrual_end_date` - The date until which the coupon accrues interest
/// * `payment_date` - The date on which the coupon is paid
/// * `fixing_date` - The date from which the floating rate is observed
/// * `rate_definition` - The definition of the floating rate
/// * `discount_curve_id` - The ID of the discount curve used to calculate the present value of the coupon
/// * `forecast_curve_id` - The ID of the forecast curve used to calculate the present value of the coupon
/// * `currency` - The currency of the coupon
/// * `side` - The side of the coupon (Pay or Receive)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatingRateCoupon {
    notional: f64,
    spread: f64,
    accrual_start_date: Date,
    accrual_end_date: Date,
    fixing_start_date: Option<Date>,
    fixing_end_date: Option<Date>,
    rate_definition: RateDefinition,
    cashflow: SimpleCashflow,
    fixing_rate: Option<f64>,
    forecast_curve_id: Option<usize>,
}

impl FloatingRateCoupon {
    pub fn new(
        notional: f64,
        spread: f64,
        accrual_start_date: Date,
        accrual_end_date: Date,
        payment_date: Date,
        rate_definition: RateDefinition,
        currency: Currency,
        side: Side,
    ) -> FloatingRateCoupon {
        FloatingRateCoupon {
            notional,
            spread,
            fixing_rate: None,
            accrual_start_date,
            accrual_end_date,
            fixing_start_date: Some(accrual_start_date),
            fixing_end_date: Some(accrual_end_date),
            rate_definition,
            forecast_curve_id: None,
            cashflow: SimpleCashflow::new(payment_date, currency, side),
        }
    }

    pub fn with_fixing_dates(
        &mut self,
        fixing_start_date: Date,
        fixing_end_date: Date,
    ) -> &mut FloatingRateCoupon {
        self.fixing_start_date = Some(fixing_start_date);
        self.fixing_end_date = Some(fixing_end_date);
        self
    }

    pub fn set_fixing_dates(&mut self, fixing_start_date: Date, fixing_end_date: Date) {
        self.fixing_start_date = Some(fixing_start_date);
        self.fixing_end_date = Some(fixing_end_date);
    }

    pub fn with_discount_curve_id(mut self, id: usize) -> FloatingRateCoupon {
        self.cashflow.set_discount_curve_id(id);
        self
    }

    pub fn with_forecast_curve_id(mut self, id: usize) -> FloatingRateCoupon {
        self.forecast_curve_id = Some(id);
        self
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        self.cashflow.set_discount_curve_id(id);
    }

    pub fn set_forecast_curve_id(&mut self, id: usize) {
        self.forecast_curve_id = Some(id);
    }

    pub fn set_spread(&mut self, spread: f64) {
        self.spread = spread;
        // if fixing rate is set, update the cashflow
        match self.fixing_rate {
            Some(fixing_rate) => {
                self.set_fixing_rate(fixing_rate);
            }
            None => {}
        }
    }

    pub fn set_notional(&mut self, notional: f64) {
        self.notional = notional;
    }

    pub fn notional(&self) -> f64 {
        self.notional
    }

    pub fn spread(&self) -> f64 {
        self.spread
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
    }

    pub fn fixing_rate(&self) -> Option<f64> {
        self.fixing_rate
    }

    pub fn with_exchange_fixing_date(&mut self, date: Date) -> &mut FloatingRateCoupon {
        self.cashflow.with_exchange_fixing_date(date);
        self
    }

    pub fn with_payment_currency(&mut self, currency: Currency) -> &mut FloatingRateCoupon {
        self.cashflow.set_payment_currency(currency);
        self
    }

    pub fn set_exchange_fixing_date(&mut self, date: Date) {
        self.cashflow.set_exchange_fixing_date(date);
    }

    pub fn set_payment_currency(&mut self, currency: Currency) {
        self.cashflow.set_payment_currency(currency);
    }
}

impl InterestAccrual for FloatingRateCoupon {
    fn accrual_start_date(&self) -> Result<Date> {
        return Ok(self.accrual_start_date);
    }
    fn accrual_end_date(&self) -> Result<Date> {
        return Ok(self.accrual_end_date);
    }
    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let fixing = self
            .fixing_rate
            .ok_or(AtlasError::ValueNotSetErr("Fixing rate".to_string()))?;
        let rate = InterestRate::from_rate_definition(fixing + self.spread, self.rate_definition);

        let (d1, d2) = self.relevant_accrual_dates(self.accrual_start_date, end_date)?;
        let acc_1 = self.notional * (rate.compound_factor(d1, d2) - 1.0);

        let (d1, d2) = self.relevant_accrual_dates(self.accrual_start_date, start_date)?;
        let acc_2 = self.notional * (rate.compound_factor(d1, d2) - 1.0);
        return Ok((acc_1 - acc_2) * self.cashflow.side().sign());
    }

    fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>> {
        let delta = Period::new(1, TimeUnit::Days);
        let mut start_date = self.accrual_start_date().unwrap();
        let end_date = self.accrual_end_date().unwrap();
        let mut map = BTreeMap::new();
        map.insert(start_date, 0.0);
        while start_date < end_date {
            let accrued = self.accrued_amount(start_date, start_date + delta).unwrap();
            let entry = map.entry(start_date + delta).or_insert(0.0);
            *entry += accrued;
            start_date = start_date + delta;
        }
        Ok(map)
    }
}

impl RequiresFixingRate for FloatingRateCoupon {
    fn set_fixing_rate(&mut self, fixing_rate: f64) {
        self.fixing_rate = Some(fixing_rate);
        let accrual = self
            .accrued_amount(self.accrual_start_date, self.accrual_end_date)
            .unwrap()
            * self.side().sign();
        self.cashflow = self.cashflow.with_amount(accrual);
    }

    fn fixing_start_date(&self) -> Result<Option<Date>> {
        Ok(Some(
            self.fixing_start_date.unwrap_or(self.accrual_start_date),
        ))
    }

    fn fixing_end_date(&self) -> Result<Option<Date>> {
        Ok(Some(self.fixing_end_date.unwrap_or(self.accrual_end_date)))
    }
}

impl Payable for FloatingRateCoupon {
    fn amount(&self) -> Result<f64> {
        return self.cashflow.amount();
    }
    fn side(&self) -> Side {
        return self.cashflow.side();
    }
    fn payment_date(&self) -> Date {
        return self.cashflow.payment_date();
    }
    fn payment_currency(&self) -> Result<Currency> {
        return self.cashflow.payment_currency();
    }
    fn exchange_fixing_date(&self) -> Result<Date> {
        return self.cashflow.exchange_fixing_date();
    }
}

impl HasCurrency for FloatingRateCoupon {
    fn currency(&self) -> Result<Currency> {
        self.cashflow.currency()
    }
}

impl HasDiscountCurveId for FloatingRateCoupon {
    fn discount_curve_id(&self) -> Result<usize> {
        self.cashflow.discount_curve_id()
    }
}

impl HasForecastCurveId for FloatingRateCoupon {
    fn forecast_curve_id(&self) -> Result<usize> {
        self.forecast_curve_id
            .ok_or(AtlasError::ValueNotSetErr("Forecast curve id".to_string()))
    }
}

impl Registrable for FloatingRateCoupon {
    fn id(&self) -> Result<usize> {
        self.cashflow.id()
    }

    fn set_id(&mut self, id: usize) {
        self.cashflow.set_id(id);
    }

    fn market_request(&self) -> Result<MarketRequest> {
        let tmp = self.cashflow.market_request()?;
        let forecast_curve_id = self.forecast_curve_id()?;

        let fixing_start_date = self.fixing_start_date()?.ok_or(AtlasError::ValueNotSetErr(
            "Fixing start date not set".to_string(),
        ))?;
        let fixing_end_date = self.fixing_end_date()?.ok_or(AtlasError::ValueNotSetErr(
            "Fixing end date not set".to_string(),
        ))?;

        if self.fixing_start_date()? >= self.fixing_end_date()? {
            return Err(AtlasError::InvalidValueErr(format!(
                "Fixing start date {} is after or equal to fixing end date {}",
                fixing_start_date, fixing_end_date
            )));
        }

        let forecast = ForwardRateRequest::new(
            forecast_curve_id,
            fixing_start_date,
            fixing_end_date,
            self.rate_definition.compounding(),
            self.rate_definition.frequency(),
        );
        Ok(MarketRequest::new(
            tmp.id(),
            tmp.df(),
            Some(forecast),
            tmp.fx(),
            tmp.fx_fwd(),
        ))
    }
}

impl Expires for FloatingRateCoupon {
    fn is_expired(&self, date: Date) -> bool {
        self.cashflow.payment_date() < date
    }
}

impl Scalable for FloatingRateCoupon {
    fn scale(&mut self, factor: f64) -> Result<()> {
        let notional = self.notional();
        self.notional = notional * factor;
        self.cashflow.scale(factor)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        rates::enums::Compounding,
        time::{daycounter::DayCounter, enums::Frequency},
    };

    use super::*;

    #[test]
    fn test_scale_trait_floating_rate_coupon() -> Result<()> {
        let notional = 1_000.0;
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let spread = 0.02;

        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::JPY;

        let mut coupon = FloatingRateCoupon::new(
            notional,
            spread,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            rate_definition,
            currency,
            Side::Pay,
        );

        coupon.set_fixing_rate(0.03);

        coupon.scale(2.5)?;
        assert!((coupon.notional() - 1_000.0 * 2.5).abs() < 1e-6);
        assert!((coupon.amount()? - 50.0 * 2.5).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_new_and_setters() -> Result<()> {
        let notional = 5000.0;
        let spread = 0.01;
        let accrual_start_date = Date::new(2022, 6, 1);
        let accrual_end_date = Date::new(2022, 12, 1);
        let payment_date = Date::new(2022, 12, 2);
        let currency = Currency::USD;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let mut coupon = FloatingRateCoupon::new(
            notional,
            spread,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            rate_definition,
            currency,
            Side::Receive,
        );

        assert_eq!(coupon.notional(), notional);
        assert_eq!(coupon.spread(), spread);
        assert_eq!(coupon.rate_definition(), rate_definition);

        coupon.set_notional(10000.0);
        assert_eq!(coupon.notional(), 10000.0);

        coupon.set_spread(0.015);
        assert_eq!(coupon.spread(), 0.015);

        Ok(())
    }

    #[test]
    fn test_fixing_rate_and_amount() -> Result<()> {
        let notional = 2000.0;
        let spread = 0.005;
        let accrual_start_date = Date::new(2023, 3, 1);
        let accrual_end_date = Date::new(2023, 9, 1);
        let payment_date = Date::new(2023, 9, 2);
        let currency = Currency::EUR;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let mut coupon = FloatingRateCoupon::new(
            notional,
            spread,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            rate_definition,
            currency,
            Side::Receive,
        );

        coupon.set_fixing_rate(0.02);
        let amount = coupon.amount()?;
        assert!(amount > 0.0);

        Ok(())
    }

    #[test]
    fn test_accrued_amount_map() -> Result<()> {
        let notional = 1000.0;
        let spread = 0.01;
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 1, 5);
        let payment_date = Date::new(2023, 1, 6);
        let currency = Currency::GBP;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let mut coupon = FloatingRateCoupon::new(
            notional,
            spread,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            rate_definition,
            currency,
            Side::Pay,
        );

        coupon.set_fixing_rate(0.015);

        let map = coupon.accrued_amount_map()?;
        assert_eq!(map.len(), 5);
        assert!(map.values().all(|v| v.is_finite()));

        Ok(())
    }

    #[test]
    fn test_with_fixing_dates_and_forecast_curve_id() -> Result<()> {
        let notional = 1000.0;
        let spread = 0.01;
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 1, 10);
        let payment_date = Date::new(2023, 1, 11);
        let currency = Currency::CHF;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let mut coupon = FloatingRateCoupon::new(
            notional,
            spread,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            rate_definition,
            currency,
            Side::Receive,
        );

        coupon.set_fixing_dates(Date::new(2023, 1, 2), Date::new(2023, 1, 9));
        coupon.set_forecast_curve_id(42);

        assert_eq!(coupon.fixing_start_date()?, Some(Date::new(2023, 1, 2)));
        assert_eq!(coupon.fixing_end_date()?, Some(Date::new(2023, 1, 9)));
        assert_eq!(coupon.forecast_curve_id()?, 42);

        Ok(())
    }

    #[test]
    fn test_is_expired() {
        let notional = 1000.0;
        let spread = 0.01;
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 1, 10);
        let payment_date = Date::new(2023, 1, 11);
        let currency = Currency::USD;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let coupon = FloatingRateCoupon::new(
            notional,
            spread,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            rate_definition,
            currency,
            Side::Pay,
        );

        assert!(!coupon.is_expired(Date::new(2023, 1, 10)));
        assert!(coupon.is_expired(Date::new(2023, 1, 12)));
    }
}
