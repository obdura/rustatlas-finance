use std::collections::BTreeMap;

use crate::{
    cashflows::{
        cashflow::Cashflow,
        traits::{InterestAccrual, Payable},
    },
    rates::interestrate::InterestRate,
    time::{date::Date, enums::TimeUnit, period::Period},
    utils::errors::{AtlasError, Result},
    visitors::traits::HasCashflows,
};

/// # InteresAccrualAtYieldRate trait
///
/// Implements fixed rate bond accrual using a yield rate.  
/// The yield rate is used to discount the cashflows to between the start and
/// end dates and calculate the accrued amount.
///
/// ## methods
///
/// - `yield_rate` - Returns the yield rate of the bond.
/// - `purchase_date` - Returns the purchase date of the bond.
/// - `accrual_amount_at_yield_rate` - Calculates the accrued amount of a bond between two dates.
/// - `total_payments_in_period` - Calculates the accrual of cash paid between two dates.
/// - `not_pay_interest` - Calculates the accrued interest for cashflows that have not yet been paid as of a given evaluation date.
/// - `discounted_cashflows_at_yield_rate` - Calculates the net present value (NPV) of the cashflows discounted to a given evaluation date.
///
pub trait InteresAccrualAtYieldRate: HasCashflows + InterestAccrual {
    /// Returns the yield rate of the bond.
    ///
    /// # Returns
    /// - `Option<InterestRate>`: The yield rate of the bond.
    fn yield_rate(&self) -> Option<InterestRate>;

    /// #Returns
    /// - `Date`: The purchase date of the bond.
    fn purchase_date(&self) -> Date;

    /// Calculates the accrued amount of a bond between two dates.
    ///
    /// This function calculates the accrued amount by discounting the cashflows at the start and end dates,
    /// and then adding the total payments made in the period between the start and end dates.
    ///
    /// # Parameters
    /// - `start_date`: The start date of the period.
    /// - `end_date`: The end date of the period.
    ///
    /// # Returns
    /// - `Result<f64>`: The accrued amount of the bond between the specified dates.
    ///
    /// # Errors
    /// - Returns an error if any of the discounted cashflows or total payments cannot be retrieved.
    fn accrual_amount_at_yield_rate(&self, start_date: Date, end_date: Date) -> Result<f64> {
        if self.purchase_date() > end_date {
            return Ok(0.0);
        }

        let ini_pv = if start_date < self.purchase_date() {
            self.discounted_cashflows_at_yield_rate(self.purchase_date())?
        } else {
            self.discounted_cashflows_at_yield_rate(start_date)?
        };

        let end_pv = self.discounted_cashflows_at_yield_rate(end_date)?;
        let contractual_cashflows = self.total_payments_in_period(start_date, end_date)?;

        Ok(end_pv - ini_pv + contractual_cashflows)
    }

    /// Calculates the accrued amount map at yield rate.
    ///
    /// This function calculates the accrued amount map by discounting the cashflows at the start and end dates,
    /// and then adding the total payments made in the period between the start and end dates.
    ///
    /// # Returns
    /// - `Result<BTreeMap<Date, f64>>` - The accrued amount map at yield rate.
    fn accrued_amount_map_at_yield_rate(&self) -> Result<BTreeMap<Date, f64>> {
        let delta = Period::new(1, TimeUnit::Days);
        let mut start_date = self.accrual_start_date().unwrap();
        let end_date = self.accrual_end_date().unwrap();

        let mut map = BTreeMap::new();
        map.insert(start_date, 0.0);
        while start_date < end_date {
            let accrued = self
                .accrual_amount_at_yield_rate(start_date, start_date + delta)
                .unwrap();
            let entry = map.entry(start_date + delta).or_insert(0.0);
            *entry += accrued;
            start_date = start_date + delta;
        }
        Ok(map)
    }

