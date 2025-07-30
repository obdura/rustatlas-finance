use crate::{
    math::solver::{
        gaussnewton::GaussNewton,
        traits::{Hessian, Jacobian, Residual},
    },
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

use nalgebra::{DMatrix, DVector};
use std::cell::RefCell;

/// # BootstrappingSolver
/// BootstrappingSolver is a solver that optimizes the bootstrapping process.
/// it implements the Residual and Jacobian traits to compute the residuals and Jacobian matrix for the bootstrapping process.
///
/// ## Parameters
/// * `engine` - The bootstrapping engine
/// * `indexer` - The indexing visitor
///
pub struct BootstrappingSolver<'a> {
    engine: &'a RefCell<BootstrappingEngine>,
    indexer: IndexingVisitor,
    optimization_order: Option<Vec<Vec<(usize, usize)>>>,
    max_iterations: usize, // Maximum number of iterations for the optimization
    tolerance: f64,        // Tolerance for convergence
    epsilon: f64,          // Epsilon for numerical differentiation
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
        Ok(BootstrappingSolver {
            engine,
            indexer,
            optimization_order: None,
            max_iterations: 500,
            tolerance: 1e-14,
            epsilon: 1e-12,
        })
    }

    pub fn with_optimization_order(mut self, optimization_order: Vec<Vec<(usize, usize)>>) -> Self {
        self.optimization_order = Some(optimization_order);
        self
    }

    pub fn with_optimization_order_by_curve_id(mut self, curve_ids: Vec<Vec<usize>>) -> Self {
        let binding = self.engine.borrow();
        let order: Vec<Vec<(usize, usize)>> = curve_ids
            .into_iter()
            .map(|group| {
                group
                    .into_iter()
                    .flat_map(|curve_id| binding.estimated_relevant_discount_factors(curve_id))
                    .collect()
            })
            .collect();

        self.optimization_order = Some(order);
        self
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub fn with_epsilon(mut self, epsilon: f64) -> Self {
        self.epsilon = epsilon;
        self
    }

    pub fn configure(&mut self, max_iterations: usize, tolerance: f64, epsilon: f64) -> &mut Self {
        self.max_iterations = max_iterations;
        self.tolerance = tolerance;
        self.epsilon = epsilon;
        self
    }

    pub fn run_optimization(&self) -> Result<()> {
        let init = self.engine.borrow().relevant_discount_factors();
        let init_guess = DVector::from_vec(init.clone());
        let solver = GaussNewton::new(self, init_guess)
            .with_max_iterations(self.max_iterations)
            .with_tolerance(self.tolerance);
        let res = solver.solve()?;
        let solution = res.solution;
        println!("\t\t Optimization completed with n iterations: {}", res.iterations);
        let mut engine = self.engine.borrow_mut();
        engine.update_discount_factors(solution.as_slice())?;
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        let start_time = std::time::Instant::now();
        println!("\t Starting bootstrapping optimization...");

        let optimization_orders = match &self.optimization_order {
            Some(orders) if !orders.is_empty() => orders.clone(),
            _ => {
                println!("\t No optimization orders provided, nothing to run.");
                return Ok(());
            }
        };

        for order in optimization_orders {
            self.engine.borrow_mut().set_optimization_order(order);
            self.run_optimization()?;
        }

        let duration = start_time.elapsed();
        println!("\t Optimization took {:?}", duration);
        Ok(())
    }

    pub fn compute_residuals(&self, discount_factors: &Vec<f64>) -> Result<Vec<f64>> {
        let mut engine = self.engine.borrow_mut();
        engine
            .update_discount_factors(discount_factors)
            .map_err(|e| argmin::core::Error::msg(format!("{:?}", e)))?;

        let relevant_instruments = engine.relevant_instruments();

        let model = BootstrappingModel::new(engine.market_store());
        let data = model.gen_market_data(&self.indexer.request()).unwrap();
        let fixing_visitor = FixingVisitor::new(&data).with_decimals_to_round(16);
        let npv_visitor = NPVConstVisitor::new(&data, true);

        let residuals = engine
            .instruments_mut()
            .iter_mut()
            .enumerate()
            .filter(|(index, _)| relevant_instruments.contains(index))
            .map(|(_, mut instrument)| {
                fixing_visitor.visit(&mut instrument)?;
                let npv = npv_visitor.visit(&instrument)?;
                Ok(npv)
            })
            .collect::<Result<Vec<f64>>>()?;

        Ok(residuals)
    }
}

