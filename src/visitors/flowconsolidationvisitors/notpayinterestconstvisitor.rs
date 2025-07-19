use crate::{
    cashflows::{cashflow::Cashflow, traits::InterestAccrual}, instruments::bonds::traits::InteresAccrualAtYieldRate, time::{date::Date, enums::TimeUnit, period::Period}, utils::errors::Result, visitors::traits::{ConstVisit, HasCashflows}
};
use std::collections::BTreeMap;

/// ## NotPayInterestMapConstVisitor
/// Calculates interest not yet paid on an instrument at a given date.
/// 
/// ## Parameters
/// 
/// * `eval_date: Date` - The evaluation date.
/// 
/// ## Output
/// * `BTreeMap<Date, f64>` - A map with the date as the key and the not paid interest amount as the value.
pub struct NotPayInterestMapConstVisitor {
    eval_date: Date,
}

impl NotPayInterestMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        NotPayInterestMapConstVisitor { eval_date: eval_date }
    }
} 

impl<T: HasCashflows> ConstVisit<&[T]> for NotPayInterestMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let mut map = get_not_pay_interest_map(inst)?;
        // keep just dates > eval_date
        map.retain(|k, _| *k >= self.eval_date);
        Ok(map)
    }
}

/// ## get_not_pay_interest_map
/// Function to calculate the not paid interest amount for a given set of instruments.
/// The result is a map with the date as the key and the not paid interest amount as the value.
/// 
pub fn get_not_pay_interest_map<T: HasCashflows>(
    instruments: &[T],
) -> Result<BTreeMap<Date, f64>> {
    let delta: Period = Period::new(1, TimeUnit::Days);
    let accrued_amount = instruments
        .iter()
        .map(|inst: &T| -> Result<BTreeMap<Date, f64>> {
            let mut local_map = BTreeMap::new();
            inst.cashflows()
            .try_for_each(|cf| -> Result<()> {
                match cf {
                   Cashflow::FixedRateCoupon (coupon) => {
                        let star_accrual_date = cf.accrual_start_date()?;
                        let end_accrual_date = cf.accrual_end_date()?; 
                        let mut start_date = star_accrual_date + delta;
                        while start_date < end_accrual_date {
                            let accrued = coupon
                               .accrued_amount(star_accrual_date, start_date)?;
                            let entry = local_map.entry(start_date + delta).or_insert(0.0);
                            *entry += accrued;
                          
                            start_date = start_date + delta;
                        }
                   }
                   Cashflow::FloatingRateCoupon(coupon) => {
                        let star_accrual_date = cf.accrual_start_date()?;
                        let end_accrual_date = cf.accrual_end_date()?; 
                        let mut start_date = star_accrual_date + delta;
                        while start_date < end_accrual_date {
                            let accrued = coupon
                               .accrued_amount(star_accrual_date, start_date)?;
                            let entry = local_map.entry(start_date + delta).or_insert(0.0);
                            *entry += accrued;
                          
                            start_date = start_date + delta;
                        }
                   }
                   _ =>{}
                }
                Ok(())
            })?;
            Ok(local_map)
        })
        .try_fold(BTreeMap::new(), |mut acc, x| -> Result<_> {
            let x = x?;
            x.iter().for_each(|(k, v)| {
                let entry = acc.entry(*k).or_insert(0.0);
                *entry += *v;
            });
            Ok(acc)
        })?;

    Ok(accrued_amount)
}

/// ## NotPayInterestMapAtYieldRateConstVisitor
/// This visitor calculates the not paid interest for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the not paid interest amount as the value.
/// It is used to calculate the not paid interest at the yield rate.
/// 
/// Parameters
/// * `eval_date: Date` - The evaluation date.
/// 
/// Output
/// * `BTreeMap<Date, f64>` - A map with the date as the key and the not paid interest amount as the value.
pub struct NotPayInterestMapAtYieldRateConstVisitor {
    eval_date: Date,
}

impl NotPayInterestMapAtYieldRateConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        NotPayInterestMapAtYieldRateConstVisitor { eval_date: eval_date }
    }
}

impl<T: InteresAccrualAtYieldRate> ConstVisit<&[T]> for NotPayInterestMapAtYieldRateConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let mut map = get_not_pay_interest_map_at_yield_rate(inst)?;
        // keep just dates > eval_date
        map.retain(|k, _| *k >= self.eval_date);
        Ok(map)
    }
}

