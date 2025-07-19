use std::collections::BTreeMap;

use super::{placementconstvisitor::get_placement_map, redemptionsconstvisitor::get_redemption_map};
use crate::{
    cashflows::{cashflow::Cashflow, traits::Payable},
    instruments::bonds::traits::InteresAccrualAtYieldRate,
    time::{date::Date, enums::TimeUnit, period::Period},
    utils::errors::Result,
    visitors::traits::{ConstVisit, HasCashflows},
};

/// ## get_outstandings_at_date
/// Returns the outstandings at a given date for one instrument
pub fn get_outstanding_at_date_single_instrument<T: HasCashflows>(
    instrument: &T,
    eval_date: Date,
) -> Result<f64> {
    let mut sum = 0.0;
    instrument.cashflows().for_each(|cf| match cf {
        Cashflow::Disbursement(f) => {
            let payment_date = f.payment_date();
            if payment_date >= eval_date {
                sum += f.amount().unwrap() * f.side().sign();
            }
        }
        Cashflow::Redemption(f) => {
            let payment_date = f.payment_date();
            if payment_date >= eval_date {
                sum += f.amount().unwrap() * f.side().sign();
            }
        }
        _ => {}
    });
    Ok(sum)
}

/// ## get_outstandings_at_date
/// Returns the outstandings at a given date.
pub fn get_outstandings_at_date<T: HasCashflows>(
    instruments: &[T],
    eval_date: Date,
) -> Result<f64> {
    instruments.iter().try_fold(0.0, |acc, inst| {
        let local_sum = get_outstanding_at_date_single_instrument(inst, eval_date)?;
        Ok(acc + local_sum)
    })
}

/// ## CashAccountMapConstVisitor
/// Visitor to calculate the cash account for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the cash account as the value.
///
/// Parameters
/// * `eval_date: Date` - The evaluation date.
///
/// Output
/// * `BTreeMap<Date, f64>` - A map with the date as the key and the cash account as the value.
pub struct CashAccountMapConstVisitor {
    eval_date: Date,
}

impl CashAccountMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        CashAccountMapConstVisitor {
            eval_date: eval_date,
        }
    }
}

impl<T: HasCashflows> ConstVisit<&[T]> for CashAccountMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_cash_account_map(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_cash_account
/// Function to calculate the cash account for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the cash account as the value.
pub fn get_cash_account_map<T: HasCashflows>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let redemptions = get_redemption_map(instruments, eval_date)?;
    let placement = get_placement_map(instruments, eval_date)?;

    let last_date = redemptions.keys().max().unwrap_or(&eval_date).clone();
    let mut start_date = eval_date;
    let mut cash_account_map = BTreeMap::new();
    let mut cash_account = 0.0;

    cash_account_map.insert(start_date, cash_account);

    while start_date < last_date {
        let redemption = redemptions.get(&start_date).unwrap_or(&0.0);
        let placement = placement.get(&start_date).unwrap_or(&0.0);
        start_date = start_date + Period::new(1, TimeUnit::Days);
        cash_account += redemption - placement;
        cash_account_map.insert(start_date, cash_account);
    }
    Ok(cash_account_map)
}

/// ## OutstandingMapConstVisitor
/// Visitor to calculate the outstandings for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the outstandings as the value.
///
/// Parameters
///  * `eval_date: Date` - The evaluation date.
///     
/// Output
/// * `BTreeMap<Date, f64>` - A map with the date as the key and the outstandings as the value.
pub struct OutstandingMapConstVisitor {
    eval_date: Date,
}

impl OutstandingMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        OutstandingMapConstVisitor {
            eval_date: eval_date,
        }
    }
}

impl<T: HasCashflows> ConstVisit<&[T]> for OutstandingMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_outstanding_map(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_outstandings
/// Function to calculate the outstandings for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the outstandings as the value.
pub fn get_outstanding_map<T: HasCashflows>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let redemptions = get_redemption_map(instruments, eval_date)?;
    let placement = get_placement_map(instruments, eval_date)?;

    let last_date = redemptions.keys().max().unwrap_or(&eval_date).clone();
    let mut start_date = eval_date;

    let mut outstandings_map = BTreeMap::new();
    let mut outstanding = get_outstandings_at_date(instruments, eval_date)?;

    outstandings_map.insert(start_date, outstanding);

    while start_date < last_date {
        let redemption = redemptions.get(&start_date).unwrap_or(&0.0);
        let placement = placement.get(&start_date).unwrap_or(&0.0);
        start_date = start_date + Period::new(1, TimeUnit::Days);
        outstanding += -redemption + placement;
        outstandings_map.insert(start_date, outstanding);
    }
    Ok(outstandings_map)
}

