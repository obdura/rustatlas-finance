use argmin::{
    core::{CostFunction, Error, Executor, State},
    solver::brent::BrentRoot,
};

use crate::{
    instruments::{
        loandepos::{
            fixedrateinstrument::FixedRateInstrument,
            floatingrateinstrument::FloatingRateInstrument,
        },
        swaps::{crosscurrencyswap::CrossCurrencySwap, leg::Leg, vanillairsswap::VanillaIRSSwap},
        traits::Structure,
    },
    rates::interestrate::InterestRate,
    utils::errors::{AtlasError, Result},
    visitors::{
        npvvisitors::npvconstvisitor::NPVConstVisitor,
        traits::{ConstVisit, Visit},
    },
};

use super::traits::{ParValue, ParValueConstVisitor};

// cost function for fixed rate instrument
impl<'a> CostFunction for ParValue<'a, FixedRateInstrument> {
    type Param = f64;
    type Output = f64;
    fn cost(&self, param: &Self::Param) -> std::result::Result<Self::Output, Error> {
        let rate = self.eval.rate();
        let new_rate = InterestRate::new(
            *param,
            rate.compounding(),
            rate.frequency(),
            rate.day_counter(),
        );

        // new instrument with the new rate
        let inst = self.eval.clone().set_rate(new_rate)?;

        // visit the instrument to calculate the npv and return the result,
        let npv = self.npv_visitor.visit(&inst).map_err(|e| Error::from(e))?;

        // if the target cost is set, subtract it from the npv
        let target_cost = self.target_cost.unwrap_or(0.0);

        Ok(npv - target_cost)
    }
}

// cost function for floating rate instrument
impl<'a> CostFunction for ParValue<'a, FloatingRateInstrument> {
    type Param = f64;
    type Output = f64;
    fn cost(&self, param: &Self::Param) -> std::result::Result<Self::Output, Error> {
        let new_spread = *param;

        // new instrument with the new spread
        let mut inst = self.eval.clone().set_spread(new_spread);

        // visit the instrument to update the fixing values
        let _ = self.fixing_visitor.visit(&mut inst);

        // visit the instrument to calculate the npv and return the result
        let npv = self.npv_visitor.visit(&inst).map_err(|e| Error::from(e))?;

        // if the target cost is set, subtract it from the npv
        let target_cost = self.target_cost.unwrap_or(0.0);

        Ok(npv - target_cost)
    }
}

// cost function for leg
impl<'a> CostFunction for ParValue<'a, Leg> {
    type Param = f64;
    type Output = f64;
    fn cost(&self, param: &Self::Param) -> std::result::Result<Self::Output, Error> {
        let new_rate = *param;

        // new instrument with the new spread
        let mut inst = self.eval.clone().set_rate_value(new_rate);

        // visit the instrument to update the fixing values
        let _ = self.fixing_visitor.visit(&mut inst);

        // visit the instrument to calculate the npv and return the npv result
        let nvp = self.npv_visitor.visit(&inst).map_err(|e| Error::from(e))?;

        // if the target cost is set, subtract it from the npv
        let target_cost = self.target_cost.unwrap_or(0.0);

        Ok(nvp - target_cost)
    }
}

/// Implement the visit trait for ParValueConstVisitor for FixedRateInstrument
impl<'a> ConstVisit<FixedRateInstrument> for ParValueConstVisitor<'a> {
    type Output = Result<f64>;
    // visit fixed rate instrument
    // use BrentRoot solver to find the par rate
    fn visit(&self, instrument: &FixedRateInstrument) -> Self::Output {
        let (min, max) = match instrument.structure() {
            Structure::EqualPayments => (-0.7, 0.7),
            _ => (-1.0, 1.0),
        };

        let mut cost = ParValue::new(instrument, &self.market_data);
        cost.set_target_cost(self.target_cost.unwrap_or(0.0));

        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;

        let best_param = res.state().get_best_param();
        match best_param {
            Some(param) => Ok(*param),
            None => Err(AtlasError::EvaluationErr("No solution found".to_string())),
        }
    }
}

/// Implement the visit trait for ParValueConstVisitor for FloatingRateInstrument
impl<'a> ConstVisit<FloatingRateInstrument> for ParValueConstVisitor<'a> {
    type Output = Result<f64>;
    // visit floating rate instrument
    // use BrentRoot solver to find the par spread
    fn visit(&self, instrument: &FloatingRateInstrument) -> Self::Output {
        let (min, max) = (-1.0, 1.0);

        let mut cost = ParValue::new(instrument, &self.market_data);
        cost.set_target_cost(self.target_cost.unwrap_or(0.0));

        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;

        let best_param = res.state().get_best_param();
        match best_param {
            Some(param) => Ok(*param),
            None => Err(AtlasError::EvaluationErr("No solution found".to_string())),
        }
    }
}

