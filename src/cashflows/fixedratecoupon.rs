use std::collections::BTreeMap;

use super::side::Side;
use super::simplecashflow::SimpleCashflow;
use super::traits::{Expires, InterestAccrual, Payable, Scalable};
use crate::core::traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId};
use crate::time::enums::TimeUnit;
use crate::time::period::Period;
use crate::utils::errors::AtlasError;
use crate::{
    core::{meta::MarketRequest, traits::Registrable},
    currencies::enums::Currency,
    rates::interestrate::InterestRate,
    time::date::Date,
    utils::errors::Result,
};
use serde::{Deserialize, Serialize};

/// # FixedRateCoupon
/// A fixed rate coupon is a cashflow that pays a fixed rate of interest on a notional amount.
///
/// ## Parameters
/// * `notional` - The notional amount of the coupon
/// * `rate` - The fixed rate of interest
/// * `accrual_start_date` - The date from which the coupon accrues interest
/// * `accrual_end_date` - The date until which the coupon accrues interest
/// * `payment_date` - The date on which the coupon is paid
/// * `currency` - The currency of the coupon
/// * `side` - The side of the coupon (Pay or Receive)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FixedRateCoupon {
    notional: f64,
    rate: InterestRate,
    accrual_start_date: Date,
    accrual_end_date: Date,
    cashflow: SimpleCashflow,
}

impl FixedRateCoupon {
    pub fn new(
        notional: f64,
        rate: InterestRate,
        accrual_start_date: Date,
        accrual_end_date: Date,
        payment_date: Date,
        currency: Currency,
        side: Side,
    ) -> FixedRateCoupon {
        let amount = notional * (rate.compound_factor(accrual_start_date, accrual_end_date) - 1.0);
        let cashflow = SimpleCashflow::new(payment_date, currency, side).with_amount(amount);
        FixedRateCoupon {
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            cashflow: cashflow,
        }
    }

    pub fn with_discount_curve_id(mut self, id: usize) -> FixedRateCoupon {
        self.cashflow.set_discount_curve_id(id);
        self
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        self.cashflow.set_discount_curve_id(id);
    }

    pub fn set_rate_value(&mut self, rate_value: f64) {
        let rate = InterestRate::from_rate_definition(rate_value, self.rate.rate_definition());
        self.set_rate(rate);
    }

    pub fn set_rate(&mut self, rate: InterestRate) {
        self.rate = rate;
        // Update the cashflow amount
        self.cashflow.set_amount(
            self.notional
                * (rate.compound_factor(self.accrual_start_date, self.accrual_end_date) - 1.0),
        );
    }

    pub fn set_notional(&mut self, notional: f64) {
        self.notional = notional;
        self.cashflow.set_amount(
            self.notional
                * (self
                    .rate
                    .compound_factor(self.accrual_start_date, self.accrual_end_date)
                    - 1.0),
        );
    }

    pub fn notional(&self) -> f64 {
        self.notional
    }

    pub fn rate(&self) -> InterestRate {
        self.rate
    }

    pub fn with_exchange_fixing_date(&mut self, date: Date) -> &mut FixedRateCoupon {
        self.cashflow.with_exchange_fixing_date(date);
        self
    }

    pub fn set_exchange_fixing_date(&mut self, date: Date) {
        self.cashflow.set_exchange_fixing_date(date);
    }

    pub fn with_payment_currency(&mut self, currency: Currency) -> &mut FixedRateCoupon {
        self.cashflow.set_payment_currency(currency);
        self
    }

    pub fn set_payment_currency(&mut self, currency: Currency) {
        self.cashflow.set_payment_currency(currency);
    }
}

impl HasCurrency for FixedRateCoupon {
    fn currency(&self) -> Result<Currency> {
        self.cashflow.currency()
    }
}

impl HasDiscountCurveId for FixedRateCoupon {
    fn discount_curve_id(&self) -> Result<usize> {
        self.cashflow.discount_curve_id()
    }
}

impl HasForecastCurveId for FixedRateCoupon {
    fn forecast_curve_id(&self) -> Result<usize> {
        return Err(AtlasError::InvalidValueErr(
            "No forecast curve id for fixed rate cashflow".to_string(),
        ));
    }
}

impl Registrable for FixedRateCoupon {
    fn id(&self) -> Result<usize> {
        return self.cashflow.id();
    }

    fn set_id(&mut self, id: usize) {
        self.cashflow.set_id(id);
    }

    fn market_request(&self) -> Result<MarketRequest> {
        return self.cashflow.market_request();
    }
}

impl InterestAccrual for FixedRateCoupon {
    fn accrual_start_date(&self) -> Result<Date> {
        return Ok(self.accrual_start_date);
    }

    fn accrual_end_date(&self) -> Result<Date> {
        return Ok(self.accrual_end_date);
    }

    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let (d1, d2) = self.relevant_accrual_dates(self.accrual_start_date, end_date)?;
        let acc_1 = self.notional * (self.rate.compound_factor(d1, d2) - 1.0);

