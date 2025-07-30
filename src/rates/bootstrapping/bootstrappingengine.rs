use std::fmt::Display;

use crate::{
    currencies::enums::Currency,
    rates::{
        bootstrapping::bootstrappingmarketstore::BootstrappingMarketStore,
        traits::HasReferenceDate,
    },
    time::date::Date,
    utils::errors::{AtlasError, Result}, visitors::traits::HasCashflows,
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
    optimization_order: Vec<(usize, usize)>,
}

impl BootstrappingEngine {
    pub fn new(reference_date: Date, local_currency: Currency) -> Self {
        BootstrappingEngine {
            reference_date,
            market_store: BootstrappingMarketStore::new(reference_date, local_currency),
            instruments: Vec::new(),
            number_of_instruments: 0,
            optimization_order: Vec::new(),
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

    pub fn instruments(&self) -> &Vec<Box<dyn HasCashflows>> {
        &self.instruments
    }

    pub fn instruments_mut(&mut self) -> &mut Vec<Box<dyn HasCashflows>> {
        &mut self.instruments
    }

    pub fn optimization_order(&self) -> &Vec<(usize, usize)> {
        &self.optimization_order
    }

    pub fn optimization_order_len(&self) -> usize {
        self.optimization_order.len()
    }

    pub fn set_optimization_order(&mut self, order: Vec<(usize, usize)>) {
        self.optimization_order = order;
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

    pub fn update_discount_factors(&mut self, new_discount_factors: &[f64]) -> Result<()> {
        if new_discount_factors.len() != self.optimization_order.len() {
            return Err(AtlasError::BootstrappingErr(format!(
                "Number of new discount factors ({}) does not match the number indexes ({})",
                new_discount_factors.len(),
                self.number_of_instruments()
            )));
        }

        let curves_map = self.market_store.curves_map_mut();
        for ((curve_id, element_id), &new_df) in self.optimization_order.iter().zip(new_discount_factors.iter()) {
            let curve = curves_map.get_mut(curve_id).ok_or_else(|| {
                AtlasError::BootstrappingErr(format!("Curve with id {} not found", curve_id))
            })?;
            let discount_factors = curve.discount_factors_mut();
            if let Some(df) = discount_factors.get_mut(*element_id) {
                *df = new_df;
            } else {
                return Err(AtlasError::BootstrappingErr(format!(
                    "Index {} out of bounds for curve {}",
                    element_id, curve_id
                )));
            }
        }
        Ok(())
    }

    pub fn relevant_instruments(&self) -> Vec<usize> {
        let mut relevant_instruments = Vec::new();
        for (curve_id, element_id) in self.optimization_order.iter() {
            if let Some(curve) = self.market_store.curves_map().get(curve_id) {
                let id = curve
                    .related_instrument_index()
                    .get(*element_id)
                    .unwrap()
                    .unwrap();
                relevant_instruments.push(id);
            }
        }
        relevant_instruments
    }

    pub fn relevant_discount_factors(&self) -> Vec<f64> {
        let mut relevant_discount_factors = Vec::new();
        for (curve_id, element_id) in self.optimization_order.iter() {
            if let Some(curve) = self.market_store.curves_map().get(curve_id) {
                let id = curve.discount_factors().get(*element_id).unwrap().clone();
                relevant_discount_factors.push(id);
            }
        }
        relevant_discount_factors
    }

    pub fn estimated_relevant_discount_factors(&self, curve_id: usize) -> Vec<(usize, usize)> {
        let curve = self.market_store.curves_map().get(&curve_id);
        if let Some(curve) = curve {
            curve.dates().iter().skip(1).enumerate().map(|(i, _)| (curve_id, i+1)).collect()
        } else {
            vec![]
        }
    }
}

use colored::*;
impl Display for BootstrappingEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let curves = self.market_store.curves_map();
        write!(f, "\n")?;
        writeln!(
            f,
            "{}",
            "=====================================".blue().bold()
        )?;
        writeln!(f, "{}", "Bootstrapping Engine:".bold().underline().blue())?;
        writeln!(f, "{} {}", "Reference Date:".yellow(), self.reference_date)?;
        writeln!(
            f,
            "{} {}",
            "Local Currency:".yellow(),
            self.market_store.local_currency()
        )?;
        writeln!(
            f,
            "{} {}",
            "Number of Instruments:".yellow(),
            self.number_of_instruments
        )?;
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
            curve
                .dates()
                .iter()
                .zip(curve.discount_factors().iter())
                .for_each(|(date, df)| {
                    writeln!(
                        f,
                        "{} {}, {} {}, {} {}",
                        "    Date:".magenta(),
                        date,
                        "tenor:".magenta(),
                        curve.day_counter().day_count(curve.reference_date(), *date),
                        "Discount Factor:".magenta(),
                        df
                    )
                    .unwrap();
                });
        }
        writeln!(
            f,
            "{}",
            "=====================================".blue().bold()
        )
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
                bootstrappingengine::BootstrappingEngine,
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
    fn test_bootstrapping_engine_creation() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let engine = BootstrappingEngine::new(ref_date, Currency::USD);
        assert_eq!(engine.reference_date(), ref_date);
        assert_eq!(engine.market_store().local_currency(), Currency::USD);
        assert_eq!(engine.number_of_instruments(), 0);
        assert!(engine.instruments().is_empty());
        assert!(engine.optimization_order().is_empty());
        Ok(())
    }

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