    /// Calculates the accrual of cash paid between two dates.
    ///
    /// This function iterates over the cashflows, filters them by the given date range,
    /// and sums the amounts of the relevant cashflows (FixedRateCoupon, FloatingRateCoupon, Redemption).
    ///
    /// # Parameters
    /// - `from`: The start date of the period.
    /// - `to`: The end date of the period.
    ///
    /// # Returns
    /// - `Result<f64>`: The total amount of cashflows paid in the specified period.
    ///
    /// # Errors
    /// - Returns an error if any of the cashflow amounts cannot be retrieved.
    fn total_payments_in_period(&self, from: Date, to: Date) -> Result<f64> {
        if from > to {
            return Err(AtlasError::EvaluationErr(
                "End date is before start date".to_string(),
            ));
        }

        let mut amount = 0.0;
        self.cashflows()
            .filter(|cf| cf.payment_date() >= from && cf.payment_date() < to)
            .try_for_each(|cf| -> Result<()> {
                if let Cashflow::FixedRateCoupon(_)
                | Cashflow::FloatingRateCoupon(_)
                | Cashflow::Redemption(_) = cf
                {
                    amount += cf.amount()? * cf.side().sign();
                }
                if let Cashflow::Disbursement(_) = cf {
                    if  cf.payment_date() > self.purchase_date(){ 
                        amount += cf.amount()? * cf.side().sign();   
                    }
                }
                Ok(())
            })?;


        Ok(amount)
    }


    /// Calculates the accrued interest Payment por coupon that finishes at a given evaluation date.
    fn interest_cupon_amount(&self, evaluation_date: Date) -> Result<f64> {
        let mut amount = 0.0;
        self.cashflows()
            .filter(|cf| cf.payment_date() == evaluation_date)
            .try_for_each(|cf| -> Result<()> {
                if let Cashflow::FloatingRateCoupon(_) | Cashflow::FixedRateCoupon(_) = cf {
                    let start_date = cf.accrual_start_date()?;
                    let end_date = cf.accrual_end_date()?;
                    if start_date <= evaluation_date && evaluation_date <= end_date {
                        amount += self.accrual_amount_at_yield_rate(start_date, end_date)?;
                    }
                }
                Ok(())
            })?;
        Ok(amount)
    }

    /// Calculates the accrued interest for cashflows that have not yet been paid as of a given evaluation date.
    ///
    /// This function iterates over the cashflows, filters them by the given evaluation date,
    /// and sums the accrued amounts of the relevant cashflows (FixedRateCoupon, FloatingRateCoupon).
    ///
    /// # Parameters
    /// - `evaluation_date`: The date up to which the interest is to be calculated.
    ///
    /// # Returns
    /// - `Result<f64>`: The total accrued interest for the cashflows that have not yet been paid as of the evaluation date.
    ///
    /// # Errors
    /// - Returns an error if any of the cashflow amounts or accrual dates cannot be retrieved.
    fn not_pay_interest(&self, evaluation_date: Date) -> Result<f64> {
        if self.purchase_date() > evaluation_date {
            return Ok(0.0);
        }

        let not_pay_accrual = self
            .cashflows()
            .filter(|cf| cf.payment_date() >= evaluation_date)
            .try_fold(0.0, |acc, cf| -> Result<f64> {
                match cf {
                    Cashflow::FixedRateCoupon(c) => {
                        let start_date = c.accrual_start_date()?;
                        let end_date = c.accrual_end_date()?;
                        if start_date <= evaluation_date && evaluation_date <= end_date {
                            Ok(acc
                                + self.accrual_amount_at_yield_rate(start_date, evaluation_date)?)
                        } else {
                            Ok(acc)
                        }
                    }
                    Cashflow::FloatingRateCoupon(c) => {
                        let start_date = c.accrual_start_date()?;
                        let end_date = c.accrual_end_date()?;
                        if start_date <= evaluation_date && evaluation_date <= end_date {
                            Ok(acc
                                + self.accrual_amount_at_yield_rate(start_date, evaluation_date)?)
                        } else {
                            Ok(acc)
                        }
                    }
                    _ => Ok(acc),
                }
            })?;

        Ok(not_pay_accrual)
    }

