use argmin::{
    core::{CostFunction, Error, Executor, State},
    solver::brent::BrentRoot,
};

use crate::{
    cashflows::{cashflow::Cashflow, simplecashflow::SimpleCashflow, traits::Payable},
    core::traits::{HasCurrency, Registrable},
    instruments::loandepos::doublerateinstrument::DoubleRateInstrument,
    utils::errors::Result,
    visitors::traits::{ConstVisit, HasCashflows, Visit},
};

use super::traits::{ParValue, ParValueConstVisitor};

/// # ParValue
/// ParValue is a cost function that calculates the NPV of a generic instrument.
///
/// ## Parameters
/// * `eval` - The instrument to evaluate
/// * `market_data` - The market data to use for evaluation

#[derive(Clone, Debug)]
struct TmpInstrument {
    cashflows: Vec<Cashflow>,
}

impl TmpInstrument {
    pub fn new(cashflows: Vec<Cashflow>) -> Self {
        TmpInstrument { cashflows }
    }
    pub fn set_rate_value(mut self, rate: f64) -> Self {
        self.mut_cashflows().for_each(|cf| match cf {
            Cashflow::FixedRateCoupon(coupon) => coupon.set_rate_value(rate),
            Cashflow::FloatingRateCoupon(coupon) => coupon.set_spread(rate),
            _ => {}
        });
        self
    }
}

impl HasCashflows for TmpInstrument {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.cashflows.iter())
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(self.cashflows.iter_mut())
    }
}

// cost function for TmpInstrument
impl<'a> CostFunction for ParValue<'a, TmpInstrument> {
    type Param = f64;
    type Output = f64;
    fn cost(&self, param: &Self::Param) -> std::result::Result<Self::Output, Error> {
        let mut inst = self.eval.clone().set_rate_value(*param);

        // visit the instrument to update the fixing values
        let _ = self.fixing_visitor.visit(&mut inst);

        // visit the instrument to calculate the npv and return the result
        self.npv_visitor.visit(&inst).map_err(|e| Error::from(e))
    }
}

impl<'a> ConstVisit<DoubleRateInstrument> for ParValueConstVisitor<'a> {
    type Output = Result<(f64, f64)>;
    // visit double rate instrument
    // use BrentRoot solver to find the par rate for the first and second rate
    fn visit(&self, instrument: &DoubleRateInstrument) -> Self::Output {
        let change_rate_date = instrument.change_rate_date();
        let notional_at_change_rate = instrument.notional_at_change_rate().unwrap_or(0.0);
        let currency = instrument.currency()?;
        let side = instrument.side();

        let (mut first_part_cashflows, mut second_part_cashflows): (Vec<Cashflow>, Vec<Cashflow>) =
            instrument
                .cashflows()
                .cloned()
                .partition(|cf| cf.payment_date() <= change_rate_date);
        // buscar id de cashflow con fecha de pago igual a change rate date
        let id = first_part_cashflows
            .iter()
            .filter(|cf| cf.payment_date() == change_rate_date)
            .map(|cf| cf.id())
            .collect::<Result<Vec<usize>>>()?[0];

        let sc = SimpleCashflow::new(change_rate_date, currency, side)
            .with_amount(notional_at_change_rate)
            .with_id(id);
        first_part_cashflows.push(Cashflow::Redemption(sc));

        let sc = SimpleCashflow::new(change_rate_date, currency, side.inverse())
            .with_amount(notional_at_change_rate)
            .with_id(id);
        second_part_cashflows.push(Cashflow::Disbursement(sc));

        let tmp_inst_fp = TmpInstrument::new(first_part_cashflows);
        let tmp_inst_sp = TmpInstrument::new(second_part_cashflows);

        let (min, max) = (-1.0, 1.0);

        let cost = ParValue::new(&tmp_inst_fp, self.market_data);
        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;
        let first_rate_par_value = *res.state().get_best_param().unwrap();

        let cost = ParValue::new(&tmp_inst_sp, self.market_data);
        let solver = BrentRoot::new(min, max, 1e-6);
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(100).target_cost(0.0))
            .run()?;
        let second_rate_par_value = *res.state().get_best_param().unwrap();

