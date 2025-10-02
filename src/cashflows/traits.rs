use std::collections::BTreeMap;

use crate::{currencies::{enums::Currency, exchangerategeneration::ExchangeGenerationMethod}, time::date::Date, utils::errors::Result};

use super::side::Side;

/// # InterestAccrual
/// A trait that defines the accrual period of an instrument.
///
/// ## Functions
/// * `accrual_start_date` - The start date of the accrual period.
/// * `accrual_end_date` - The end date of the accrual period.
/// * `accrued_amount` - The accrued amount between two dates.
/// * `relevant_accrual_dates` - The relevant accrual dates between two dates.
/// * `accrued_amount_map` - The accrued amount map. DEFAULT IMPLEMENTATION PROVIDED, UASEFULLF FOR CASHFLOWS, TRY TO OVERRIDE FOR PERFORMANCE.
///
pub trait InterestAccrual {
    fn accrual_start_date(&self) -> Result<Date>;
    fn accrual_end_date(&self) -> Result<Date>;
    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64>;
    fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>>;
    fn relevant_accrual_dates(&self, start_date: Date, end_date: Date) -> Result<(Date, Date)> {
        let accrual_start = self.accrual_start_date()?;
        let accrual_end = self.accrual_end_date()?;

        // Check if the ranges intersect
        if start_date <= accrual_end && end_date >= accrual_start {
            // The ranges intersect, so we find the relevant accrual dates
            let relevant_start = if accrual_start >= start_date {
                accrual_start
            } else {
                start_date
            };

            let relevant_end = if accrual_end <= end_date {
                accrual_end
            } else {
                end_date
            };

            Ok((relevant_start, relevant_end))
        } else {
            // The ranges do not intersect, so return Date::empty()
            Ok((Date::empty(), Date::empty()))
        }
    }
}

/// # RequiresFixingRate
/// A trait that defines if an instrument requires a fixing rate.
pub trait RequiresFixingRate: InterestAccrual {
    fn set_fixing_rate(&mut self, fixing_rate: f64);

    fn fixing_start_date(&self) -> Result<Option<Date>> {
        self.accrual_start_date().map(Some)
    }
    
    fn fixing_end_date(&self) -> Result<Option<Date>> {
        self.accrual_end_date().map(Some)
    }
}

/// #  RequiresFixingExchangeRate
/// A trait for objects that have a fixing exchange rate.
pub trait RequiresFixingExchangeRate {
    fn set_fixing_exchange_rate(&mut self, fixing_exchange_rate: f64);
}

/// # Payable
/// A trait that defines the payment of an instrument.
pub trait Payable {
    fn amount(&self) -> Result<f64>;
    fn side(&self) -> Side;
    fn payment_date(&self) -> Date;
    fn payment_currency(&self) -> Result<Currency>;
    fn exchange_fixing_method(&self) -> Result<&Option<ExchangeGenerationMethod>>;
}

/// # Expires
/// A trait that defines if an instrument expires.
pub trait Expires {
    fn is_expired(&self, date: Date) -> bool;
}

/// # Scalable
/// A trait that scale a cashflow
pub trait Scalable {
    fn scale(&mut self, factor: f64) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::{
        cashflows::fixedratecoupon::FixedRateCoupon,
        currencies::enums::Currency,
        rates::{enums::Compounding, interestrate::InterestRate},
        time::{daycounter::DayCounter, enums::Frequency},
    };

    use super::*;

    #[test]
    fn test_delta_accrued_amount_simple() {
        let notional = 10000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 1);
        let accrual_end_date = Date::new(2023, 3, 31);
        let payment_date = Date::new(2023, 3, 31);
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

        let mut start_date = Date::new(2023, 1, 1);
        let mut end_date = Date::new(2023, 3, 31);
        let mut accrued_amount =
            coupon.accrued_amount(start_date, end_date).unwrap() * Side::Pay.sign();
        assert!((accrued_amount - 125.0).abs() < 0.00001);