/// ## CleanPriceMapConstVisitor
/// Visitor to calculate the clean price for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the clean price as the value.
///
/// Parameters
///  * `eval_date: Date` - The evaluation date.
///     
/// Output
/// * `BTreeMap<Date, f64>` - A map with the date as the key and the outstandings as the value.
pub struct CleanPriceMapConstVisitor {
    eval_date: Date,
}

impl CleanPriceMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        CleanPriceMapConstVisitor {
            eval_date: eval_date,
        }
    }
}

impl<T: InteresAccrualAtYieldRate> ConstVisit<&[T]> for CleanPriceMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_clean_price_map(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_clean_price_at_yield_rate
/// Function to calculate the clean price for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the clean price as the value.
pub fn get_clean_price_map<T: InteresAccrualAtYieldRate>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let mut clean_price_map = BTreeMap::new();

    instruments.iter().try_for_each(|inst: &T| -> Result<()> {
        let mut start_date = inst.purchase_date() + Period::new(1, TimeUnit::Days);
        let end_date = inst.accrual_end_date()?;
        if eval_date > end_date {
            return Ok(());
        }
        if eval_date > start_date {
            start_date = eval_date;
        }

        while start_date <= end_date {
            let clean_price = inst.clean_price(start_date)?;
            clean_price_map
                .entry(start_date)
                .and_modify(|e| *e += clean_price)
                .or_insert(clean_price);

            start_date = start_date + Period::new(1, TimeUnit::Days);
        }
        Ok(())
    })?;
    Ok(clean_price_map)
}

/// ## DirtyPriceMapConstVisitor
/// Visitor to calculate the dirty price for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the dirty price as the value.
///
pub struct DirtyPriceMapConstVisitor {
    eval_date: Date,
}

impl DirtyPriceMapConstVisitor {
    pub fn new(eval_date: Date) -> Self {
        DirtyPriceMapConstVisitor {
            eval_date: eval_date,
        }
    }
}

impl<T: InteresAccrualAtYieldRate> ConstVisit<&[T]> for DirtyPriceMapConstVisitor {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, inst: &&[T]) -> Self::Output {
        let map = get_dirty_price_map(inst, self.eval_date)?;
        Ok(map)
    }
}

