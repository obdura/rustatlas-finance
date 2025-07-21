use crate::{
    models::traits::Model,
    rates::bootstrapping::{
        bootstrappingengine::BootstrappingEngine, bootstrappingmarketstore::BootstrappingModel,
    },
    utils::errors::Result,
    visitors::{
        indexingvisitors::{fixingvisitor::FixingVisitor, indexingvisitor::IndexingVisitor},
        npvvisitors::npvconstvisitor::NPVConstVisitor,
        traits::{ConstVisit, Visit},
    },
};
use argmin::{
    core::{CostFunction, Executor, State},
    solver::neldermead::NelderMead,
};
use std::cell::RefCell;

struct BootstrappingSolver<'a> {
    engine: &'a RefCell<BootstrappingEngine>,
    indexer: IndexingVisitor,
}

impl<'a> BootstrappingSolver<'a> {
    pub fn new(engine: &'a RefCell<BootstrappingEngine>) -> Result<Self> {
        let indexer = IndexingVisitor::new();
        engine
            .borrow_mut()
            .instruments_mut()
            .iter_mut()
            .try_for_each(|mut instrument| -> Result<()> {
                indexer.visit(&mut instrument)?;
                Ok(())
            })?;
        Ok(BootstrappingSolver { engine, indexer })
    }

    pub fn run(&self) -> Result<()> {
        let n = self.engine.borrow().number_of_instruments();
        let init = vec![1.0; n];
        // Construcción del simplex inicial (n+1 vértices)
        let mut simplex = Vec::with_capacity(n + 1);
        simplex.push(init.clone());
        for i in 0..n {
            let mut v = init.clone();
            v[i] -= 0.05;
            simplex.push(v);
        }

        let solver = NelderMead::new(simplex);
        let res = Executor::new(self, solver)
            .configure(|state| state.param(init).max_iters(100000).target_cost(1e-16))
            .run()?;
        
        println!(
            "Bootstrapping optimization completed in {} iterations.",
            res.state().get_iter()
        );
        let solution = res.state().get_param().unwrap();
        let mut engine = self.engine.borrow_mut();
        engine.update_discount_factors(solution.clone())?;
        Ok(())
    }
}

impl<'a> CostFunction for &BootstrappingSolver<'a> {
    type Param = Vec<f64>;
    type Output = f64;

    fn cost(
        &self,
        discount_factors: &Self::Param,
    ) -> std::result::Result<f64, argmin::core::Error> {
        let mut engine = self.engine.borrow_mut();
        engine
            .update_discount_factors(discount_factors.clone())
            .map_err(|e| argmin::core::Error::msg(format!("{:?}", e)))?;

        // Calcula los NPVs de todos los instrumentos
        let model = BootstrappingModel::new(engine.market_store());
        let data = model.gen_market_data(&self.indexer.request()).unwrap();
        let fixing_visitor = FixingVisitor::new(&data).with_decimals_to_round(15);
        let npv_visitor = NPVConstVisitor::new(&data, true);

        let sum_sq_npv = engine.instruments_mut().iter_mut().try_fold(
            0.0,
            |acc, mut instrument| -> Result<f64> {
                fixing_visitor.visit(&mut instrument)?;
                let npv = npv_visitor.visit(&instrument)?;
                Ok(acc + npv.abs())
            },
        )?;
        Ok(sum_sq_npv)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    
    use crate::{
        cashflows::side::Side,
        currencies::enums::Currency,
        instruments::{
            constructors::{makefixedrateinstrument::MakeFixedRateInstrument, makefixedrateleg::MakeFixedRateLeg},
            loandepos::fixedrateinstrument::FixedRateInstrument, swaps::vanillairsswap::VanillaIRSSwap,
        },
        rates::{
            bootstrapping::{
                bootstrappingengine::BootstrappingEngine, bootstrappingsolver::BootstrappingSolver,
            }, enums::Compounding, interestrate::{InterestRate, RateDefinition}
        },
        time::{date::Date, daycounter::DayCounter, enums::{Frequency, TimeUnit}, period::Period},
        utils::errors::Result,
    };
    use crate::instruments::constructors::makefloatingrateleg::MakeFloatingRateLeg;

    fn make_fixed_zero_instruments(
        ref_date: Date,
        end_date: Date,
        rate: f64,
        curve_id: usize,
    ) -> Result<FixedRateInstrument> {
        let rate_definition = RateDefinition::default();
        let rate = InterestRate::from_rate_definition(rate, rate_definition);
        let inst = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(curve_id)) 
            .zero()
            .build()?;
        Ok(inst)
    }

