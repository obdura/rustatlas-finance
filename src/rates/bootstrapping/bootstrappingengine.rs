use crate::{
    rates::bootstrapping::bootstrappingcurve::BootstrappingCurve,
    time::date::Date,
    utils::errors::{AtlasError, Result},
    visitors::traits::HasCashflows,
};
use std::collections::HashMap;

pub struct BootstrappingEngine {
    curves: HashMap<usize, BootstrappingCurve>,
    instruments: Vec<Box<dyn HasCashflows>>,
}

impl BootstrappingEngine {
    pub fn new() -> Self {
        BootstrappingEngine {
            curves: HashMap::new(),
            instruments: Vec::new(),
        }
    }

    pub fn add_curve(&mut self, curve: BootstrappingCurve) {
        self.curves.insert(curve.id(), curve);
    }

    pub fn add_instrument(
        &mut self,
        curve_id: usize,
        end_date: Date,
        instrument: Box<dyn HasCashflows>,
    ) -> Result<()> {
        self.curves
            .get_mut(&curve_id)
            .ok_or(AtlasError::BootstrappingErr(format!(
                "Curve with id {} not found",
                curve_id
            )))?
            .add_date(end_date)?;
        self.instruments.push(instrument);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        cashflows::side::Side,
        currencies::enums::Currency,
        instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument,
        rates::{
            bootstrapping::bootstrappingengine::{BootstrappingCurve, BootstrappingEngine},
            enums::Compounding,
            interestrate::InterestRate,
        },
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
    };

    #[test]
    fn test_bootstrapping_engine_add_curve() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let curve = BootstrappingCurve::new(ref_date, 1, Currency::USD);
        let mut engine = BootstrappingEngine::new();
        engine.add_curve(curve);
        assert!(engine.curves.contains_key(&1));
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_add_instrument() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let curve = BootstrappingCurve::new(ref_date, 1, Currency::USD);
        let mut engine = BootstrappingEngine::new();
        engine.add_curve(curve);

        let end_date = ref_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let inst = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(1)) // Assuming the curve ID is 1
            .zero()
            .build()?;

        engine.add_instrument(1, end_date, Box::new(inst))?;

        Ok(())
    }
}
