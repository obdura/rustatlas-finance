use crate::{
    cashflows::traits::InterestAccrual, instruments::bonds::traits::InteresAccrualAtYieldRate, time::date::Date, utils::errors::Result, visitors::traits::ConstVisit
};
use std::collections::BTreeMap;

/// ## AccruedAmountMapConstVisitor
/// This visitor calculates the accrued amount for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the accrued amount as the value.
///
/// Parameters
///    * `eval_date: Date` - The evaluation date.
///
/// Output
///   * `BTreeMap<Date, f64>` - A map with the date as the key and the accrued amount as the value.
///
pub struct AccruedAmountMapConstVisitor {
    eval_date: Date,
}

impl AccruedAmountMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        AccruedAmountMapConstVisitor {
            eval_date: eval_date,
        }
    }
}

impl<T: InterestAccrual> ConstVisit<&[T]> for AccruedAmountMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_accrued_amount(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_accrued_amount
/// Function to calculate the accrued amount for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the accrued amount as the value.
pub fn get_accrued_amount<T: InterestAccrual>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let accrued_amount = instruments
        .iter()
        .map(|inst| {
            let local_map = inst.accrued_amount_map();
            local_map
        })
        .collect::<Result<Vec<_>>>()? // Collect the results into a Vec<Result<BTreeMap<Date, f64>>>
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, x| {
            x.iter().for_each(|(k, v)| {
                let entry = acc.entry(*k).or_insert(0.0);
                *entry += *v;
            });
            acc
        })
        .into_iter()
        .filter(|(k, _)| *k >= eval_date)
        .collect();
    Ok(accrued_amount)
}

/// ## AccruedAmountMapAtYieldRateConstVisitor
/// This visitor calculates the accrued amount with YiledRate for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the accrued amount as the value.
/// 
/// Parameters
/// * `eval_date: Date` - The evaluation date.
/// 
/// Output
/// * `BTreeMap<Date, f64>` - A map with the date as the key and the accrued amount as the value.
pub struct AccruedAmountMapAtYieldRateConstVisitor {
    eval_date: Date,
}

impl AccruedAmountMapAtYieldRateConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        AccruedAmountMapAtYieldRateConstVisitor { eval_date: eval_date }
    }
}

impl<T: InteresAccrualAtYieldRate> ConstVisit<&[T]> for AccruedAmountMapAtYieldRateConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_accrued_amount_at_yield_rate(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_accrued_amount_at_yield_rate
/// Function to calculate the accrued amount with yiled rate for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the accrued amount as the value.
pub fn get_accrued_amount_at_yield_rate<T: InteresAccrualAtYieldRate>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let accrued_amount = instruments
        .iter()
        .map(|inst| {
            let local_map = inst.accrued_amount_map_at_yield_rate();
            local_map
        })
        .collect::<Result<Vec<_>>>()? // Collect the results into a Vec<Result<BTreeMap<Date, f64>>>
        .into_iter()
        .fold(BTreeMap::new(), |mut acc, x| {
            x.iter().for_each(|(k, v)| {
                let entry = acc.entry(*k).or_insert(0.0);
                *entry += *v;
            });
            acc
        })
        .into_iter()
        .filter(|(k, _)| *k >= eval_date)
        .collect();
    Ok(accrued_amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cashflows::side::Side,
        currencies::enums::Currency,
        instruments::{
            constructors::makefixedrateinstrument::MakeFixedRateInstrument,
            loandepos::instrument::Instrument,
        },
        rates::{enums::Compounding, interestrate::InterestRate},
        time::{daycounter::DayCounter, enums::{Frequency, TimeUnit}, period::Period},
        utils::errors::Result,
    };

    pub fn make_test_instrument(side: Side) -> Result<Vec<Instrument>> {
        let start_date_0 = Date::new(2020, 1, 1);
        let end_date_0 = start_date_0 + Period::new(5, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let mut instruments = Vec::new();

        for i in 0..5 {
            let start_date = start_date_0 + Period::new(i * 2, TimeUnit::Months);
            let end_date = end_date_0 + Period::new(i * 2, TimeUnit::Months);

            let tem = MakeFixedRateInstrument::new()
                .with_start_date(start_date)
                .with_end_date(end_date)
                .with_payment_frequency(Frequency::Monthly)
                .with_rate(rate)
                .with_notional(100.0)
                .with_side(side)
                .with_currency(Currency::CLP)
                .equal_redemptions()
                .build()?;
            instruments.push(Instrument::FixedRateInstrument(tem));
        }
        Ok(instruments)
    }

    #[test]
    fn test_get_accrued_amount() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 4, 4);
        let accrued_amount = get_accrued_amount(&instruments, eval_date)?;
        assert!(
            (accrued_amount.get(&Date::new(2023, 9, 30)).unwrap().clone() - 0.02267545523630214)
                .abs()
                < 0.0000000000001
        );
        Ok(())
    }

    #[test]
    fn test_get_accrued_amount_inverse() -> Result<()> {
        let instruments = make_test_instrument(Side::Pay)?;
        let eval_date = Date::new(2020, 4, 4);
        let accrued_amount = get_accrued_amount(&instruments, eval_date)?;
        assert!(
            (accrued_amount.get(&Date::new(2023, 9, 30)).unwrap().clone() + 0.02267545523630214)
                .abs()
                < 0.0000000000001
        );
        Ok(())
    }

    #[test]
    fn test_cost_visit() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 4, 4);
        let visitor = AccruedAmountMapConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instruments.as_slice())?;
        assert!(
            (accrued_amount.get(&Date::new(2023, 9, 30)).unwrap().clone() - 0.02267545523630214)
                .abs()
                < 0.0000000000001
        );
        Ok(())
    }
}