/// Implement the visit trait for ParValueConstVisitor for Leg
impl<'a> ConstVisit<Leg> for ParValueConstVisitor<'a> {
    type Output = Result<f64>;
    // visit leg
    // use BrentRoot solver to find the par rate
    fn visit(&self, instrument: &Leg) -> Self::Output {
        let (min, max) = (-1.0, 1.0);

        let mut cost = ParValue::new(instrument, &self.market_data);
        cost.set_target_cost(self.target_cost.unwrap_or(0.0));

        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;

        let best_param = res.state().get_best_param();
        match best_param {
            Some(param) => Ok(*param),
            None => Err(AtlasError::EvaluationErr("No solution found".to_string())),
        }
    }
}

impl<'a> ConstVisit<VanillaIRSSwap> for ParValueConstVisitor<'a> {
    type Output = Result<f64>;
    // visit vanilla irs swap
    // use BrentRoot solver to find the par rate
    fn visit(&self, instrument: &VanillaIRSSwap) -> Self::Output {
        let first_leg = instrument.first_leg();
        let second_leg = instrument.second_leg();

        let nvp_visitor = NPVConstVisitor::new(&self.market_data, true);
        let nvp_first_leg = nvp_visitor.visit(first_leg)?;

        let target_cost = self.target_cost.unwrap_or(0.0) - nvp_first_leg;

        let (min, max) = (-1.0, 1.0);

        let mut cost = ParValue::new(second_leg, &self.market_data);
        cost.set_target_cost(target_cost);

        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;

        let best_param = res.state().get_best_param();
        match best_param {
            Some(param) => Ok(*param),
            None => Err(AtlasError::EvaluationErr("No solution found".to_string())),
        }
    }
}

