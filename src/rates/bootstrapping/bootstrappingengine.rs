use std::fmt::Display;

use crate::{
    currencies::enums::Currency,
    rates::bootstrapping::bootstrappingmarketstore::BootstrappingMarketStore,
    time::date::Date,
    utils::errors::{AtlasError, Result},
    visitors::traits::HasCashflows,
};

/// # BootstrappingEngine
/// BootstrappingEngine is the main structure that holds the bootstrapping process.
/// It contains the reference date, the market store, and the instruments to be bootstrapped
/// 
/// The correct use requires the following steps:
/// 1. Create a new BootstrappingEngine with the reference date and local currency.
/// 2. Add curves to the market store.
/// 3. Add instruments to the engine.
/// 
pub struct BootstrappingEngine {
    reference_date: Date,
    market_store: BootstrappingMarketStore,
    instruments: Vec<Box<dyn HasCashflows>>,
    number_of_instruments: usize,
}

impl BootstrappingEngine {
    pub fn new(reference_date: Date, local_currency: Currency) -> Self {
        BootstrappingEngine {
            reference_date,
            market_store: BootstrappingMarketStore::new(reference_date, local_currency),
            instruments: Vec::new(),
            number_of_instruments: 0,
        }
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn market_store(&self) -> &BootstrappingMarketStore {
        &self.market_store
    }

    pub fn market_store_mut(&mut self) -> &mut BootstrappingMarketStore {
        &mut self.market_store
    }

    pub fn number_of_instruments(&self) -> usize {
        self.number_of_instruments
    }

    pub fn add_instrument(
        &mut self,
        curve_id: usize,
        end_date: Date,
        instrument: Box<dyn HasCashflows>,
    ) -> Result<()> {
        let id = self.number_of_instruments();
        let curves_map = self.market_store.curves_map_mut();
        curves_map
            .get_mut(&curve_id)
            .ok_or(AtlasError::BootstrappingErr(format!(
                "Curve with id {} not found",
                curve_id
            )))?
            .add_date(end_date, id)?;
        self.number_of_instruments += 1;
        self.instruments.push(instrument);
        Ok(())
    }

    pub fn update_discount_factors(&mut self, new_discount_factors: Vec<f64>) -> Result<()> {
        if new_discount_factors.len() != self.number_of_instruments() {
            return Err(AtlasError::BootstrappingErr(format!(
                "Number of new discount factors ({}) does not match number of instruments ({})",
                new_discount_factors.len(),
                self.number_of_instruments()
            )));
        }

        let curves_map = self.market_store.curves_map_mut();
        for (_, curve) in curves_map.iter_mut() {
            let index = curve.discount_factor_index().clone();
            let discount_factors = curve.discount_factors_mut();
            for (df, idx) in discount_factors.iter_mut().zip(index.iter()) {
                if let Some(id) = idx {
                    *df = new_discount_factors[id.clone()];
                }
            }
        }
        Ok(())
    }

    pub fn instruments(&self) -> &Vec<Box<dyn HasCashflows>> {
        &self.instruments
    }

    pub fn instruments_mut(&mut self) -> &mut Vec<Box<dyn HasCashflows>> {
        &mut self.instruments
    }
}



use colored::*; 
impl Display for BootstrappingEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let curves = self.market_store.curves_map();
        write!(f, "\n")?;
        writeln!(f, "{}", "=====================================".blue().bold())?;
        writeln!(f, "{}", "Bootstrapping Engine:".bold().underline().blue())?;
        writeln!(f, "{} {}", "Reference Date:".yellow(), self.reference_date)?;
        writeln!(f, "{} {}", "Local Currency:".yellow(), self.market_store.local_currency())?;
        writeln!(f, "{} {}", "Number of Instruments:".yellow(), self.number_of_instruments)?;
        writeln!(f, "{}", "Curves:".bold().green())?;
        for (id, curve) in curves.iter() {
            writeln!(
                f,
                "{} {}, {} {}",
                "  Curve ID:".cyan(),
                id,
                "Currency:".cyan(),
                curve.currency()
            )?;
            curve.dates()
                .iter()
                .zip(curve.discount_factors().iter())
                .for_each(|(date, df)| {
                    writeln!(
                        f,
                        "{} {}, {} {}",
                        "    Date:".magenta(),
                        date,
                        "Discount Factor:".magenta(),
                        df
                    ).unwrap();
                });
        }
        writeln!(f, "{}", "=====================================".blue().bold())
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        cashflows::side::Side,
        currencies::enums::Currency,
        instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument,
        models::traits::Model,
        rates::{
            bootstrapping::{
                bootstrappingengine::{BootstrappingEngine},
                bootstrappingmarketstore::BootstrappingModel,
            },
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
        visitors::{
            indexingvisitors::indexingvisitor::IndexingVisitor,
            npvvisitors::npvconstvisitor::NPVConstVisitor,
            traits::{ConstVisit, Visit},
        },
    };

