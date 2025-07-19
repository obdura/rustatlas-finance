use crate::{cashflows::{cashflow::Cashflow, traits::Payable}, instruments::bonds::traits::InteresAccrualAtYieldRate, time::{date::Date, enums::TimeUnit, period::Period}, utils::errors::Result, visitors::traits::{ConstVisit, HasCashflows}};
use std::collections::BTreeMap;


/// ## InterestPaymentMapConstVisitor
/// This visitor calculates the interest payment map for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the interest payment as the value.
/// 
pub struct InterestPaymentMapConstVisitor {
    eval_date: Date,
}

impl InterestPaymentMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        InterestPaymentMapConstVisitor { eval_date: eval_date }
    }
}

impl<T: HasCashflows> ConstVisit<&[T]> for InterestPaymentMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_interest_payment_map(inst, self.eval_date)?;
        Ok(map)
    }
}


/// ## get_interest_payment_map
/// Function to calculate the interest payment map for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the interest payment as the value.
pub fn get_interest_payment_map<T: HasCashflows>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let redemptions = instruments
        .iter()
        .map(|inst| {
            let mut local_map = BTreeMap::new();
            inst.cashflows().for_each(|cf| match cf {
                Cashflow::FixedRateCoupon(frc) => {
                    let payment_date = frc.payment_date();
                    if payment_date >= eval_date {
                        let amount = frc.amount().unwrap();
                        let entry = local_map.entry(payment_date).or_insert(0.0);
                        *entry += amount * frc.side().sign();
                    }
                }
                Cashflow::FloatingRateCoupon(frc) => {
                    let payment_date = frc.payment_date();
                    if payment_date >= eval_date {
                        let amount = frc.amount().unwrap();
                        let entry = local_map.entry(payment_date).or_insert(0.0);
                        *entry += amount * frc.side().sign();
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

    Ok(redemptions)
}


/// ## InterestPaymentMapAtYieldRateConstVisitor
/// This visitor calculates the interest payment map for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the interest payment as the value.
/// It is used to calculate the interest payment map at the yield rate.
/// 
/// Parameters
///   * `eval_date: Date` - The evaluation date.
/// 
/// Output
///  * `BTreeMap<Date, f64>` - A map with the date as the key and the cupon interest as the value.
pub struct InterestPaymentMapAtYieldRateConstVisitor {
    eval_date: Date,
}

impl InterestPaymentMapAtYieldRateConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        InterestPaymentMapAtYieldRateConstVisitor { eval_date: eval_date }
    }
}

impl<T: InteresAccrualAtYieldRate> ConstVisit<&[T]> for InterestPaymentMapAtYieldRateConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let mut map = get_interest_payment_map_at_yield_rate(inst)?;
        map.retain(|k, _| *k >= self.eval_date);
        Ok(map)
    }
}

/// ## get_interest_payment_map_at_yield_rate
/// Function to calculate the interest payment map for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the interest payment as the value.
pub fn get_interest_payment_map_at_yield_rate<T: InteresAccrualAtYieldRate>(
    instruments: &[T],
) -> Result<BTreeMap<Date, f64>> {
    let mut redemption = BTreeMap::new();
    let _ = instruments.iter().try_for_each(|inst| -> Result<()> {
        let mut start_date = inst.accrual_start_date()? + Period::new(1, TimeUnit::Days);
        let end_date = inst.accrual_end_date()?;
        while start_date <= end_date {
            let notional_cupon_amount = inst.interest_cupon_amount(start_date)?;
            let entry = redemption.entry(start_date).or_insert(0.0);
            *entry += notional_cupon_amount;
            start_date = start_date + Period::new(1, TimeUnit::Days);
        }
        Ok(())
    });
    
    Ok(redemption)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cashflows::side::Side, currencies::enums::Currency, instruments::{constructors::makefixedrateinstrument::MakeFixedRateInstrument, loandepos::instrument::Instrument}, rates::{enums::Compounding, interestrate::InterestRate}, time::{daycounter::DayCounter, enums::{Frequency, TimeUnit}, period::Period}, utils::errors::Result};

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
    fn test_get_cupon_interest() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 4, 5);
        let cupon_interest = get_interest_payment_map(&instruments, eval_date)?;
        assert!(cupon_interest.get(&Date::new(2020, 6, 1)).unwrap().clone() == 1.2209618854308135);
        assert!(cupon_interest.get(&Date::new(2020, 7, 1)).unwrap().clone() == 1.1611252783397807);
        assert!(cupon_interest.get(&Date::new(2025, 5, 1)).unwrap().clone() == 0.06111185675472461);

        let instruments = make_test_instrument(Side::Pay)?;
        let cupon_interest = get_interest_payment_map(&instruments, eval_date)?;
        assert!(cupon_interest.get(&Date::new(2020, 6, 1)).unwrap().clone() == - 1.2209618854308135);
        assert!(cupon_interest.get(&Date::new(2020, 7, 1)).unwrap().clone() == - 1.1611252783397807);
        assert!(cupon_interest.get(&Date::new(2025, 5, 1)).unwrap().clone() == - 0.06111185675472461);
        Ok(())
    }

    #[test]
    fn test_get_cupon_interest_const_visitor() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 4, 5);
        let visitor = InterestPaymentMapConstVisitor::new(eval_date);
        let cupon_interest = visitor.visit(&instruments.as_slice())?;
        assert!(cupon_interest.get(&Date::new(2020, 6, 1)).unwrap().clone() == 1.2209618854308135);
        assert!(cupon_interest.get(&Date::new(2020, 7, 1)).unwrap().clone() == 1.1611252783397807);
        assert!(cupon_interest.get(&Date::new(2025, 5, 1)).unwrap().clone() == 0.06111185675472461);
        Ok(())
    }
}
