use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    cashflows::{
        cashflow::Cashflow,
        side::Side,
        traits::{InterestAccrual, Payable, Scalable},
    },
    core::traits::HasCurrency,
    currencies::enums::Currency,
    instruments::traits::Structure,
    rates::interestrate::RateDefinition,
    time::{date::Date, enums::Frequency},
    visitors::traits::HasCashflows,
};

use crate::utils::errors::Result;

/// # FloatingRateInstrument
/// A floating rate instrument.
///
/// ## Parameters
/// * `start_date` - The start date.
/// * `end_date` - The end date.
/// * `notional` - The notional.
/// * `spread` - The spread.
/// * `side` - The side.
/// * `cashflows` - The cashflows.
/// * `payment_frequency` - The payment frequency.
/// * `rate_definition` - The rate definition.
/// * `structure` - The structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloatingRateInstrument {
    start_date: Date,
    end_date: Date,
    notional: f64,
    spread: f64,
    side: Side,
    cashflows: Vec<Cashflow>,
    payment_frequency: Frequency,
    rate_definition: RateDefinition,
    structure: Structure,
    currency: Currency,
    discount_curve_id: Option<usize>,
    forecast_curve_id: Option<usize>,
    id: Option<String>,
    issue_date: Option<Date>,
}

impl FloatingRateInstrument {
    pub fn new(
        start_date: Date,
        end_date: Date,
        notional: f64,
        spread: f64,
        side: Side,
        cashflows: Vec<Cashflow>,
        payment_frequency: Frequency,
        rate_definition: RateDefinition,
        structure: Structure,
        currency: Currency,
        discount_curve_id: Option<usize>,
        forecast_curve_id: Option<usize>,
        id: Option<String>,
        issue_date: Option<Date>,
    ) -> Self {
        FloatingRateInstrument {
            start_date,
            end_date,
            notional,
            spread,
            side,
            cashflows,
            payment_frequency,
            rate_definition,
            structure,
            currency,
            discount_curve_id,
            forecast_curve_id,
            id,
            issue_date,
        }
    }

    pub fn issue_date(&self) -> Option<Date> {
        self.issue_date
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

    pub fn spread(&self) -> f64 {
        self.spread
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn payment_frequency(&self) -> Frequency {
        self.payment_frequency
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
    }

    pub fn structure(&self) -> Structure {
        self.structure
    }

    pub fn discount_curve_id(&self) -> Option<usize> {
        self.discount_curve_id
    }

    pub fn forecast_curve_id(&self) -> Option<usize> {
        self.forecast_curve_id
    }

    pub fn set_discount_curve_id(mut self, discount_curve_id: usize) -> Self {
        self.discount_curve_id = Some(discount_curve_id);
        self.mut_cashflows()
            .for_each(|cf| cf.set_discount_curve_id(discount_curve_id));
        self
    }

    pub fn set_forecast_curve_id(mut self, forecast_curve_id: usize) -> Self {
        self.forecast_curve_id = Some(forecast_curve_id);
        self.mut_cashflows()
            .for_each(|cf| cf.set_forecast_curve_id(forecast_curve_id));
        self
    }

    pub fn set_spread(mut self, spread: f64) -> Self {
        self.spread = spread;
        self.mut_cashflows().for_each(|cf| match cf {
            Cashflow::FloatingRateCoupon(coupon) => {
                coupon.set_spread(spread);
            }
            _ => {}
        });
        self
    }
}

impl HasCurrency for FloatingRateInstrument {
    fn currency(&self) -> Result<Currency> {
        Ok(self.currency)
    }
}

impl InterestAccrual for FloatingRateInstrument {
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

impl HasCashflows for FloatingRateInstrument {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.cashflows.iter())
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(self.cashflows.iter_mut())
    }
}

impl Scalable for FloatingRateInstrument {
    fn scale(&mut self, factor: f64) -> Result<()> {
        self.mut_cashflows()
            .try_for_each(|cashflow| cashflow.scale(factor))?;
        self.notional = self.notional() * factor;
        Ok(())
    }
}

/// # Display
/// Implement the display for FixedRateInstrument.
impl Display for FloatingRateInstrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Instrument: id: {:?}", self.id())?;
        writeln!(f, "\tnotional: {}", self.notional())?;
        writeln!(f, "\tcurrency: {:?}", self.currency())?;
        writeln!(f, "\tstart_date: {:?}", self.start_date())?;
        writeln!(f, "\tend_date: {:?}", self.end_date())?;
        writeln!(f, "\tstructure: {:?}", self.structure())?;
        writeln!(f, "\tpayment_frequency: {:?}", self.payment_frequency())?;
        writeln!(f, "\tside: {:?}", self.side())?;
        writeln!(f, "\trate definition: {:?}", self.rate_definition())?;
        writeln!(f, "\tspread: {}", self.spread())?;
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
mod tests{
    use crate::{
        cashflows::{
            cashflow::Cashflow,
            side::Side,
            traits::{Payable, RequiresFixingRate},
        },
        core::traits::HasCurrency,
        currencies::enums::Currency,
        instruments::constructors::makefloatingrateinstrument::MakeFloatingRateInstrument,
        rates::{enums::Compounding, interestrate::RateDefinition},
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
        visitors::traits::HasCashflows,
    };

    #[test]
    fn test_float_rate_instrument() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let spread = 0.04;

        let instrument = MakeFloatingRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate_definition(rate_definition)
            .with_payment_frequency(Frequency::Semiannual)
            .with_spread(spread)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        assert_eq!(instrument.start_date(), start_date);
        assert_eq!(instrument.end_date(), end_date);
        assert_eq!(instrument.notional(), 5_000_000.0);
        assert_eq!(instrument.spread(), spread);
        assert_eq!(instrument.side(), Side::Receive);
        assert_eq!(instrument.payment_frequency(), Frequency::Semiannual);
        assert_eq!(instrument.rate_definition(), rate_definition);
        assert_eq!(instrument.currency().unwrap(), Currency::USD);

        Ok(())
    }

    #[test]
    fn test_set_spread() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Simple,
            Frequency::Annual,
        );

        let spread = 0.04;

        let mut instrument = MakeFloatingRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate_definition(rate_definition)
            .with_payment_frequency(Frequency::Semiannual)
            .with_spread(spread)
            .with_notional(5_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        instrument
            .mut_cashflows()
            .for_each(|cf| cf.set_fixing_rate(0.02));

        instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FloatingRateCoupon(coupon) => {
                assert!((coupon.amount().unwrap() - 150000.0).abs() < 1e-6);
                assert_eq!(coupon.spread(), spread);
            }
            _ => {}
        });

        let new_spread = 0.01;
        let new_instrument = instrument.set_spread(new_spread);

        new_instrument.cashflows().for_each(|cf| match cf {
            Cashflow::FloatingRateCoupon(coupon) => {
                assert!((coupon.amount().unwrap() - 75000.0).abs() < 1e-6);
                assert_eq!(coupon.spread(), new_spread);
            }
            _ => {}
        });

        Ok(())
    }
}