        Ok((first_rate_par_value, second_rate_par_value))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cashflows::side::Side,
        core::marketstore::MarketStore,
        currencies::enums::Currency,
        instruments::{
            constructors::makedoublerateinstrument::MakeDoubleRateInstrument, traits::RateType,
        },
        models::{simplemodel::SimpleModel, traits::Model},
        rates::{
            enums::Compounding,
            interestrate::RateDefinition,
            interestrateindex::{iborindex::IborIndex, overnightindex::OvernightIndex},
            traits::HasReferenceDate,
            yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
        },
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        }, visitors::indexingvisitors::indexingvisitor::IndexingVisitor,
    };
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    pub fn create_store() -> Result<MarketStore> {
        let ref_date = Date::new(2021, 9, 1);
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
    fn test_par_value_floating_then_fixed_rate() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();
        let start_date = ref_date.clone();

        let rate_type = RateType::FloatingThenFixed;
        let first_part_rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let first_part_rate = 0.05;
        let second_part_rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let second_part_rate = 0.02;

        let mut instrument = MakeDoubleRateInstrument::new()
            .with_start_date(start_date)
            .with_tenor(Period::new(9, TimeUnit::Years))
            .with_change_rate_tenor(Period::new(4, TimeUnit::Years))
            .with_tenor_grace_period(Period::new(2, TimeUnit::Years))
            .with_rate_type(rate_type)
            .with_first_part_rate_definition(first_part_rate_definition)
            .with_first_part_rate(first_part_rate)
            .with_second_part_rate_definition(second_part_rate_definition)
            .with_second_part_rate(second_part_rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_notional(100.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(1))
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let (first_rate_par_value, second_rate_par_value) = parvaluevisitor.visit(&instrument)?;

        print!(
            "first_rate_par_value: {:?}, second_rate_par_value: {:?}",
            first_rate_par_value, second_rate_par_value
        );

        assert!((first_rate_par_value - 0.02).abs() < 1e-6);
        assert!((second_rate_par_value - 0.05).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_par_value_fixed_then_floating_rate() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();
        let start_date = ref_date.clone();

        let rate_type = RateType::FixedThenFloating;
        let first_part_rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let first_part_rate = 0.05;
        let second_part_rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let second_part_rate = 0.02;

        let mut instrument = MakeDoubleRateInstrument::new()
            .with_start_date(start_date)
            .with_tenor(Period::new(9, TimeUnit::Years))
            .with_change_rate_tenor(Period::new(4, TimeUnit::Years))
            .with_tenor_grace_period(Period::new(2, TimeUnit::Years))
            .with_rate_type(rate_type)
            .with_first_part_rate_definition(first_part_rate_definition)
            .with_first_part_rate(first_part_rate)
            .with_second_part_rate_definition(second_part_rate_definition)
            .with_second_part_rate(second_part_rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_notional(100.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(1))
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let (first_rate_par_value, second_rate_par_value) = parvaluevisitor.visit(&instrument)?;

        print!(
            "first_rate_par_value: {:?}, second_rate_par_value: {:?}",
            first_rate_par_value, second_rate_par_value
        );

        assert!((first_rate_par_value - 0.05).abs() < 1e-6);
        assert!((second_rate_par_value - 0.02).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_par_value_fixed_then_fixed_rate() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();
        let start_date = ref_date.clone();

        let rate_type = RateType::FixedThenFixed;
        let first_part_rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let first_part_rate = 0.05;
        let second_part_rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let second_part_rate = 0.02;

        let mut instrument = MakeDoubleRateInstrument::new()
            .with_start_date(start_date)
            .with_tenor(Period::new(9, TimeUnit::Years))
            .with_change_rate_tenor(Period::new(4, TimeUnit::Years))
            .with_tenor_grace_period(Period::new(2, TimeUnit::Years))
            .with_rate_type(rate_type)
            .with_first_part_rate_definition(first_part_rate_definition)
            .with_first_part_rate(first_part_rate)
            .with_second_part_rate_definition(second_part_rate_definition)
            .with_second_part_rate(second_part_rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_notional(100.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(2))
            .with_forecast_curve_id(Some(1))
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let parvaluevisitor = ParValueConstVisitor::new(&data);
        let (first_rate_par_value, second_rate_par_value) = parvaluevisitor.visit(&instrument)?;

        print!(
            "first_rate_par_value: {:?}, second_rate_par_value: {:?}",
            first_rate_par_value, second_rate_par_value
        );

        assert!((first_rate_par_value - 0.05).abs() < 1e-6);
        assert!((second_rate_par_value - 0.05).abs() < 1e-6);

        Ok(())
    }
}
