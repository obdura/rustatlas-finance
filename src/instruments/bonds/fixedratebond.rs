use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    cashflows::{
        cashflow::Cashflow,
        side::Side,
        traits::{InterestAccrual, Payable},
    },
    core::traits::HasCurrency,
    currencies::enums::Currency,
    instruments::traits::Structure,
    rates::interestrate::InterestRate,
    time::{
        date::Date,
        enums::Frequency,
    },
    utils::errors::Result,
    visitors::traits::HasCashflows,
};

use super::traits::InteresAccrualAtYieldRate;

/// # FixedRateBond
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
pub struct FixedRateBond {
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
    mnemonic: Option<String>,
    issue_date: Option<Date>,
    yield_rate: Option<InterestRate>,
    purchase_date: Option<Date>,
}

impl FixedRateBond {
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
        mnemonic: Option<String>,
        issue_date: Option<Date>,
        yield_rate: Option<InterestRate>,
        purchase_date: Option<Date>,
    ) -> Self {
        FixedRateBond {
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
            mnemonic,
            issue_date,
            yield_rate,
            purchase_date,
        }
    }

    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn mnemonic(&self) -> Option<String> {
        self.mnemonic.clone()
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

    pub fn set_rate(mut self, rate: InterestRate) -> Self {
        self.rate = rate;
        self.mut_cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(coupon) => {
                coupon.set_rate(rate);
            }
            _ => {}
        });
        self
    }
}

impl HasCurrency for FixedRateBond {
    fn currency(&self) -> Result<Currency> {
        Ok(self.currency)
    }
}

impl InterestAccrual for FixedRateBond {
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

impl HasCashflows for FixedRateBond {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.cashflows.iter())
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(self.cashflows.iter_mut())
    }
}

impl InteresAccrualAtYieldRate for FixedRateBond {
    fn yield_rate(&self) -> Option<InterestRate> {
        self.yield_rate
    }
    fn purchase_date(&self) -> Date {
        match self.purchase_date {
            Some(date) => date,
            None => self.start_date,
        }
    }
}