        engine.set_optimization_order(vec![(1, 1)]);

        let indexer = IndexingVisitor::new();
        engine
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;

        engine.update_discount_factors(&vec![0.5])?;
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
        engine.set_optimization_order(vec![(1, 1), (3, 1)]);

        let indexer = IndexingVisitor::new();
        engine
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;

        engine.update_discount_factors(&vec![0.5, 0.25])?;
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

    #[test]
    fn test_bootstrapping_engine_relevant_instruments_and_discount_factors() -> Result<()> {
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
            .with_discount_curve_id(Some(1))
            .zero()
            .build()?;

        engine.add_instrument(1, end_date, Box::new(inst))?;
        engine.set_optimization_order(vec![(1, 1)]);

        let relevant_instruments = engine.relevant_instruments();
        assert_eq!(relevant_instruments.len(), 1);

        let relevant_discount_factors = engine.relevant_discount_factors();
        assert_eq!(relevant_discount_factors.len(), 1);

        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_set_optimization_order_and_len() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.set_optimization_order(vec![(1, 1), (2, 2), (3, 3)]);
        assert_eq!(engine.optimization_order_len(), 3);
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_update_discount_factors_invalid_len() {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.set_optimization_order(vec![(1, 1), (2, 2)]);
        let result = engine.update_discount_factors(&vec![0.5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bootstrapping_engine_update_discount_factors_invalid_curve() {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.set_optimization_order(vec![(99, 1)]);
        let result = engine.update_discount_factors(&vec![0.5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bootstrapping_engine_update_discount_factors_invalid_element() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(1, Currency::USD)?;
        engine.set_optimization_order(vec![(1, 99)]);
        let result = engine.update_discount_factors(&vec![0.5]);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_display_trait() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(1, Currency::USD)?;
        let display = format!("{}", engine);
        assert!(display.contains("Bootstrapping Engine:"));
        assert!(display.contains("Reference Date:"));
        assert!(display.contains("Local Currency:"));
        assert!(display.contains("Curves:"));
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_estimate_relevant_discount_factors() -> Result<()> {
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

        let end_date = ref_date + Period::new(2, TimeUnit::Years);
        let inst2 = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(1)) // Assuming the curve ID is 1
            .zero()
            .build()?;

        engine.add_instrument(1, end_date, Box::new(inst2))?;

        let estimated_dfs = engine.estimated_relevant_discount_factors(1);

        assert_eq!(estimated_dfs.len(), 2);
        assert_eq!(estimated_dfs, vec![(1, 1), (1, 2)]);

        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_estimate_relevant_discount_factors_empty_curve() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;

        let estimated_dfs = engine.estimated_relevant_discount_factors(1);
        assert!(estimated_dfs.is_empty(), "Expected no discount factors for empty curve");
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_estimate_relevant_discount_factors_not_set_curve() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;

        let estimated_dfs = engine.estimated_relevant_discount_factors(2);
        assert!(estimated_dfs.is_empty(), "Expected no discount factors for non-existent curve");
        Ok(())
    }

}