    #[test]
    fn test_bootstrapping_engine_add_curve() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(1, Currency::USD)?;
        assert!(engine.market_store().curves_map().contains_key(&1));
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_add_instrument() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(1, Currency::USD)?;

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

    #[test]
    fn test_bootstrapping_engine_npv_valuation() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;

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

        let indexer = IndexingVisitor::new();
        engine
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;

        let model = BootstrappingModel::new(engine.market_store());

        let data = model.gen_market_data(&indexer.request()).unwrap();

        let npv_visitor = NPVConstVisitor::new(&data, true);
        engine
            .instruments()
            .iter()
            .try_for_each(|instrument| -> Result<()> {
                let npv = npv_visitor.visit(&instrument)?;
                let expected_npv = 1_000_000.0 * 0.05 / 360.0 * 365.0;
                assert!((npv - expected_npv).abs() < 1e-6);
                Ok(())
            })?;

        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_update_discount_factors() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;

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

        let indexer = IndexingVisitor::new();
        engine
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;

        engine.update_discount_factors(vec![0.5])?;
        let model = BootstrappingModel::new(engine.market_store());

        let data = model.gen_market_data(&indexer.request()).unwrap();

        let npv_visitor = NPVConstVisitor::new(&data, true);
        engine
            .instruments()
            .iter()
            .try_for_each(|instrument| -> Result<()> {
                let npv = npv_visitor.visit(&instrument)?;
                let expected_npv = 1_000_000.0 * (1.0 + 0.05 / 360.0 * 365.0) * 0.5 - 1_000_000.0;
                assert!((npv - expected_npv).abs() < 1e-6);
                Ok(())
            })?;

        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_npv_valuation_with_two_instruments() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;
        bootstrappingmarketstore.add_curve(3, Currency::USD)?;

        let end_date = ref_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let inst1 = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(1)) // Assuming the curve ID is 1
            .zero()
            .build()?;

        engine.add_instrument(1, end_date, Box::new(inst1))?;

        let inst2 = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(3)) // Assuming the curve ID is 1
            .zero()
            .build()?;

        engine.add_instrument(3, end_date, Box::new(inst2))?;

        let indexer = IndexingVisitor::new();
        engine
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;

        let model = BootstrappingModel::new(engine.market_store());

        let data = model.gen_market_data(&indexer.request()).unwrap();

        let npv_visitor = NPVConstVisitor::new(&data, true);
        engine
            .instruments()
            .iter()
            .try_for_each(|instrument| -> Result<()> {
                let npv = npv_visitor.visit(&instrument)?;
                let expected_npv = 1_000_000.0 * 0.05 / 360.0 * 365.0;
                assert!((npv - expected_npv).abs() < 1e-6);
                Ok(())
            })?;

        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_update_discount_factors_with_two_instruments() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;
        bootstrappingmarketstore.add_curve(3, Currency::USD)?;

        let end_date = ref_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let inst1 = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(1)) // Assuming the curve ID is 1
            .zero()
            .build()?;

        engine.add_instrument(1, end_date, Box::new(inst1))?;

        let inst2 = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(3)) // Assuming the curve ID is 1
            .zero()
            .build()?;

        engine.add_instrument(3, end_date, Box::new(inst2))?;

        let indexer = IndexingVisitor::new();
        engine
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;

        engine.update_discount_factors(vec![0.5, 0.25])?;
        let model = BootstrappingModel::new(engine.market_store());

        let data = model.gen_market_data(&indexer.request()).unwrap();

        let npv_visitor = NPVConstVisitor::new(&data, true);
        engine.instruments().get(0).map(|instrument| -> Result<()> {
            let npv = npv_visitor.visit(&instrument)?;
            let expected_npv = 1_000_000.0 * (1.0 + 0.05 / 360.0 * 365.0) * 0.5 - 1_000_000.0;
            assert!((npv - expected_npv).abs() < 1e-6);

            Ok(())
        });

        engine.instruments().get(1).map(|instrument| -> Result<()> {
            let npv = npv_visitor.visit(&instrument)?;
            let expected_npv = 1_000_000.0 * (1.0 + 0.05 / 360.0 * 365.0) * 0.25 - 1_000_000.0;
            assert!((npv - expected_npv).abs() < 1e-6);
            Ok(())
        });

        Ok(())
    }


}