/// # Display
/// Implement the display for FixedRateBond.
impl Display for FixedRateBond {
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
        writeln!(f, "\tyield rate: {:?}", self.yield_rate())?;
        writeln!(f, "\tpurchase date: {:?}", self.purchase_date())?;
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
    use crate::{
        cashflows::{cashflow::Cashflow, side::Side, traits::Payable}, core::traits::HasDiscountCurveId, currencies::enums::Currency, instruments::{
            bonds::traits::InteresAccrualAtYieldRate,
            constructors::makefixedratebond::MakeFixedRateBond,
        }, rates::{enums::Compounding, interestrate::InterestRate}, time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        }, utils::errors::Result, visitors::traits::HasCashflows
    };

    #[test]
    fn bond_accrual_bullet_instrument() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let yield_rate = InterestRate::new(
            0.07,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_yield_rate(yield_rate)
            .bullet()
            .build()?;

        let date = start_date + Period::new(1, TimeUnit::Months);
        let mut accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(1, TimeUnit::Months))?;
        assert!((accrual_aux - 27_385.1934467).abs() < 1e-6);

        let date = start_date + Period::new(2, TimeUnit::Months);
        accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(1, TimeUnit::Months))?;
        assert!((accrual_aux - 27_540.0333112).abs() < 1e-6);

        let date = start_date + Period::new(3, TimeUnit::Months);
        accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(6, TimeUnit::Months))?;
        assert!((accrual_aux - 165_982.433650).abs() < 1e-6);

        let date = start_date + Period::new(54, TimeUnit::Months);
        accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(6, TimeUnit::Months))?;
        assert!((accrual_aux - 171_307.0814148).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn bond_accrual_bullet_instrument_inverse_side() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let yield_rate = InterestRate::new(
            0.07,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive.inverse())
            .with_currency(Currency::USD)
            .with_yield_rate(yield_rate)
            .bullet()
            .build()?;

        let date = start_date + Period::new(1, TimeUnit::Months);
        let mut accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(1, TimeUnit::Months))?;
        println!("{}", accrual_aux);
        assert!((accrual_aux + 27_385.1934467).abs() < 1e-6);

        let date = start_date + Period::new(2, TimeUnit::Months);
        accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(1, TimeUnit::Months))?;
        assert!((accrual_aux + 27_540.0333112).abs() < 1e-6);

        let date = start_date + Period::new(3, TimeUnit::Months);
        accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(6, TimeUnit::Months))?;
        println!("{}", accrual_aux);
        assert!((accrual_aux + 165_982.433650).abs() < 1e-6);

        let date = start_date + Period::new(54, TimeUnit::Months);
        accrual_aux = instrument
            .accrual_amount_at_yield_rate(date, date + Period::new(6, TimeUnit::Months))?;
        assert!((accrual_aux + 171_307.0814148).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn bond_accrual_bullet_instrument_shift_dates() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let yield_rate = InterestRate::new(
            0.07,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_yield_rate(yield_rate)
            .bullet()
            .build()?;

        // test accrual amount with shifted dates at the start
        let accrual_aux = instrument.accrual_amount_at_yield_rate(
            start_date,
            start_date + Period::new(1, TimeUnit::Months),
        )?;
        let accrual_aux_shift = instrument.accrual_amount_at_yield_rate(
            start_date - Period::new(1, TimeUnit::Months),
            start_date + Period::new(1, TimeUnit::Months),
        )?;

        assert!((accrual_aux - accrual_aux_shift).abs() < 1e-6);

        // test accrual amount with shifted dates at the end
        let accrual_aux = instrument
            .accrual_amount_at_yield_rate(end_date - Period::new(1, TimeUnit::Months), end_date)?;
        let accrual_aux_shift = instrument.accrual_amount_at_yield_rate(
            end_date - Period::new(1, TimeUnit::Months),
            end_date + Period::new(1, TimeUnit::Months),
        )?;

        assert!((accrual_aux - accrual_aux_shift).abs() < 1e-6);

        // test accrual amount with shifted dates at the start and end
        let accrual_aux = instrument.accrual_amount_at_yield_rate(start_date, end_date)?;
        let accrual_aux_shift = instrument.accrual_amount_at_yield_rate(
            start_date - Period::new(1, TimeUnit::Months),
            end_date + Period::new(1, TimeUnit::Months),
        )?;

        assert!((accrual_aux - accrual_aux_shift).abs() < 1e-6);

        // test accrual amount with shifted star date and end date les than purchase date
        let accrual_aux = instrument.accrual_amount_at_yield_rate(
            start_date - Period::new(2, TimeUnit::Years),
            start_date - Period::new(1, TimeUnit::Months),
        )?;

        assert!((accrual_aux - 0.0).abs() < 1e-6);

        // test accrual amount with shifted star date and end date greater than purchase date
        let accrual_aux = instrument.accrual_amount_at_yield_rate(
            end_date + Period::new(1, TimeUnit::Months),
            end_date + Period::new(2, TimeUnit::Months),
        )?;

        assert!((accrual_aux - 0.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn bond_discounted_cashflows_bullet_instrument() -> Result<()> {
        let start_date = Date::new(2015, 3, 1);
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate = InterestRate::new(
            4.5 / 100.0,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let tir = InterestRate::new(
            2.731 / 100.0,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );
        let notional = 5e7;

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(notional)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_yield_rate(tir)
            .bullet()
            .build()?;

        let npv = instrument.discounted_cashflows_at_yield_rate(start_date)?;
        assert!((npv / notional - 1.0344883241422198).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn bond_discounted_cashflows_bullet_instrument_inverse_side() -> Result<()> {
        let start_date = Date::new(2015, 3, 1);
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate = InterestRate::new(
            4.5 / 100.0,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let tir = InterestRate::new(
            2.731 / 100.0,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );
        let notional = 5e7;

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(notional)
            .with_side(Side::Receive.inverse())
            .with_currency(Currency::CLP)
            .with_yield_rate(tir)
            .bullet()
            .build()?;

        let npv = instrument.discounted_cashflows_at_yield_rate(start_date)?;
        assert!((npv / notional + 1.0344883241422198).abs() < 1e-6);
        Ok(())
    }

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

        let instrument = MakeFixedRateBond::new()
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

        let new_instrument = instrument.set_rate(new_rate);

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
    fn test_not_pay_interes_for_bond_accrual() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let yield_rate = InterestRate::new(
            0.07,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_yield_rate(yield_rate)
            .bullet()
            .build()?;

        let test_date = Date::new(2024, 1, 1);
        println!("{}", instrument.not_pay_interest(test_date)?);
        assert!((instrument.not_pay_interest(test_date)? - 0.0).abs() < 1e-6);

        let test_date = Date::new(2024, 1, 2);
        println!("{}", instrument.not_pay_interest(test_date)?);
        assert!((instrument.not_pay_interest(test_date)? - 931.9152881586924).abs() < 1e-6);

        let test_date = Date::new(2024, 7, 2);
        println!("{}", instrument.not_pay_interest(test_date)?);
        assert!((instrument.not_pay_interest(test_date)? - 935.786916904297).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_not_pay_interes_for_bond_accrual_inverse_side() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.06,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let yield_rate = InterestRate::new(
            0.07,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive.inverse())
            .with_currency(Currency::USD)
            .with_yield_rate(yield_rate)
            .bullet()
            .build()?;

        let test_date = Date::new(2024, 1, 1);
        println!("{}", instrument.not_pay_interest(test_date)?);
        assert!((instrument.not_pay_interest(test_date)? + 0.0).abs() < 1e-6);

        let test_date = Date::new(2024, 1, 2);
        println!("{}", instrument.not_pay_interest(test_date)?);
        assert!((instrument.not_pay_interest(test_date)? + 931.9152881586924).abs() < 1e-6);

        let test_date = Date::new(2024, 7, 2);
        println!("{}", instrument.not_pay_interest(test_date)?);
        assert!((instrument.not_pay_interest(test_date)? + 935.786916904297).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_set_discount_curve_id() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(3, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Annual)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Pay)
            .with_currency(Currency::EUR)
            .bullet()
            .build()?;

        let instrument = instrument.set_discount_curve_id(42);

        assert_eq!(instrument.discount_curve_id(), Some(42));
        instrument.cashflows().for_each(|cf| {
            assert_eq!(cf.discount_curve_id().unwrap(), 42);
        });

        Ok(())
    }

    #[test]
    fn test_display_trait() -> Result<()> {
        let start_date = Date::new(2023, 6, 15);
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate = InterestRate::new(
            0.04,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Annual)
            .with_rate(rate)
            .with_notional(2_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        let display = format!("{}", instrument);
        assert!(display.contains("Instrument: id:"));
        assert!(display.contains("notional: 2000000"));
        assert!(display.contains("currency: Ok(USD)"));
        assert!(display.contains("start_date:"));
        assert!(display.contains("end_date:"));
        assert!(display.contains("cashflows:"));

        Ok(())
    }

    #[test]
    fn test_bond_with_issue_and_purchase_date() -> Result<()> {
        let start_date = Date::new(2022, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.03,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );
        let issue_date = Some(Date::new(2021, 12, 15));
        let purchase_date = Some(Date::new(2022, 1, 10));

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Annual)
            .with_rate(rate)
            .with_notional(100_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_issue_date(issue_date.unwrap())
            .with_purchase_date(purchase_date.unwrap())
            .bullet()
            .build()?;

        assert_eq!(instrument.issue_date(), issue_date);
        assert_eq!(instrument.purchase_date(), purchase_date.unwrap());

        Ok(())
    }

    #[test]
    fn test_bond_id_and_mnemonic() -> Result<()> {
        let start_date = Date::new(2023, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.025,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );
        let id = Some("BOND123".to_string());
        let mnemonic = Some("MYBOND".to_string());

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Annual)
            .with_rate(rate)
            .with_notional(50_000.0)
            .with_side(Side::Pay)
            .with_currency(Currency::EUR)
            .with_id(id.clone())
            .with_mnemonic(mnemonic.clone())
            .bullet()
            .build()?;

        assert_eq!(instrument.id(), id);
        assert_eq!(instrument.mnemonic(), mnemonic);

        Ok(())
    }

    #[test]
    fn test_bond_zero_notional() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Annual)
            .with_rate(rate)
            .with_yield_rate(rate)
            .with_notional(0.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        let accrual = instrument.accrual_amount_at_yield_rate(start_date, end_date)?;
        assert!((accrual - 0.0).abs() < 1e-6);

        Ok(())
    }

}
