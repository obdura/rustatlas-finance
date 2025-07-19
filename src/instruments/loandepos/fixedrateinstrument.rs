use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
};

use serde::{Deserialize, Serialize};

use crate::{
    cashflows::{
        cashflow::Cashflow,
        side::Side,
        traits::{InterestAccrual, Payable, Scalable},
    },
    core::traits::{HasCurrency, Registrable},
    currencies::enums::Currency,
    instruments::{
        constructors::makefixedrateinstrument::MakeFixedRateInstrument, traits::Structure
    },
    rates::interestrate::InterestRate,
    time::{date::Date, enums::Frequency},
    utils::errors::{AtlasError, Result},
    visitors::traits::HasCashflows,
};

/// # FixedRateInstrument
/// A fixed rate instrument.
///
/// ## Parameters
/// * `start_date` - The start date.
/// * `end_date` - The end date.
/// * `notional` - The notional.
/// * `rate` - The rate.
/// * `cashflows` - The cashflows.
/// * `structure` - The structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FixedRateInstrument {
    start_date: Date,
    end_date: Date,
    notional: f64,
    rate: InterestRate,
    payment_frequency: Frequency,
    cashflows: Vec<Cashflow>,
    structure: Structure,
    side: Side,
    currency: Currency,
    discount_curve_id: Option<usize>,
    id: Option<String>,
    issue_date: Option<Date>,
}

impl FixedRateInstrument {
    pub fn new(
        start_date: Date,
        end_date: Date,
        notional: f64,
        rate: InterestRate,
        payment_frequency: Frequency,
        cashflows: Vec<Cashflow>,
        structure: Structure,
        side: Side,
        currency: Currency,
        discount_curve_id: Option<usize>,
        id: Option<String>,
        issue_date: Option<Date>,
    ) -> Self {
        FixedRateInstrument {
            start_date,
            end_date,
            notional,
            rate,
            payment_frequency,
            cashflows,
            structure,
            side,
            currency,
            discount_curve_id,
            id,
            issue_date,
        }
    }

    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn start_date(&self) -> Date {
        self.start_date
    }

    pub fn end_date(&self) -> Date {
        self.end_date
    }

    pub fn notional(&self) -> f64 {
        self.notional
    }

    pub fn rate(&self) -> InterestRate {
        self.rate
    }

    pub fn structure(&self) -> Structure {
        self.structure
    }

    pub fn payment_frequency(&self) -> Frequency {
        self.payment_frequency
    }

    pub fn discount_curve_id(&self) -> Option<usize> {
        self.discount_curve_id
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn issue_date(&self) -> Option<Date> {
        self.issue_date
    }

    pub fn set_discount_curve_id(mut self, discount_curve_id: usize) -> Self {
        self.discount_curve_id = Some(discount_curve_id);
        self.mut_cashflows()
            .for_each(|cf| cf.set_discount_curve_id(discount_curve_id));

        self
    }

    pub fn set_rate(mut self, rate: InterestRate) -> Result<Self> {
        self.rate = rate;
        let structure = self.structure();
        match structure {
            Structure::EqualPayments => {
                let builder = MakeFixedRateInstrument::from(&self).with_rate(rate);
                let mut tmp_inst = builder.build()?;

                let mut old_cashflows = Vec::new();
                std::mem::swap(&mut self.cashflows, &mut old_cashflows);

                let mut new_cashflows = Vec::new();
                std::mem::swap(&mut tmp_inst.cashflows, &mut new_cashflows);

                re_indexing_cashflows_for_equal_payment_instrument(&mut new_cashflows, &old_cashflows)?;

                std::mem::swap(&mut self.cashflows, &mut new_cashflows);
            }
            _ => {
                self.mut_cashflows().for_each(|cf| match cf {
                    Cashflow::FixedRateCoupon(coupon) => {
                        coupon.set_rate(rate);
                    }
                    _ => {}
                });
            }
        }

        Ok(self)
    }
}

impl HasCurrency for FixedRateInstrument {
    fn currency(&self) -> Result<Currency> {
        Ok(self.currency)
    }
}

impl InterestAccrual for FixedRateInstrument {
    fn accrual_start_date(&self) -> Result<Date> {
        Ok(self.start_date)
    }
    fn accrual_end_date(&self) -> Result<Date> {
        Ok(self.end_date)
    }
    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let total_accrued_amount = self.cashflows().fold(0.0, |acc, cf| {
            acc + cf.accrued_amount(start_date, end_date).unwrap_or(0.0)
        });
        Ok(total_accrued_amount)
    }

    fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>> {
        let map = self
            .cashflows()
            .try_fold(BTreeMap::new(), |mut acc, cf| -> Result<_> {
                let cf_map = cf.accrued_amount_map()?;
                for (date, amount) in cf_map {
                    let entry = acc.entry(date).or_insert(0.0);
                    *entry += amount;
                }
                Ok(acc)
            })?;
        Ok(map)
    }
}

impl HasCashflows for FixedRateInstrument {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.cashflows.iter())
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(self.cashflows.iter_mut())
    }
}

impl Scalable for FixedRateInstrument {
    fn scale(&mut self, factor: f64) -> Result<()> {
        self.mut_cashflows()
            .try_for_each(|cashflow| cashflow.scale(factor))?;
        self.notional = self.notional() * factor;
        Ok(())
    }
}

fn re_indexing_cashflows_for_equal_payment_instrument(
    new_cashflows: &mut Vec<Cashflow>,
    old_cashflows: &Vec<Cashflow>,
) -> Result<()> {
    let (redemptions_map, disbursements_map, coupon_map) = old_cashflows.iter().fold(
        (HashMap::new(), HashMap::new(), HashMap::new()),
        |(mut redemptions, mut disbursements, mut coupons), cashflow| {
            match cashflow {
                Cashflow::Redemption(c) => {
                    if let Ok(id) = c.id() {
                        redemptions.insert(c.payment_date(), id);
                    }
                }
                Cashflow::Disbursement(c) => {
                    if let Ok(id) = c.id() {
                        disbursements.insert(c.payment_date(), id);
                    }
                }
                Cashflow::FixedRateCoupon(c) => {
                    if let Ok(id) = c.id() {
                        coupons.insert(c.payment_date(), id);
                    }
                }
                _ => {}
            }
            (redemptions, disbursements, coupons)
        },
    );

    let _ = new_cashflows.iter_mut().try_for_each(|cf| -> Result<()> {
        match cf {
            Cashflow::FixedRateCoupon(_) => {
                if let Some(id) = coupon_map.get(&cf.payment_date()) {
                    cf.set_id(*id);
                }
                Ok(())
            }
            Cashflow::Redemption(_) => {
                if let Some(id) = redemptions_map.get(&cf.payment_date()) {
                    cf.set_id(*id);
                } else if let Some(id) = disbursements_map.get(&cf.payment_date()) {
                    cf.set_id(*id);
                } 
                Ok(())
            }
            Cashflow::Disbursement(_) => {
                if let Some(id) = disbursements_map.get(&cf.payment_date()) {
                    cf.set_id(*id);
                } else if let Some(id) = redemptions_map.get(&cf.payment_date()) {
                    cf.set_id(*id);
                }
                Ok(())
            }
            _ => {
                Err(AtlasError::IndexingErr("not supported cashflow type for equal payment instrument".to_string()))
            }
        }
    });
    Ok(())
}