        let (d1, d2) = self.relevant_accrual_dates(self.accrual_start_date, start_date)?;
        let acc_2 = self.notional * (self.rate.compound_factor(d1, d2) - 1.0);

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

impl Payable for FixedRateCoupon {
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

impl Expires for FixedRateCoupon {
    fn is_expired(&self, date: Date) -> bool {
        return self.cashflow.is_expired(date);
    }
}

impl Scalable for FixedRateCoupon {
    fn scale(&mut self, factor: f64) -> Result<()> {
        let notional = self.notional();
        self.notional = notional * factor;
        self.cashflow.scale(factor)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::currencies::enums::Currency;
    use crate::rates::enums::Compounding;
    use crate::rates::interestrate::InterestRate;
    use crate::time::date::Date;
    use crate::time::daycounter::DayCounter;
    use crate::time::enums::Frequency;

    #[test]
    fn test_fixed_rate_coupon_creation() -> Result<()> {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::JPY;

        let coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        assert_eq!(coupon.accrual_start_date()?, accrual_start_date);
        assert_eq!(coupon.accrual_end_date()?, accrual_end_date);

        Ok(())
    }

    #[test]
    fn test_amount_calculation() {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let id = 1;
        let currency = Currency::USD;

        let mut coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        coupon.set_discount_curve_id(id);

        let expected_amount = notional
            * (rate.compound_factor(accrual_start_date, accrual_end_date) - 1.0)
            * Side::Pay.sign();
        assert_eq!(
            coupon
                .accrued_amount(accrual_start_date, accrual_end_date)
                .unwrap(),
            expected_amount
        );
    }

    #[test]
    fn test_accrual() {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 12, 10);
        let accrual_end_date = Date::new(2024, 3, 30);
        let payment_date = Date::new(2024, 1, 10);
        let id = 1;
        let currency = Currency::USD;

        let mut coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Receive,
        );

        coupon.set_discount_curve_id(id);

        let star_date = Date::new(2024, 2, 28);
        let end_date = Date::new(2024, 3, 1);
        let accrued_amount = coupon.accrued_amount(star_date, end_date).unwrap();

        print!(
            "Accrued amount between {} and {} is {}",
            star_date, end_date, accrued_amount
        );
    }

    #[test]
    fn test_scale_trait_for_fixed_rate_coupon() -> Result<()> {
        let notional = 1_000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::JPY;

        let mut coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        coupon.scale(2.5)?;
        assert!((coupon.notional() - 1_000.0 * 2.5).abs() < 1e-6);
        assert!((coupon.amount()? - 50.0 * 2.5).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_set_rate_value_updates_amount() {
        let notional = 1000.0;
        let initial_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::EUR;

        let mut coupon = FixedRateCoupon::new(
            notional,
            initial_rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Receive,
        );

        let new_rate_value = 0.06;
        coupon.set_rate_value(new_rate_value);

        let expected_amount = notional
            * (InterestRate::from_rate_definition(new_rate_value, initial_rate.rate_definition())
                .compound_factor(accrual_start_date, accrual_end_date)
                - 1.0);

        assert!((coupon.amount().unwrap() - expected_amount).abs() < 1e-10);
    }

    #[test]
    fn test_set_notional_updates_amount() {
        let notional = 500.0;
        let rate = InterestRate::new(
            0.04,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let accrual_start_date = Date::new(2022, 1, 1);
        let accrual_end_date = Date::new(2022, 12, 31);
        let payment_date = Date::new(2023, 1, 1);
        let currency = Currency::GBP;

        let mut coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        let new_notional = 800.0;
        coupon.set_notional(new_notional);

        let expected_amount = new_notional
            * (rate.compound_factor(accrual_start_date, accrual_end_date) - 1.0);

        assert!((coupon.amount().unwrap() - expected_amount).abs() < 1e-10);
    }

    #[test]
    fn test_set_exchange_fixing_date_and_payment_currency() {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::JPY;

        let mut coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        let fixing_date = Date::new(2023, 12, 15);
        let payment_currency = Currency::USD;

        coupon.set_exchange_fixing_date(fixing_date);
        coupon.set_payment_currency(payment_currency);

        assert_eq!(coupon.exchange_fixing_date().unwrap(), fixing_date);
        assert_eq!(coupon.payment_currency().unwrap(), payment_currency);
    }

    #[test]
    fn test_is_expired_trait() {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::JPY;

        let coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        let before_payment = Date::new(2023, 12, 31);
        let after_payment = Date::new(2024, 1, 2);

        assert!(!coupon.is_expired(before_payment));
        assert!(coupon.is_expired(after_payment));
    }

    #[test]
    fn test_forecast_curve_id_returns_error() {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 12, 31);
        let payment_date = Date::new(2024, 1, 1);
        let currency = Currency::JPY;

        let coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Pay,
        );

        let result = coupon.forecast_curve_id();
        assert!(result.is_err());
    }

    #[test]
    fn test_accrued_amount_map_monotonicity() {
        let notional = 1000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 1, 10);
        let payment_date = Date::new(2023, 1, 11);
        let currency = Currency::USD;

        let coupon = FixedRateCoupon::new(
            notional,
            rate,
            accrual_start_date,
            accrual_end_date,
            payment_date,
            currency,
            Side::Receive,
        );

        let map = coupon.accrued_amount_map().unwrap();
        let mut prev = 0.0;
        for (_date, amount) in map.iter() {
            assert!(*amount >= prev);
            prev = *amount;
        }
    }
}