impl<'a> ConstVisit<CrossCurrencySwap> for ParValueConstVisitor<'a> {
    type Output = Result<f64>;
    // visit cross currency swap
    // use BrentRoot solver to find the par rate
    fn visit(&self, instrument: &CrossCurrencySwap) -> Self::Output {
        let first_leg = instrument.first_leg();
        let second_leg = instrument.second_leg();
        let mut nvp_visitor = NPVConstVisitor::new(&self.market_data, true);
        nvp_visitor.set_in_local_currency(true);

        let nvp_first_leg = nvp_visitor.visit(first_leg)?;

        let target_cost = self.target_cost.unwrap_or(0.0) - nvp_first_leg;

        let (min, max) = (-1.0, 1.0);

        let mut cost = ParValue::new_with_local_currency_npv(second_leg, &self.market_data);
        cost.set_target_cost(target_cost);

        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;

        let best_param = res.state().get_best_param();
        match best_param {
            Some(param) => Ok(*param),
            None => Err(AtlasError::EvaluationErr("No solution found".to_string())),
        }
    }
}
#[cfg(test)]
mod tests{
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use crate::{
        cashflows::side::Side,
        core::marketstore::MarketStore,
        currencies::enums::Currency,
        instruments::constructors::{
            makefixedrateinstrument::MakeFixedRateInstrument,
            makefloatingrateinstrument::MakeFloatingRateInstrument,
        },
        models::{simplemodel::SimpleModel, traits::Model},
        rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
            interestrateindex::{iborindex::IborIndex, overnightindex::OvernightIndex},
            traits::HasReferenceDate,
            yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
        },
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        visitors::indexingvisitors::indexingvisitor::IndexingVisitor,
    };
    use crate::{
        instruments::{
            constructors::{
                makefixedrateleg::MakeFixedRateLeg,
                makefloatingrateleg::MakeFloatingRateLeg,
            },
            swaps::vanillairsswap::VanillaIRSSwap,
        },
        visitors::traits::ConstVisit,
    };
    use super::*;
    use crate::visitors::indexingvisitors::fixingvisitor::FixingVisitor;

    pub fn create_store() -> Result<MarketStore> {
        let ref_date = Date::new(2025, 6, 30);
        let local_currency = Currency::USD;
        let mut market_store = MarketStore::new(ref_date, local_currency);

        let forecast_curve_1 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.02,
            RateDefinition::new(
                DayCounter::Thirty360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let forecast_curve_2 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.03,
            RateDefinition::new(
                DayCounter::Thirty360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let discount_curve = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.05,
            RateDefinition::new(
                DayCounter::Thirty360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let mut ibor_fixings = HashMap::new();
        ibor_fixings.insert(Date::new(2021, 9, 1), 0.02); // today
        ibor_fixings.insert(Date::new(2021, 8, 31), 0.02); // yesterday

        let ibor_index = IborIndex::new(forecast_curve_1.reference_date())
            .with_fixings(ibor_fixings)
            .with_term_structure(forecast_curve_1)
            .with_frequency(Frequency::Annual);

        let overnight_fixings =
            make_fixings(ref_date - Period::new(1, TimeUnit::Years), ref_date, 0.06);
        let overnigth_index = OvernightIndex::new(forecast_curve_2.reference_date())
            .with_term_structure(forecast_curve_2)
            .with_fixings(overnight_fixings);

        market_store
            .mut_index_store()
            .add_index(0, Arc::new(RwLock::new(ibor_index)))?;

        market_store
            .mut_index_store()
            .add_index(1, Arc::new(RwLock::new(overnigth_index)))?;

        let discount_index =
            IborIndex::new(discount_curve.reference_date()).with_term_structure(discount_curve);

        market_store
            .mut_index_store()
            .add_index(2, Arc::new(RwLock::new(discount_index)))?;
        return Ok(market_store);
    }

    fn make_fixings(start: Date, end: Date, rate: f64) -> HashMap<Date, f64> {
        let mut fixings = HashMap::new();
        let mut seed = start;
        let mut init = 100.0;
        while seed <= end {
            fixings.insert(seed, init);
            seed = seed + Period::new(1, TimeUnit::Days);
            init = init * (1.0 + rate * 1.0 / 360.0);
        }
        return fixings;
    }

    #[test]
    fn test_par_value_fixed_equal_payment() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let mut instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .equal_payments()
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let par_value = parvaluevisitor.visit(&instrument)?;

        assert!((par_value - 0.05).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_par_value_fixed_bullet() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );
        let mut instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .bullet()
            .build()?;
        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let par_value = parvaluevisitor.visit(&instrument)?;

        assert!((par_value - 0.05).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_par_value_fixed_bullet_negative_rate() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            -0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let mut instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .bullet()
            .build()?;
        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let par_value = parvaluevisitor.visit(&instrument)?;

        assert!((par_value - 0.05).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_par_value_floating_bullet() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let spread = 0.04;

        let mut instrument = MakeFloatingRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate_definition(rate_definition)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(0))
            .with_notional(notional)
            .with_spread(spread)
            .bullet()
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let par_value = parvaluevisitor.visit(&instrument)?;

        assert!((par_value - 0.03).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_par_value_vanilla_irs_swap() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(0.05, rate_definition);
        let notional = 100.0;

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .bullet()
            .build()
            .unwrap();

        let float_leg = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Quarterly)
            .with_spread(0.00)
            .with_rate_definition(rate_definition)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(0))
            .with_notional(notional)
            .bullet()
            .build()
            .unwrap();

        let mut vanillairsswap = VanillaIRSSwap::new(fix_leg, float_leg, Currency::USD)?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut vanillairsswap)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let par_value_visitor = ParValueConstVisitor::new(&data);
        
        let par_rate = par_value_visitor.visit(&vanillairsswap)?;

        assert!(par_rate > -1.0 && par_rate < 1.0);
        println!("Par rate for swap: {}", par_rate);

        Ok(())
    }

    #[test]
    fn test_npv_vanilla_irs_swap() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(0.02999999716676659, rate_definition);
        let notional = 100.0;

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .bullet()
            .build()
            .unwrap();

        let float_leg = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Quarterly)
            .with_spread(0.0)
            .with_rate_definition(rate_definition)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(0))
            .with_notional(notional)
            .bullet()
            .build()
            .unwrap();

        let mut vanillairsswap = VanillaIRSSwap::new(fix_leg, float_leg, Currency::USD)?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut vanillairsswap)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let fixing_visitor = FixingVisitor::new(&data);
        let _ = fixing_visitor.visit(&mut vanillairsswap);

        let npv_visitor = NPVConstVisitor::new(&data, true);
        
        let swap_npv = npv_visitor.visit(&vanillairsswap)?;

        println!("NPV for vanilla IRS swap: {}", swap_npv);
        
        // El NPV debería ser diferente de cero ya que la tasa fija (5%) 
        // probablemente no es la tasa par del mercado
        assert!(swap_npv != 0.0);

        Ok(())
    }


    #[test]
    fn test_par_value_fixed_leg() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(0.05, rate_definition);
        let notional = 100.0;

        let mut fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .bullet()
            .build()
            .unwrap();

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut fix_leg)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let par_value_visitor = ParValueConstVisitor::new(&data);
        
        let par_rate = par_value_visitor.visit(&fix_leg)?;

        assert!(par_rate > -1.0 && par_rate < 1.0);
        println!("Par rate for fixed leg: {}", par_rate);

        Ok(())
    }

    #[test]
    fn test_par_value_floating_leg() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let notional = 100.0;

        let mut float_leg = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Quarterly)
            .with_spread(0.0)
            .with_rate_definition(rate_definition)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)            
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(0))
            .with_notional(notional)
            .bullet()
            .build()
            .unwrap();

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut float_leg)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let par_value_visitor = ParValueConstVisitor::new(&data);
        
        let par_spread = par_value_visitor.visit(&float_leg)?;

        assert!(par_spread > -1.0 && par_spread < 1.0);
        println!("Par spread for floating leg: {}", par_spread);

        Ok(())
    }

}
