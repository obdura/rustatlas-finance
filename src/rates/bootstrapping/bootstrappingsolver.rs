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

/// # BootstrappingSolver
/// BootstrappingSolver is a solver that optimizes the bootstrapping process.
/// It uses the Nelder-Mead algorithm to find the optimal bootstrapping parameters.
/// 
/// ## Parameters
/// * `engine` - The bootstrapping engine
/// * `indexer` - The indexing visitor
///
pub struct BootstrappingSolver<'a> {
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

    pub fn run_optimization(&self) -> Result<()> {
        let n = self.engine.borrow().optimization_order_len();
        let init = self.engine.borrow().relevant_discount_factors();
        let mut simplex = Vec::with_capacity(n + 1);
        simplex.push(init.clone());
        for i in 0..n {
            let mut v = init.clone();
            v[i] -= 0.5 / n as f64;
            simplex.push(v);
        }

        let solver = NelderMead::new(simplex);
        let res = Executor::new(self, solver)
            .configure(|state| state.param(init).max_iters(1_000_000).target_cost(1e-15))
            .run()?;

        println!("Bootstrapping optimization completed in {} iterations with a final cost of {:.2e}.", res.state().get_iter(), res.state().get_best_cost());
        let solution = res.state().get_param().unwrap();
        let mut engine = self.engine.borrow_mut();
        engine.update_discount_factors(solution)?;

        Ok(())
    }

    pub fn run(&self, optimization_orders: Vec<Vec<(usize, usize)>>) -> Result<()> {
        let start_time = std::time::Instant::now();

        for order in optimization_orders {
            self.engine.borrow_mut().set_optimization_order(order);
            self.run_optimization()?;
        }

        let duration = start_time.elapsed();
        println!("Optimization took {:?}", duration);

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
            .update_discount_factors(discount_factors)
            .map_err(|e| argmin::core::Error::msg(format!("{:?}", e)))?;

        let relevant_instruments = engine.relevant_instruments();

        let model = BootstrappingModel::new(engine.market_store());
        let data = model.gen_market_data(&self.indexer.request()).unwrap();
        let fixing_visitor = FixingVisitor::new(&data).with_decimals_to_round(16);
        let npv_visitor = NPVConstVisitor::new(&data, true);

        let sum_sq_npv = engine.instruments_mut().iter_mut().enumerate().try_fold(
            0.0,
            |acc, (index, mut instrument)| -> Result<f64> {
                if relevant_instruments.contains(&index) {
                    fixing_visitor.visit(&mut instrument)?;
                    let npv = npv_visitor.visit(&instrument)?;
                    Ok(acc + npv * npv)
                } else {
                    Ok(acc)
                }
            },
        )?;
        Ok(sum_sq_npv)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::{
        cashflows::{side::Side, traits::InterestAccrual},
        currencies::enums::Currency,
        instruments::{
            constructors::{
                makefixedrateinstrument::MakeFixedRateInstrument,
                makefixedrateleg::MakeFixedRateLeg, makefloatingrateleg::MakeFloatingRateLeg,
            },
            loandepos::fixedrateinstrument::FixedRateInstrument,
            swaps::vanillairsswap::VanillaIRSSwap,
            traits::Structure,
        },
        rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
        },
        time::{
            calendar::Calendar,
            calendars::unitedstates::UnitedStates,
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
    };

    use super::*;

    fn make_fixed_instruments(
        ref_date: Date,
        end_date: Date,
        rate: f64,
        structure: Structure,
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
            .with_payment_frequency(Frequency::Annual)
            .with_structure(structure)
            .build()?;
        Ok(inst)
    }

    fn make_vanillairs_swap(
        ref_date: Date,
        tenor: Period,
        fixed_rate: f64,
        curve_id: usize,
    ) -> Result<VanillaIRSSwap> {
        let start_date = ref_date;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(fixed_rate, rate_definition);
        let notional = 1_000_000.0;

        let calendar = Calendar::UnitedStates(UnitedStates::default());
        let fix_leg = MakeFixedRateLeg::new()
            .with_calendar(Some(calendar))
            .with_settlement_period(Period::new(2, TimeUnit::Days))
            .with_negotiation_date(start_date)
            .with_tenor(tenor)
            .with_notional(notional)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(curve_id))
            .with_payment_frequency(Frequency::Annual)
            .bullet()
            .build()
            .unwrap();

        let calendar = Calendar::UnitedStates(UnitedStates::default());
        let float_leg = MakeFloatingRateLeg::new()
            .with_calendar(Some(calendar))
            .with_settlement_period(Period::new(2, TimeUnit::Days))
            .with_negotiation_date(start_date)
            .with_tenor(tenor)
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
        let start_time = Instant::now();
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(5, Currency::USD)?;

        let end_date = ref_date + Period::new(3, TimeUnit::Days);
        let inst1 = make_fixed_instruments(ref_date, end_date, 0.0434, Structure::Zero, 5)?;
        engine.add_instrument(5, end_date, Box::new(inst1))?;

        let end_date = ref_date + Period::new(4, TimeUnit::Days);
        let inst2 =
            make_fixed_instruments(ref_date, end_date, 0.0434039240833228, Structure::Zero, 5)?;
        engine.add_instrument(5, end_date, Box::new(inst2))?;

        let tenor = Period::new(1, TimeUnit::Weeks);
        let swap1 = make_vanillairs_swap(ref_date, tenor, 0.0432544, 5)?;
        engine.add_instrument(5, swap1.accrual_end_date()?, Box::new(swap1))?;

        let tenor = Period::new(2, TimeUnit::Weeks);
        let swap2 = make_vanillairs_swap(ref_date, tenor, 0.04335, 5)?;
        engine.add_instrument(5, swap2.accrual_end_date()?, Box::new(swap2))?;

        let tenor = Period::new(1, TimeUnit::Months);
        let swap3 = make_vanillairs_swap(ref_date, tenor, 0.0434345, 5)?;
        engine.add_instrument(5, swap3.accrual_end_date()?, Box::new(swap3))?;

        let tenor = Period::new(2, TimeUnit::Months);
        let swap4 = make_vanillairs_swap(ref_date, tenor, 0.043481, 5)?;
        engine.add_instrument(5, swap4.accrual_end_date()?, Box::new(swap4))?;

        let tenor = Period::new(3, TimeUnit::Months);
        let swap5 = make_vanillairs_swap(ref_date, tenor, 0.043192, 5)?;
        engine.add_instrument(5, swap5.accrual_end_date()?, Box::new(swap5))?;

        let tenor = Period::new(1, TimeUnit::Years);
        let swap6 = make_vanillairs_swap(ref_date, tenor, 0.039811, 5)?;
        engine.add_instrument(5, swap6.accrual_end_date()?, Box::new(swap6))?;

        let tenor = Period::new(2, TimeUnit::Years);
        let swap7 = make_vanillairs_swap(ref_date, tenor, 0.0362295, 5)?;
        engine.add_instrument(5, swap7.accrual_end_date()?, Box::new(swap7))?;

        let tenor = Period::new(3, TimeUnit::Years);
        let swap8 = make_vanillairs_swap(ref_date, tenor, 0.03531375, 5)?;
        engine.add_instrument(5, swap8.accrual_end_date()?, Box::new(swap8))?;

        let tenor = Period::new(4, TimeUnit::Years);
        let swap9 = make_vanillairs_swap(ref_date, tenor, 0.0353595, 5)?;
        engine.add_instrument(5, swap9.accrual_end_date()?, Box::new(swap9))?;

        let optimization_orders = vec![
            vec![(5, 1) , (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (5, 7), (5, 8), (5, 9), (5, 10), (5, 11)],
        ];

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization = BootstrappingSolver::new(&engine_cell)?;
        bootstrapping_optimization.run(optimization_orders)?;

        println!("Bootstrapping optimization completed successfully.");
        println!("Engine state: {}", engine_cell.borrow());


        let duration = start_time.elapsed();
        println!("Optimization took {:?}", duration);


        let binding = engine_cell.borrow();
        let curve = binding.market_store().curves_map().get(&5).unwrap().discount_factors().get(9).unwrap();
        assert!((curve - 0.93006371).abs() < 1e-8);
 
        Ok(())
    }


    #[test]
    fn test_bootstrapping_engine_run_optimization_by_order() -> Result<()> {
        let start_time = Instant::now();
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(5, Currency::USD)?;

        let end_date = ref_date + Period::new(3, TimeUnit::Days);
        let inst1 = make_fixed_instruments(ref_date, end_date, 0.0434, Structure::Zero, 5)?;
        engine.add_instrument(5, end_date, Box::new(inst1))?;

        let end_date = ref_date + Period::new(4, TimeUnit::Days);
        let inst2 =
            make_fixed_instruments(ref_date, end_date, 0.0434039240833228, Structure::Zero, 5)?;
        engine.add_instrument(5, end_date, Box::new(inst2))?;

        let tenor = Period::new(1, TimeUnit::Weeks);
        let swap1 = make_vanillairs_swap(ref_date, tenor, 0.0432544, 5)?;
        engine.add_instrument(5, swap1.accrual_end_date()?, Box::new(swap1))?;

        let tenor = Period::new(2, TimeUnit::Weeks);
        let swap2 = make_vanillairs_swap(ref_date, tenor, 0.04335, 5)?;
        engine.add_instrument(5, swap2.accrual_end_date()?, Box::new(swap2))?;

        let tenor = Period::new(1, TimeUnit::Months);
        let swap3 = make_vanillairs_swap(ref_date, tenor, 0.0434345, 5)?;
        engine.add_instrument(5, swap3.accrual_end_date()?, Box::new(swap3))?;

        let tenor = Period::new(2, TimeUnit::Months);
        let swap4 = make_vanillairs_swap(ref_date, tenor, 0.043481, 5)?;
        engine.add_instrument(5, swap4.accrual_end_date()?, Box::new(swap4))?;

        let tenor = Period::new(3, TimeUnit::Months);
        let swap5 = make_vanillairs_swap(ref_date, tenor, 0.043192, 5)?;
        engine.add_instrument(5, swap5.accrual_end_date()?, Box::new(swap5))?;

        let tenor = Period::new(1, TimeUnit::Years);
        let swap6 = make_vanillairs_swap(ref_date, tenor, 0.039811, 5)?;
        engine.add_instrument(5, swap6.accrual_end_date()?, Box::new(swap6))?;

        let tenor = Period::new(2, TimeUnit::Years);
        let swap7 = make_vanillairs_swap(ref_date, tenor, 0.0362295, 5)?;
        engine.add_instrument(5, swap7.accrual_end_date()?, Box::new(swap7))?;

        let tenor = Period::new(3, TimeUnit::Years);
        let swap8 = make_vanillairs_swap(ref_date, tenor, 0.03531375, 5)?;
        engine.add_instrument(5, swap8.accrual_end_date()?, Box::new(swap8))?;

        let tenor = Period::new(4, TimeUnit::Years);
        let swap9 = make_vanillairs_swap(ref_date, tenor, 0.0353595, 5)?;
        engine.add_instrument(5, swap9.accrual_end_date()?, Box::new(swap9))?;

        let optimization_orders = vec![
            vec![(5, 1)],
            vec![(5, 2)],
            vec![(5, 3)],
            vec![(5, 4)],
            vec![(5, 5)],
            vec![(5, 6)],
            vec![(5, 7)],
            vec![(5, 8)],
            vec![(5, 9)],
            vec![(5, 10)],
            vec![(5, 11)],
        ];

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization = BootstrappingSolver::new(&engine_cell)?;
        bootstrapping_optimization.run(optimization_orders)?;

        println!("Bootstrapping optimization completed successfully.");
        println!("Engine state: {}", engine_cell.borrow());

        let duration = start_time.elapsed();
        println!("Optimization took {:?}", duration);

        let binding = engine_cell.borrow();
        let curve = binding.market_store().curves_map().get(&5).unwrap().discount_factors().get(9).unwrap();
        assert!((curve - 0.93006371).abs() < 1e-8);
 

        Ok(())
    }


    #[test]
    fn test_bootstrapping_solver_run_optimization_updates_discount_factors() -> Result<()> {
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(5, Currency::USD)?;

        let end_date = ref_date + Period::new(3, TimeUnit::Days);
        let inst1 = make_fixed_instruments(ref_date, end_date, 0.0434, Structure::Zero, 5)?;
        engine.add_instrument(5, end_date, Box::new(inst1))?;

        let optimization_orders = vec![vec![(5, 1)]];
        let engine_cell = RefCell::new(engine);
        let solver = BootstrappingSolver::new(&engine_cell)?;
        solver.run(optimization_orders)?;

        let binding = engine_cell.borrow();
        let curve = binding.market_store().curves_map().get(&5).unwrap().discount_factors().get(0).unwrap();
        assert!(*curve > 0.0 && *curve <= 1.0);
        Ok(())
    }

    #[test]
    fn test_bootstrapping_solver_run_with_empty_orders_does_nothing() -> Result<()> {
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(5, Currency::USD)?;

        let engine_cell = RefCell::new(engine);
        let solver = BootstrappingSolver::new(&engine_cell)?;
        let optimization_orders: Vec<Vec<(usize, usize)>> = vec![];
        let result = solver.run(optimization_orders);
        assert!(result.is_ok());
        Ok(())
    }
}