    /// Calculates the net present value (NPV) of the cashflows discounted to a given evaluation date.
    ///
    /// This function iterates over the cashflows, filters them by the given evaluation date,
    /// and calculates the NPV by discounting each cashflow amount using the yield rate and summing them up.
    ///
    /// # Parameters
    /// - `evaluation_date`: The date to which the cashflows are discounted.
    ///
    /// # Returns
    /// - `Result<f64>`: The net present value of the cashflows discounted to the evaluation date.
    ///
    /// # Errors
    /// - Returns an error if the yield rate is not found or if any of the cashflow amounts cannot be retrieved.
    fn discounted_cashflows_at_yield_rate(&self, evaluation_date: Date) -> Result<f64> {
        if evaluation_date < self.purchase_date() {
            return Ok(0.0);
        }
        
        // Get the yield rate of the bond.
        let rate = self
            .yield_rate()
            .ok_or(AtlasError::NotFoundErr("Yield rate".to_string()))?;



        let npv = self
            .cashflows()
            .filter(|cf| cf.payment_date() >= evaluation_date)
            .try_fold(0.0, |acc, cf| -> Result<f64> {
                match cf {
                    Cashflow::FixedRateCoupon(_) => {
                        let npv = cf.amount()?
                            * rate.discount_factor(evaluation_date, cf.payment_date())
                            * cf.side().sign();
                        Ok(acc + npv)
                    }
                    Cashflow::Redemption(_) => {
                        let npv = cf.amount()?
                            * rate.discount_factor(evaluation_date, cf.payment_date())
                            * cf.side().sign();
                        Ok(acc + npv)
                    }
                    Cashflow::FloatingRateCoupon(_) => {
                        let npv = cf.amount()?
                            * rate.discount_factor(evaluation_date, cf.payment_date())
                            * cf.side().sign();
                        Ok(acc + npv)
                    }
                    Cashflow::Disbursement(_) => {
                        if cf.payment_date() ==  self.purchase_date() {
                            Ok(acc)
                        }
                        else {
                            let npv = cf.amount()?
                                * rate.discount_factor(evaluation_date, cf.payment_date())
                                * cf.side().sign();
                            Ok(acc + npv)
                        }
                    }
                }
            })?;

        Ok(npv)
    }

    fn dirty_price(&self, evaluation_date: Date) -> Result<f64> {
        if evaluation_date < self.purchase_date() {
            return Ok(0.0);
        }

        if evaluation_date > self.accrual_end_date()? {
            return Ok(0.0);
        }

        self.discounted_cashflows_at_yield_rate(evaluation_date)
    }