/// ## get_dirty_price_at_yield_rate
/// Function to calculate the dirty price for a given set of instruments, starting from a given evaluation date.
/// The result is a map with the date as the key and the dirty price as the value.
pub fn get_dirty_price_map<T: InteresAccrualAtYieldRate>(
    instruments: &[T],
    eval_date: Date,
) -> Result<BTreeMap<Date, f64>> {
    let mut dirty_price_map = BTreeMap::new();

    instruments.iter().try_for_each(|inst: &T| -> Result<()> {
        let mut start_date = inst.purchase_date() + Period::new(1, TimeUnit::Days);
        let end_date = inst.accrual_end_date()?;
        if eval_date > end_date {
            return Ok(());
        }
        if eval_date > start_date {
            start_date = eval_date;
        }

        while start_date <= end_date {
            let clean_price = inst.dirty_price(start_date)?;
            dirty_price_map
                .entry(start_date)
                .and_modify(|e| *e += clean_price)
                .or_insert(clean_price);

            start_date = start_date + Period::new(1, TimeUnit::Days);
        }
        Ok(())
    })?;
    Ok(dirty_price_map)
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

    pub fn make_test_instrument(side: Side) -> Result<Vec<Instrument>> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let mut instruments = Vec::new();
        let item = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Monthly)
            .with_rate(rate)
            .with_notional(100.0)
            .with_side(side)
            .with_currency(Currency::CLP)
            .equal_redemptions()
            .build()?;
        instruments.push(Instrument::FixedRateInstrument(item));
        Ok(instruments)
    }

    #[test]
    fn test_get_outstandings_at_date() -> Result<()> {
        // Receive side
        let instruments = make_test_instrument(Side::Receive)?;

        let eval_date = Date::new(2020, 1, 1);
        let outstandings = get_outstandings_at_date(&instruments, eval_date);
        assert!((outstandings.unwrap() - 0.0).abs() < 1e-6);

        let eval_date = Date::new(2020, 1, 2);
        let outstandings = get_outstandings_at_date(&instruments, eval_date);
        assert!((outstandings.unwrap() - 100.0).abs() < 1e-6);

        let eval_date = Date::new(2020, 2, 2);
        let outstandings = get_outstandings_at_date(&instruments, eval_date);
        assert!((outstandings.unwrap() - 91.666666666).abs() < 1e-6);

        // Pay side
        let instruments = make_test_instrument(Side::Pay)?;

        let eval_date = Date::new(2020, 1, 1);
        let outstandings = get_outstandings_at_date(&instruments, eval_date);
        assert!((outstandings.unwrap() + 0.0).abs() < 1e-6);

        let eval_date = Date::new(2020, 1, 2);
        let outstandings = get_outstandings_at_date(&instruments, eval_date);
        assert!((outstandings.unwrap() + 100.0).abs() < 1e-6);

        let eval_date = Date::new(2020, 2, 2);
        let outstandings = get_outstandings_at_date(&instruments, eval_date);
        assert!((outstandings.unwrap() + 91.666666666).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_get_cash_account() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2019, 12, 31);
        let cash_account = get_cash_account_map(&instruments, eval_date)?;

        let date = Date::new(2019, 12, 31);
        assert!((cash_account.get(&date).unwrap() - 0.0).abs() < 0.00001);

        let date = Date::new(2020, 1, 1);
        assert!((cash_account.get(&date).unwrap() - 0.0).abs() < 0.00001);

        let date = Date::new(2020, 1, 2);
        assert!((cash_account.get(&date).unwrap() + 100.0).abs() < 0.00001);

        let eval_date = Date::new(2020, 01, 5);
        let cash_account = get_cash_account_map(&instruments, eval_date)?;

        let date = Date::new(2020, 1, 5);
        assert!((cash_account.get(&date).unwrap() + 0.0).abs() < 0.00001);

        let date = Date::new(2020, 2, 2);
        assert!((cash_account.get(&date).unwrap() - 8.333333333333).abs() < 0.00001);

        Ok(())
    }

    #[test]
    fn test_cost_visit_cash_account() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2019, 12, 31);
        let visitor = CashAccountMapConstVisitor::new(eval_date);
        let cash_account = visitor.visit(&instruments.as_slice())?;

        let date = Date::new(2019, 12, 31);
        assert!((cash_account.get(&date).unwrap() - 0.0).abs() < 0.00001);

        let date = Date::new(2020, 1, 1);
        assert!((cash_account.get(&date).unwrap() - 0.0).abs() < 0.00001);

        let date = Date::new(2020, 1, 2);
        assert!((cash_account.get(&date).unwrap() + 100.0).abs() < 0.00001);

        let eval_date = Date::new(2020, 01, 5);
        let visitor = CashAccountMapConstVisitor::new(eval_date);
        let cash_account = visitor.visit(&instruments.as_slice())?;

        let date = Date::new(2020, 1, 5);
        assert!((cash_account.get(&date).unwrap() + 0.0).abs() < 0.00001);

        let date = Date::new(2020, 2, 2);
        assert!((cash_account.get(&date).unwrap() - 8.333333333333).abs() < 0.00001);

        Ok(())
    }

    #[test]
    fn test_get_outstandings() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 1, 1);
        let outstandings = get_outstanding_map(&instruments, eval_date)?;

        let dates = vec![
            (Date::new(2020, 1, 1), 0.0),
            (Date::new(2020, 1, 2), 100.0),
            (Date::new(2020, 2, 1), 100.0),
        ];

        for (date, value) in dates {
            assert!((outstandings.get(&date).unwrap() - value).abs() < 1e-6);
        }

        let instruments = make_test_instrument(Side::Pay)?;
        let outstandings = get_outstanding_map(&instruments, eval_date)?;

        let dates = vec![
            (Date::new(2020, 1, 1), -0.0),
            (Date::new(2020, 1, 2), -100.0),
            (Date::new(2020, 2, 1), -100.0),
        ];

        for (date, value) in dates {
            assert!((outstandings.get(&date).unwrap() - value).abs() < 1e-6);
        }

        Ok(())
    }

    #[test]
    fn test_cost_visit_outstandings() -> Result<()> {
        let instruments = make_test_instrument(Side::Receive)?;
        let eval_date = Date::new(2020, 1, 1);
        let visitor = OutstandingMapConstVisitor::new(eval_date);
        let outstandings = visitor.visit(&instruments.as_slice())?;

        let dates = vec![
            (Date::new(2020, 1, 1), 0.0),
            (Date::new(2020, 1, 2), 100.0),
            (Date::new(2020, 2, 1), 100.0),
        ];

        for (date, value) in dates {
            assert!((outstandings.get(&date).unwrap() - value).abs() < 1e-6);
        }

        let instruments = make_test_instrument(Side::Pay)?;
        let outstandings = visitor.visit(&instruments.as_slice())?;

        let dates = vec![
            (Date::new(2020, 1, 1), -0.0),
            (Date::new(2020, 1, 2), -100.0),
            (Date::new(2020, 2, 1), -100.0),
        ];

        for (date, value) in dates {
            assert!((outstandings.get(&date).unwrap() - value).abs() < 1e-6);
        }

        Ok(())
    }
}
