use crate::{
    cashflows::{cashflow::Cashflow, traits::Payable}, instruments::bonds::traits::InteresAccrualAtYieldRate, time::{date::Date, enums::TimeUnit, period::Period}, utils::errors::Result, visitors::traits::{ConstVisit, HasCashflows}
};
use std::collections::BTreeMap;


/// ## PlacementMapConstVisitor
/// This visitor calculates the placement for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the placement as the value.
///
/// Parameters
///   * `eval_date: Date` - The evaluation date.
/// 
/// Output
///  * `BTreeMap<Date, f64>` - A map with the date as the key and the placement as the value.
pub struct PlacementMapConstVisitor {
    eval_date: Date,
}

impl PlacementMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        PlacementMapConstVisitor { eval_date: eval_date }
    }
}

impl<T: HasCashflows> ConstVisit<&[T]> for PlacementMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_placement_map(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_placement_map
/// Function to calculate the placement for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the placement as the value.
pub fn get_placement_map<T: HasCashflows>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let placement = instruments
        .iter()
        .map(|inst| {
            let mut local_map = BTreeMap::new();
            inst.cashflows().for_each(|cf| match cf {
                Cashflow::Disbursement(f) => {
                    let payment_date = f.payment_date();
                    if payment_date >= eval_date {
                        let amount = f.amount().unwrap();
                        let entry = local_map.entry(payment_date).or_insert(0.0);
                        *entry += amount * f.side().inverse().sign();
                    }
                }
                _ => {}
            });
            local_map
        })
        .fold(BTreeMap::new(), |mut acc, x| {
            for (k, v) in x {
                let entry = acc.entry(k).or_insert(0.0);
                *entry += v;
            }
            acc
        });

    Ok(placement)
}


/// ## PlacementMapAtYieldRateConstVisitor
/// This visitor calculates the placement for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the placement as the value.
/// 
/// Parameters
///   * `eval_date: Date` - The evaluation date.
/// 
/// Output
///  * `BTreeMap<Date, f64>` - A map with the date as the key and the placement as the value.
pub struct PlacementMapAtYieldRateConstVisitor {
    eval_date: Date,
}

impl PlacementMapAtYieldRateConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        PlacementMapAtYieldRateConstVisitor { eval_date: eval_date }
    }
}

impl<T: InteresAccrualAtYieldRate> ConstVisit<&[T]> for PlacementMapAtYieldRateConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let mut map = get_placement_map_at_yield_rate(inst)?;

        map.retain(|k, _| *k >= self.eval_date);
        Ok(map)
    }
}

/// ## get_placement_at_yield_rate
/// Function to calculate the placement for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the placement as the value.
pub fn get_placement_map_at_yield_rate<T: InteresAccrualAtYieldRate>(
    instruments: &[T]
) -> Result<BTreeMap<Date, f64>> {
    let mut placement = BTreeMap::new();
    let _ = instruments.iter().try_for_each(|inst| -> Result<()> {
        let purchase_date = inst.purchase_date();
        let clean_price = inst.clean_price(purchase_date)?;
        let entry: &mut f64 = placement.entry(purchase_date).or_insert(0.0);
        *entry += clean_price;
        
        let mut start_date = inst.purchase_date() + Period::new(1, TimeUnit::Days);
        let mut actual_clean_price = inst.clean_price(start_date)?;
        let end_date = inst.accrual_end_date()?;
        while start_date <= end_date {
            let next_clean_price = inst.clean_price(start_date + Period::new(1, TimeUnit::Days))?;
            if next_clean_price.abs() > actual_clean_price.abs() {
                let entry: &mut f64 = placement.entry(start_date).or_insert(0.0);
                *entry += next_clean_price - actual_clean_price;
            }
            actual_clean_price = next_clean_price;
            start_date = start_date + Period::new(1, TimeUnit::Days);
        }
        Ok(())
    });
    
    Ok(placement)
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
        time::{
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
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
    fn test_get_placement() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 4, 5);

        let placement = get_placement_map(&instruments, eval_date)?;

        assert!(placement.get(&Date::new(2020, 5, 1)).unwrap().clone() == 100.0);
        assert!(placement.get(&Date::new(2020, 7, 1)).unwrap().clone() == 100.0);
        assert!(placement.get(&Date::new(2020, 9, 1)).unwrap().clone() == 100.0);
        assert!(placement.len() == 3);

        let instruments = make_test_instrument(Side::Pay)?;
        let placement = get_placement_map(&instruments, eval_date)?;

        assert!(placement.get(&Date::new(2020, 5, 1)).unwrap().clone() == -100.0);
        assert!(placement.get(&Date::new(2020, 7, 1)).unwrap().clone() == -100.0);
        assert!(placement.get(&Date::new(2020, 9, 1)).unwrap().clone() == -100.0);
        assert!(placement.len() == 3);
        Ok(())
    }

    #[test]
    fn test_placement_visit() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 4, 5);
        let visitor = PlacementMapConstVisitor::new(eval_date);
        let placement = visitor.visit(&instruments.as_slice())?;

        assert!(placement.get(&Date::new(2020, 5, 1)).unwrap().clone() == 100.0);
        assert!(placement.get(&Date::new(2020, 7, 1)).unwrap().clone() == 100.0);
        assert!(placement.get(&Date::new(2020, 9, 1)).unwrap().clone() == 100.0);
        assert!(placement.len() == 3);

        let instruments = make_test_instrument(Side::Pay)?;
        let placement = visitor.visit(&instruments.as_slice())?;

        assert!(placement.get(&Date::new(2020, 5, 1)).unwrap().clone() == -100.0);
        assert!(placement.get(&Date::new(2020, 7, 1)).unwrap().clone() == -100.0);
        assert!(placement.get(&Date::new(2020, 9, 1)).unwrap().clone() == -100.0);
        assert!(placement.len() == 3);
        Ok(())
    }
}