    fn clean_price(&self, evaluation_date: Date) -> Result<f64> {
        if evaluation_date < self.purchase_date() {
            return Ok(0.0);
        }

        if evaluation_date > self.accrual_end_date()? {
            return Ok(0.0);
        }

        Ok(self.dirty_price(evaluation_date)? - self.not_pay_interest(evaluation_date)?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        cashflows::{side::Side, traits::InterestAccrual},
        currencies::enums::Currency,
        instruments::{bonds::traits::InteresAccrualAtYieldRate, constructors::makefixedratebond::MakeFixedRateBond},
        rates::{enums::Compounding, interestrate::InterestRate},
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
    };

    #[test]
    fn test_complex_yield_rate_case_discount_value() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from(
            [(Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0) ]);

        let redemptions = HashMap::from(
            [(Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0)]);
    
        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2019, 12, 30))? - 0.0).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2019, 12, 31))? - 0.0).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2020,  1,  1))? - 1484481.36472548).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2020,  4,  1))? - 1495461.5925049107).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2020,  7,  1))? - 2002686.6003660692).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2021,  1,  1))? - 1515123.2876712328).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2021,  1,  2))? - 0.0).abs() < 1e-6);
        assert!((instrument.discounted_cashflows_at_yield_rate(Date::new(2021,  1,  3))? - 0.0).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_complex_yield_rate_case_accrual_value() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from(
            [(Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0) ]);

        let redemptions = HashMap::from(
            [(Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0)]);
    
        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2019, 12, 30), Date::new(2019, 12, 31))? - 0.0).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2019, 12, 31), Date::new(2020, 1, 1))? - 0.0).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 1, 1), Date::new(2020, 1, 2))? - 120.2226517).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 1, 2), Date::new(2020, 1, 3))? - 120.2323881).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 3, 30), Date::new(2020, 3, 31))? - 121.0922853).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 3, 31), Date::new(2020, 4, 1))? - 121.1020921).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 4, 1), Date::new(2020, 4, 2))? - 160.9993161).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 6, 30), Date::new(2020, 7, 1))? - 162.177041808376).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 12, 31), Date::new(2021, 1, 1))? - 122.6942911).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2019, 1, 1), Date::new(2020, 3, 31))? - 10859.12568728917).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 1, 1), Date::new(2020, 3, 31))? - 10859.12568728917).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2019, 1, 1), Date::new(2020, 4, 1))? - 10980.2277794322).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 1, 1), Date::new(2020, 4, 1))? - 10980.2277794322).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2019, 1, 1), Date::new(2020, 4, 2))? - 11141.2270955708).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 1, 1), Date::new(2020, 4, 2))? - 11141.2270955708).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2020, 1, 1), Date::new(2021, 1, 1))? - 48093.9777402745).abs() < 1e-6);
        assert!((instrument.accrual_amount_at_yield_rate(Date::new(2019, 1, 1), Date::new(2022, 1, 1))? - 48093.9777402745).abs() < 1e-6);


        assert!((instrument.not_pay_interest(Date::new(2020, 1, 1))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 1, 2))? - 120.222651742399).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2019, 12, 30))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2019, 12, 31))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 1, 1))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 1, 2))? - 120.222651742399).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 4, 1))? - 10980.2277794322).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 4, 2))? - 160.999316138565).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2021, 1, 2))? - 0.0).abs() < 1e-6);

        assert!((instrument.dirty_price(Date::new(2019, 12, 30))? - 0.0).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2020, 1, 1))? - 1484481.36472548).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2020, 4, 1))? - 1495461.59250491).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2021, 1, 1))? - 1515123.28767123).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2021, 1, 2))? - 0.0).abs() < 1e-6);

        Ok(())
    }


    #[test]
    fn test_complex_yield_rate_not_pay_interest_value() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from(
            [(Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0) ]);

        let redemptions = HashMap::from(
            [(Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0)]);
    
        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        assert!((instrument.not_pay_interest(Date::new(2020, 1, 1))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 1, 2))? - 120.222651742399).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2019, 12, 30))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2019, 12, 31))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 1, 1))? - 0.0).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 1, 2))? - 120.222651742399).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 4, 1))? - 10980.2277794322).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2020, 4, 2))? - 160.999316138565).abs() < 1e-6);
        assert!((instrument.not_pay_interest(Date::new(2021, 1, 2))? - 0.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_complex_yield_rate_dirty_price_value() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from(
            [(Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0) ]);

        let redemptions = HashMap::from(
            [(Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0)]);
    
        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        assert!((instrument.dirty_price(Date::new(2019, 12, 30))? - 0.0).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2020, 1, 1))? - 1484481.36472548).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2020, 4, 1))? - 1495461.59250491).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2021, 1, 1))? - 1515123.28767123).abs() < 1e-6);
        assert!((instrument.dirty_price(Date::new(2021, 1, 2))? - 0.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_complex_yield_rate_clean_price_value() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from(
            [(Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0) ]);

        let redemptions = HashMap::from(
            [(Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0)]);
    
        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        assert!((instrument.clean_price(Date::new(2019, 12, 30))? - 0.0).abs() < 1e-6);
        assert!((instrument.clean_price(Date::new(2020, 1, 1))? - 1484481.36472548).abs() < 1e-6);
        assert!((instrument.clean_price(Date::new(2020, 4, 1))? - 1484481.36472548).abs() < 1e-6);
        assert!((instrument.clean_price(Date::new(2021, 1, 1))? - 1492713.99762634 ).abs() < 1e-6);
        assert!((instrument.clean_price(Date::new(2021, 1, 2))? - 0.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_instrument_at_par_value() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from(
            [(Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0) ]);

        let redemptions = HashMap::from(
            [(Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0)]);
    
        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        let accrual_map = instrument.accrued_amount_map()?;
        let accrual_map_at_yield_rate = instrument.accrued_amount_map_at_yield_rate()?;

        for (date, value) in accrual_map.iter() {
            assert!((accrual_map_at_yield_rate.get(date).unwrap() - value).abs() < 1e-6);
        }

        for (date, value) in accrual_map_at_yield_rate.iter() {
            assert!((accrual_map.get(date).unwrap() - value).abs() < 1e-6);
        }

        Ok(())
    }



}