/// Implementing the Operator trait for BootstrappingSolver
impl<'a> Residual for &BootstrappingSolver<'a> {
    fn residual(&self, x: &nalgebra::DVector<f64>) -> Result<DVector<f64>> {
        let discount_factors: Vec<f64> = x.iter().map(|&v| v).collect();
        let residuals = self.compute_residuals(&discount_factors)?;
        Ok(DVector::from_vec(residuals))
    }
}

/// Implementing the Jacobian trait for BootstrappingSolver
impl<'a> Jacobian for &BootstrappingSolver<'a> {
    fn jacobian(&self, x: &nalgebra::DVector<f64>) -> Result<DMatrix<f64>> {
        let epsilon = self.epsilon;
        let n = x.len();
        let mut jacobian = DMatrix::zeros(n, n);
        for i in 0..n {
            let mut plus = x.clone();
            let mut minus = x.clone();
            plus[i] += epsilon;
            minus[i] -= epsilon;

            let r_plus = self.residual(&plus)?;
            let r_minus = self.residual(&minus)?;

            for j in 0..n {
                jacobian[(j, i)] = (r_plus[j] - r_minus[j]) / (2.0 * epsilon);
            }
        }
        Ok(jacobian)
    }
}

impl<'a> Hessian for &BootstrappingSolver<'a> {
    fn hessian(&self, param: &nalgebra::DVector<f64>) -> Result<DMatrix<f64>> {
        let epsilon = self.epsilon;
        let n = param.len();
        let mut hessian = DMatrix::zeros(n, n);
        for i in 0..n {
            for j in 0..n {
                let mut plus_plus = param.clone();
                let mut plus_minus = param.clone();
                let mut minus_plus = param.clone();
                let mut minus_minus = param.clone();
                plus_plus[i] += epsilon;
                plus_plus[j] += epsilon;
                plus_minus[i] += epsilon;
                minus_plus[j] -= epsilon;
                minus_minus[i] -= epsilon;
                minus_minus[j] -= epsilon;

                let r_pp = self.residual(&plus_plus)?;
                let r_pm = self.residual(&plus_minus)?;
                let r_mp = self.residual(&minus_plus)?;
                let r_mm = self.residual(&minus_minus)?;

                hessian[(i, j)] =
                    (r_pp[i] - r_pm[i] - r_mp[i] + r_mm[i]) / (4.0 * epsilon * epsilon);
            }
        }
        Ok(hessian)
    }
}

#[cfg(test)]
mod tests {
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
    use std::time::Instant;

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

        let optimization_orders = vec![vec![
            (5, 1),
            (5, 2),
            (5, 3),
            (5, 4),
            (5, 5),
            (5, 6),
            (5, 7),
            (5, 8),
            (5, 9),
            (5, 10),
            (5, 11),
        ]];

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization =
            BootstrappingSolver::new(&engine_cell)?.with_optimization_order(optimization_orders);
        bootstrapping_optimization.run()?;

        println!("Bootstrapping optimization completed successfully.");
        println!("Engine state: {}", engine_cell.borrow());

        let duration = start_time.elapsed();
        println!("Optimization took {:?}", duration);

        let binding = engine_cell.borrow();
        let curve = binding
            .market_store()
            .curves_map()
            .get(&5)
            .unwrap()
            .discount_factors()
            .get(9)
            .unwrap();
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
        let bootstrapping_optimization =
            BootstrappingSolver::new(&engine_cell)?.with_optimization_order(optimization_orders);
        bootstrapping_optimization.run()?;

        println!("Bootstrapping optimization completed successfully.");
        println!("Engine state: {}", engine_cell.borrow());

        let duration = start_time.elapsed();
        println!("Optimization took {:?}", duration);

        let binding = engine_cell.borrow();
        let curve = binding
            .market_store()
            .curves_map()
            .get(&5)
            .unwrap()
            .discount_factors()
            .get(9)
            .unwrap();
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

        let optimization_order = vec![vec![(5, 1)]];
        let engine_cell = RefCell::new(engine);
        let solver =
            BootstrappingSolver::new(&engine_cell)?.with_optimization_order(optimization_order);
        solver.run()?;