        start_date = Date::new(2023, 1, 15);
        end_date = Date::new(2023, 1, 16);
        accrued_amount = coupon.accrued_amount(start_date, end_date).unwrap() * Side::Pay.sign();
        assert!((accrued_amount - 125.0 / 90.0).abs() < 0.00001);
    }

    #[test]
    fn test_delta_accrued_amount_compounded() {
        let notional = 10000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let accrual_start_date = Date::new(2023, 1, 30);
        let accrual_end_date = Date::new(2023, 3, 31);
        let payment_date = Date::new(2023, 3, 31);
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

        let start_date = Date::new(2023, 1, 30);
        let end_date = Date::new(2023, 3, 31);
        let accrued_amount = coupon.clone().accrued_amount(start_date, end_date).unwrap();

        assert!(accrued_amount - 122.72234429 < 0.00001);
    }

    #[test]
    fn test_requires_fixing_rate_trait_defaults() {
        struct DummyFixing {
            start: Date,
            end: Date,
            fixing_rate: f64,
        }
        impl InterestAccrual for DummyFixing {
            fn accrual_start_date(&self) -> Result<Date> {
                Ok(self.start)
            }
            fn accrual_end_date(&self) -> Result<Date> {
                Ok(self.end)
            }
            fn accrued_amount(&self, _start_date: Date, _end_date: Date) -> Result<f64> {
                Ok(self.fixing_rate)
            }
            fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>> {
                Ok(BTreeMap::new())
            }
        }
        impl RequiresFixingRate for DummyFixing {
            fn set_fixing_rate(&mut self, fixing_rate: f64) {
                self.fixing_rate = fixing_rate;
            }
        }

        let mut dummy = DummyFixing {
            start: Date::new(2023, 1, 1),
            end: Date::new(2023, 3, 31),
            fixing_rate: 0.0,
        };
        dummy.set_fixing_rate(0.07);
        assert_eq!(dummy.fixing_start_date().unwrap(), Some(Date::new(2023, 1, 1)));
        assert_eq!(dummy.fixing_end_date().unwrap(), Some(Date::new(2023, 3, 31)));
        assert_eq!(
            dummy
                .accrued_amount(Date::new(2023, 1, 1), Date::new(2023, 3, 31))
                .unwrap(),
            0.07
        );
    }

    #[test]
    fn test_payable_trait() {
        struct DummyPayable;
        impl Payable for DummyPayable {
            fn amount(&self) -> Result<f64> {
                Ok(100.0)
            }
            fn side(&self) -> Side {
                Side::Receive
            }
            fn payment_date(&self) -> Date {
                Date::new(2024, 1, 1)
            }
            fn payment_currency(&self) -> Result<Currency> {
                Ok(Currency::USD)
            }
            fn exchange_fixing_method(&self) -> Result<&Option<ExchangeGenerationMethod>> {
                Ok(&None)
            }	
        }

        let dummy = DummyPayable;
        assert_eq!(dummy.amount().unwrap(), 100.0);
        assert_eq!(dummy.side(), Side::Receive);
        assert_eq!(dummy.payment_date(), Date::new(2024, 1, 1));
        assert_eq!(dummy.payment_currency().unwrap(), Currency::USD);
        assert_eq!(
            dummy.exchange_fixing_method().unwrap(),
            &None
        );
    }

    #[test]
    fn test_expires_trait() {
        struct DummyExpires(Date);
        impl Expires for DummyExpires {
            fn is_expired(&self, date: Date) -> bool {
                date > self.0
            }
        }
        let dummy = DummyExpires(Date::new(2023, 6, 30));
        assert!(!dummy.is_expired(Date::new(2023, 6, 30)));
        assert!(dummy.is_expired(Date::new(2023, 7, 1)));
    }

    #[test]
    fn test_scalable_trait() {
        struct DummyScalable {
            value: f64,
        }
        impl Scalable for DummyScalable {
            fn scale(&mut self, factor: f64) -> Result<()> {
                self.value *= factor;
                Ok(())
            }
        }
        let mut dummy = DummyScalable { value: 10.0 };
        dummy.scale(2.5).unwrap();
        assert!((dummy.value - 25.0).abs() < 1e-8);
    }
}