    fn make_fixed_bullet_instruments(
        ref_date: Date,
        end_date: Date,
        rate: f64,
        curve_id: usize,
    ) -> Result<FixedRateInstrument> {
        let rate_definition = RateDefinition::default();
        let rate = InterestRate::from_rate_definition(rate, rate_definition);
        let inst = MakeFixedRateInstrument::new()
            .with_currency(Currency::USD)
            .with_side(Side::Receive)
            .with_start_date(ref_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Annual)
            .with_notional(1_000_000.0)
            .with_discount_curve_id(Some(curve_id)) 
            .bullet()
            .build()?;
        Ok(inst)
    }

    fn make_vanillairs_swap(
        ref_date: Date,
        end_date: Date,
        fixed_rate: f64,
        curve_id: usize,
    ) -> Result<VanillaIRSSwap> {
        let start_date = ref_date;
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(fixed_rate, rate_definition);
        let notional = 1_000_000.0;

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(curve_id))
            .with_payment_frequency(Frequency::Annual)
            .bullet()
            .build()
            .unwrap();

        let float_leg = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Annual)
            .with_spread(0.0)
            .with_rate_definition(rate_definition)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .with_notional(notional)
            .with_discount_curve_id(Some(curve_id))
            .with_forecast_curve_id(Some(curve_id))
            .bullet()
            .build()
            .unwrap();

        let vanillairsswap = VanillaIRSSwap::new(fix_leg, float_leg, Currency::USD)?;
        Ok(vanillairsswap)
    }

    #[test]
    fn test_bootstrapping_engine_run_optimization() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(1, Currency::USD)?;
        bootstrappingmarketstore.add_curve(3, Currency::USD)?;
        bootstrappingmarketstore.add_curve(5, Currency::USD)?;

        let end_date = ref_date + Period::new(1, TimeUnit::Years);
        let inst1 = make_fixed_zero_instruments(ref_date, end_date, 0.05, 1)?;
        engine.add_instrument(1, end_date, Box::new(inst1))?;

        let inst2 = make_fixed_zero_instruments(ref_date, end_date, 0.05, 3)?;
        engine.add_instrument(3, end_date, Box::new(inst2))?;

        let end_date = ref_date + Period::new(2, TimeUnit::Years);
        let inst3 = make_fixed_bullet_instruments(ref_date, end_date, 0.08, 3)?;
        engine.add_instrument(3, end_date, Box::new(inst3))?;

        let end_date = ref_date + Period::new(1, TimeUnit::Years);
        let swap1 = make_vanillairs_swap(ref_date, end_date, 0.07, 5)?;
        print!("Swap1: {}\n", swap1);
        engine.add_instrument(5, end_date, Box::new(swap1))?;

        let end_date = ref_date + Period::new(2, TimeUnit::Years);
        let swap2 = make_vanillairs_swap(ref_date, end_date, 0.06, 5)?;
        engine.add_instrument(5, end_date, Box::new(swap2))?;

        let end_date = ref_date + Period::new(3, TimeUnit::Years);
        let swap3 = make_vanillairs_swap(ref_date, end_date, 0.06, 5)?;
        engine.add_instrument(5, end_date, Box::new(swap3))?;

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization = BootstrappingSolver::new(&engine_cell)?;
        bootstrapping_optimization.run()?;

        println!("Bootstrapping optimization completed successfully.");
        println!("Engine state: {}", engine_cell.borrow());

        Ok(())
    }
}