        let binding = engine_cell.borrow();
        let curve = binding
            .market_store()
            .curves_map()
            .get(&5)
            .unwrap()
            .discount_factors()
            .get(0)
            .unwrap();
        assert!(*curve > 0.0 && *curve <= 1.0);
        Ok(())
    }

    #[test]
    fn test_bootstrapping_solver_run_with_empty_orders_does_nothing() -> Result<()> {
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        engine.market_store_mut().add_curve(5, Currency::USD)?;

        let engine_cell = RefCell::new(engine);
        let optimization_orders: Vec<Vec<(usize, usize)>> = vec![];
        let solver =
            BootstrappingSolver::new(&engine_cell)?.with_optimization_order(optimization_orders);
        let result = solver.run();
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_run_optimization_by_order_with_id() -> Result<()> {
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

        let optimization_orders = vec![vec![5]];

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization = BootstrappingSolver::new(&engine_cell)?
            .with_optimization_order_by_curve_id(optimization_orders);
        bootstrapping_optimization.run()?;

        println!("Bootstrapping optimization completed successfully.");
        println!("Engine state: {}", engine_cell.borrow());

        let duration = start_time.elapsed();
        println!("Optimization took {:?}", duration);

        let binding = engine_cell.borrow();
        let curve = binding
            .market_store()
            .curves_map()
            .get(&5)
            .unwrap()
            .discount_factors()
            .get(9)
            .unwrap();
        assert!((curve - 0.93006371).abs() < 1e-8);

        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_run_optimization_by_order_with_two_ids() -> Result<()> {
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(5, Currency::USD)?;
        bootstrappingmarketstore.add_curve(6, Currency::USD)?;

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
        let swap8 = make_vanillairs_swap(ref_date, tenor, 0.03531375, 6)?;
        engine.add_instrument(6, swap8.accrual_end_date()?, Box::new(swap8))?;

        let tenor = Period::new(4, TimeUnit::Years);
        let swap9 = make_vanillairs_swap(ref_date, tenor, 0.0353595, 6)?;
        engine.add_instrument(6, swap9.accrual_end_date()?, Box::new(swap9))?;

        let optimization_orders = vec![vec![5, 6]];

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization = BootstrappingSolver::new(&engine_cell)?
            .with_optimization_order_by_curve_id(optimization_orders);

        let result = bootstrapping_optimization
            .optimization_order
            .clone()
            .unwrap();
        let expected: Vec<Vec<(usize, usize)>> = vec![vec![
            (5, 1),
            (5, 2),
            (5, 3),
            (5, 4),
            (5, 5),
            (5, 6),
            (5, 7),
            (5, 8),
            (5, 9),
            (6, 1),
            (6, 2),
        ]];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_bootstrapping_engine_run_optimization_by_order_with_two_ids_separate() -> Result<()> {
        let ref_date = Date::new(2025, 7, 18);
        let mut engine = BootstrappingEngine::new(ref_date, Currency::USD);
        let bootstrappingmarketstore = engine.market_store_mut();
        bootstrappingmarketstore.add_curve(5, Currency::USD)?;
        bootstrappingmarketstore.add_curve(6, Currency::USD)?;

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
        let swap8 = make_vanillairs_swap(ref_date, tenor, 0.03531375, 6)?;
        engine.add_instrument(6, swap8.accrual_end_date()?, Box::new(swap8))?;

        let tenor = Period::new(4, TimeUnit::Years);
        let swap9 = make_vanillairs_swap(ref_date, tenor, 0.0353595, 6)?;
        engine.add_instrument(6, swap9.accrual_end_date()?, Box::new(swap9))?;

        let optimization_orders = vec![vec![5], vec![6]];

        let engine_cell = RefCell::new(engine);
        let bootstrapping_optimization = BootstrappingSolver::new(&engine_cell)?
            .with_optimization_order_by_curve_id(optimization_orders);

        let result = bootstrapping_optimization
            .optimization_order
            .clone()
            .unwrap();
        let expected: Vec<Vec<(usize, usize)>> = vec![
            vec![
                (5, 1),
                (5, 2),
                (5, 3),
                (5, 4),
                (5, 5),
                (5, 6),
                (5, 7),
                (5, 8),
                (5, 9),
            ],
            vec![(6, 1), (6, 2)],
        ];

        assert_eq!(result, expected);
        Ok(())
    }
}