/// # Display
/// Implement the display for FixedRateInstrument.
impl Display for FixedRateInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Instrument: id: {:?}", self.id())?;
        writeln!(f, "\tnotional: {}", self.notional())?;
        writeln!(f, "\tcurrency: {:?}", self.currency())?;
        writeln!(f, "\tstart_date: {:?}", self.start_date())?;
        writeln!(f, "\tend_date: {:?}", self.end_date())?;
        writeln!(f, "\tstructure: {:?}", self.structure())?;
        writeln!(f, "\tpayment_frequency: {:?}", self.payment_frequency())?;
        writeln!(f, "\tside: {:?}", self.side())?;
        writeln!(f, "\trate definition: {:?}", self.rate().rate_definition())?;
        writeln!(f, "\trate value: {:?}", self.rate().rate())?;
        writeln!(f, "\tcashflows: ")?;
        // Group cashflows by payment date and print them in order
        let btree_map =
            self.cashflows
                .iter()
                .fold(std::collections::BTreeMap::new(), |mut acc, cf| {
                    let date = cf.payment_date();
                    acc.entry(date).or_insert(vec![]).push(cf);
                    acc
                });
        for (_, cashflows) in btree_map.iter() {
            for cf in cashflows {
                writeln!(f, "\t\t{}", cf)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        cashflows::{
            cashflow::Cashflow,
            side::Side,
            traits::{Payable, Scalable},
        }, core::traits::Registrable, currencies::enums::Currency, instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument, rates::{enums::Compounding, interestrate::InterestRate}, time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        }, utils::errors::Result, visitors::traits::HasCashflows
    };

    #[test]
    fn test_set_rate() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(coupon) => {
                assert!((coupon.amount().unwrap() - 150000.0).abs() < 1e-6);
                assert_eq!(coupon.rate(), rate);
            }
            _ => {}
        });

        let new_rate = InterestRate::new(
            0.03,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let new_instrument = instrument.set_rate(new_rate)?;

        new_instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(coupon) => {
                assert!((coupon.amount().unwrap() - 75000.0).abs() < 1e-6);
                assert_eq!(coupon.rate(), new_rate);
            }
            _ => {}
        });

        Ok(())
    }

    #[test]
    fn test_set_rate_equalpayment() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .equal_payments()
            .build()?;

        let new_rate = InterestRate::new(
            0.03,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let new_instrument_rc = instrument.set_rate(new_rate)?;
        let new_instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(new_rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .equal_payments()
            .build()?;

        let mut map_cupons = HashMap::new();
        new_instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(_) | Cashflow::Redemption(_) => {
                let date = cf.payment_date();
                let amount = cf.amount().unwrap();
                map_cupons
                    .entry(date)
                    .and_modify(|e| *e += amount)
                    .or_insert(amount);
            }
            _ => {}
        });

        let mut map_cupons_rc = HashMap::new();
        new_instrument_rc.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(_) | Cashflow::Redemption(_) => {
                let date = cf.payment_date();
                let amount = cf.amount().unwrap();
                map_cupons_rc
                    .entry(date)
                    .and_modify(|e| *e += amount)
                    .or_insert(amount);
            }
            _ => {}
        });

        for (date, amount) in map_cupons.iter() {
            assert!((amount - map_cupons_rc.get(date).unwrap()).abs() < 1e-6);
        }

        for (date, amount) in map_cupons_rc.iter() {
            assert!((amount - map_cupons.get(date).unwrap()).abs() < 1e-6);
        }

        Ok(())
    }

    #[test]
    fn test_set_rate_equalpayment_with_delay() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let first_coupon_date = Some(Date::new(2024, 6, 1));
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_first_coupon_date(first_coupon_date)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .equal_payments()
            .build()?;

        let new_rate = InterestRate::new(
            0.03,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let new_instrument_rc = instrument.set_rate(new_rate)?;
        let new_instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(new_rate)
            .with_first_coupon_date(first_coupon_date)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .equal_payments()
            .build()?;

        let mut map_cupons = HashMap::new();
        new_instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(_) | Cashflow::Redemption(_) => {
                let date = cf.payment_date();
                let amount = cf.amount().unwrap();
                map_cupons
                    .entry(date)
                    .and_modify(|e| *e += amount)
                    .or_insert(amount);
            }
            _ => {}
        });

        let mut map_cupons_rc = HashMap::new();
        new_instrument_rc.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(_) | Cashflow::Redemption(_) => {
                let date = cf.payment_date();
                let amount = cf.amount().unwrap();
                map_cupons_rc
                    .entry(date)
                    .and_modify(|e| *e += amount)
                    .or_insert(amount);
            }
            _ => {}
        });

        for (date, amount) in map_cupons.iter() {
            assert!((amount - map_cupons_rc.get(date).unwrap()).abs() < 1e-6);
        }

        for (date, amount) in map_cupons_rc.iter() {
            assert!((amount - map_cupons.get(date).unwrap()).abs() < 1e-6);
        }

        Ok(())
    }

    #[test]
    fn test_scale() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let mut instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        instrument.scale(2.0)?;

        assert!((instrument.notional() - 2.0 * 5_000_000.0).abs() < 1e-6);
        instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(c) => {
                assert!((c.amount().unwrap() - 2.0 * 5_000_000.0 * 0.06 * 0.5) < 1e-6)
            }
            Cashflow::Disbursement(c) => {
                assert!((c.amount().unwrap() - 2.0 * 5_000_000.0) < 1e-6)
            }
            Cashflow::Redemption(c) => {
                assert!((c.amount().unwrap() - 2.0 * 5_000_000.0) < 1e-6)
            }
            _ => {}
        });
        Ok(())
    }

    #[test]
    fn test_reindexing() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let notional = 100.0;
        let mut instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(notional)
            .with_discount_curve_id(Some(0))
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .equal_payments()
            .build()?;

        instrument.mut_cashflows().for_each(|cf| cf.set_id(0));
        instrument.cashflows().for_each(|cf| assert!(cf.id().unwrap() == 0));

        let new_instrument = instrument.set_rate(InterestRate::new(
            0.06,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        ))?;

        new_instrument.cashflows().for_each(|cf| assert!(cf.id().unwrap() == 0));
        Ok(())
    }
}