/// ## get_not_pay_interest_amount_at_yield_rate
/// Function to calculate the not paid interest amount for a given set of instruments.
/// The result is a map with the date as the key and the not paid interest amount as the value.
/// 
pub fn get_not_pay_interest_map_at_yield_rate<T: InteresAccrualAtYieldRate>(
    instruments: &[T],
) -> Result<BTreeMap<Date, f64>> {
    let delta: Period = Period::new(1, TimeUnit::Days);
    let accrued_amount = instruments
        .iter()
        .map(|inst: &T| -> Result<BTreeMap<Date, f64>> {
            let mut local_map = BTreeMap::new();
            let mut start_accrual_date = inst.accrual_start_date()?;
            let end_accrual_date = inst.accrual_end_date()?;
            while start_accrual_date < end_accrual_date {
                let nop_pay_interest = inst.not_pay_interest(start_accrual_date)?;
                let interest_cupon_amount = inst.interest_cupon_amount(start_accrual_date)?;
                start_accrual_date = start_accrual_date + delta;
                let entry = local_map.entry(start_accrual_date).or_insert(0.0);
                *entry += nop_pay_interest - interest_cupon_amount;
            }       
            Ok(local_map)
        })
        .try_fold(BTreeMap::new(), |mut acc, x| -> Result<_> {
            let x = x?;
            x.iter().for_each(|(k, v)| {
                let entry = acc.entry(*k).or_insert(0.0);
                *entry += *v;
            });
            Ok(acc)
        })?;

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
        time::{daycounter::DayCounter, enums::Frequency},
        utils::errors::Result,
    };

    pub fn make_test_instrument() -> Result<Vec<Instrument>> {
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
                .with_side(Side::Receive)
                .with_currency(Currency::CLP)
                .equal_redemptions()
                .build()?;
            instruments.push(Instrument::FixedRateInstrument(tem));
        }
        Ok(instruments)
    }

    #[test]
    fn test_get_notpay_interest_amount() -> Result<()> {
        let start_date = Date::new(2024, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.02,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let inst = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(100.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_payment_frequency(Frequency::Semiannual)
            .bullet()
            .build()?;

        let vec_inst = vec![Instrument::FixedRateInstrument(inst)];

        let not_pay_interest = get_not_pay_interest_map(&vec_inst)?;

        assert!((not_pay_interest.get(&Date::new(2024, 1, 3)).unwrap().clone() - 100.0 * ((1.0 + 0.02f64).powf(1.0 / 360.0) - 1.0)).abs() < 0.0000000000001);
        assert!((not_pay_interest.get(&Date::new(2024, 1, 4)).unwrap().clone() - 100.0 * ((1.0 + 0.02f64).powf(2.0 / 360.0) - 1.0)).abs() < 0.0000000000001);
        assert!((not_pay_interest.get(&Date::new(2024, 1, 5)).unwrap().clone() - 100.0 * ((1.0 + 0.02f64).powf(3.0 / 360.0) - 1.0)).abs() < 0.0000000000001);

        assert!((not_pay_interest.get(&Date::new(2024, 7, 3)).unwrap().clone() - 100.0 * ((1.0 + 0.02f64).powf(1.0 / 360.0) - 1.0)).abs() < 0.0000000000001);
        assert!((not_pay_interest.get(&Date::new(2024, 7, 4)).unwrap().clone() - 100.0 * ((1.0 + 0.02f64).powf(2.0 / 360.0) - 1.0)).abs() < 0.0000000000001);
        assert!((not_pay_interest.get(&Date::new(2024, 7, 5)).unwrap().clone() - 100.0 * ((1.0 + 0.02f64).powf(3.0 / 360.0) - 1.0)).abs() < 0.0000000000001);
    
        Ok(())
    }

    #[test]
    fn test_get_notpay_interest_amount_multiple_instruments() -> Result<()> {
        let instruments = make_test_instrument()?;
        let not_pay_interest = get_not_pay_interest_map(&instruments)?;

        for (date, value) in not_pay_interest.iter() {
            println!("Date: {:?}, Value: {:?}", date, value);
        }

        Ok(())
    }
}
