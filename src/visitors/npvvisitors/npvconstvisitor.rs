use crate::{
    cashflows::traits::Payable,
    core::traits::Registrable,
    models::traits::Model,
    utils::errors::{AtlasError, Result},
    visitors::traits::{ConstVisit, HasCashflows},
};

/// # NPVConstVisitor
/// NPVConstVisitor is a visitor that calculates the NPV of an instrument.
/// It assumes that the cashflows of the instrument have already been fixed.
///
/// ## Parameters
/// * `model` - The model to use for calculation
/// * `include_today_cashflows` - Flag to include cashflows with payment date equal to the reference date
/// * `in_local_currency` - Flag to calculate the NPV in the local currency of the market data or in the currency of the cashflows
pub struct NPVConstVisitor<'a> {
    model: &'a dyn Model,
    include_today_cashflows: bool,
    in_local_currency: bool,
}

impl<'a> NPVConstVisitor<'a> {
    pub fn new(model: &'a dyn Model, include_today_cashflows: bool) -> Self {
        NPVConstVisitor {
            model,
            include_today_cashflows,
            in_local_currency: false,
        }
    }

    pub fn with_include_today_cashflows(mut self, include_today_cashflows: bool) -> Self {
        self.include_today_cashflows = include_today_cashflows;
        self
    }

    pub fn with_in_local_currency(mut self, in_local_currency: bool) -> Self {
        self.in_local_currency = in_local_currency;
        self
    }

    pub fn set_include_today_cashflows(&mut self, include_today_cashflows: bool) {
        self.include_today_cashflows = include_today_cashflows;
    }

    pub fn set_in_local_currency(&mut self, in_local_currency: bool) {
        self.in_local_currency = in_local_currency;
    }

    fn visit_cashflows(&self, visitable: &dyn HasCashflows) -> Result<f64> {
        let mut in_local_currency = self.in_local_currency;
        // First loop: check if cashflows are in local currency
        if !in_local_currency {
            let mut first_currency = None;
            let mut multi_currency = false;
            for cf in visitable.cashflows() {
                let currency = cf.payment_currency()?;
                if let Some(cur) = first_currency {
                    if cur != currency {
                        multi_currency = true;
                        break;
                    }
                } else {
                    first_currency = Some(currency);
                }
            }
            if multi_currency {
                in_local_currency = true;
            }
        }

        let mut npv = 0.0;
        // Second loop: calculate NPV
        for cf in visitable.cashflows() {
            if (self.model.reference_date() == cf.payment_date() && !self.include_today_cashflows)
                || cf.payment_date() < self.model.reference_date()
            {
                continue;
            }

            let df_request = &cf.df_request()?.ok_or(AtlasError::NotFoundErr(format!(
                "Discount factor request not found"
            )))?;
            let df = self.model.gen_df_data(df_request)?;
            let flag = cf.side().sign();

            let fx_fwd_request = &cf.fx_fwd_request()?.ok_or(AtlasError::NotFoundErr(format!(
                "Forward rate request not found"
            )))?;

            let fx_fwd = self.model.gen_fx_data(fx_fwd_request)?;
            let amount = cf.amount()?;

            npv += if in_local_currency {
                let fx_request = &cf.fx_request()?.ok_or(AtlasError::NotFoundErr(format!(
                    "Exchange rate request not found"
                )))?;
                let fx = self.model.gen_fx_data(fx_request)?;
                amount * fx_fwd * df * flag / fx
            } else {
                amount * fx_fwd * df * flag
            };
        }

        Ok(npv)
    }
}

// Implementación genérica
impl<'a, T: HasCashflows> ConstVisit<T> for NPVConstVisitor<'a> {
    type Output = Result<f64>;
    fn visit(&self, visitable: &T) -> Self::Output {
        self.visit_cashflows(visitable)
    }
}

// Implementación para trait object
impl<'a> ConstVisit<&mut Box<dyn HasCashflows>> for NPVConstVisitor<'a> {
    type Output = Result<f64>;
    fn visit(&self, visitable: &&mut Box<dyn HasCashflows>) -> Self::Output {
        self.visit_cashflows(visitable.as_ref())
    }
}

// Implementación para trait object
impl<'a> ConstVisit<&Box<dyn HasCashflows>> for NPVConstVisitor<'a> {
    type Output = Result<f64>;
    fn visit(&self, visitable: &&Box<dyn HasCashflows>) -> Self::Output {
        self.visit_cashflows(visitable.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use rayon::{
        prelude::{IntoParallelIterator, ParallelIterator},
        slice::ParallelSliceMut,
    };

    use crate::{
        cashflows::{side::Side, simplecashflow::SimpleCashflow},
        core::marketstore::MarketStore,
        currencies::enums::Currency,
        instruments::{
            constructors::{
                makefixedrateinstrument::MakeFixedRateInstrument,
                makefloatingrateinstrument::MakeFloatingRateInstrument,
            },
            forwards::fxforward::FxForward,
            loandepos::fixedrateinstrument::FixedRateInstrument,
        },
        models::simplemodel::SimpleModel,
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
        visitors::{
            fixingvisitor::fixingvisitor::FixingVisitor,
            traits::Visit,
        },
    };

    use super::*;

    pub fn create_store() -> Result<MarketStore> {
        let ref_date = Date::new(2021, 9, 1);
        let local_currency = Currency::CLP;
        let mut market_store = MarketStore::new(ref_date, local_currency);

        let forecast_curve_1 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.02,
            RateDefinition::default(),
        ));

        let forecast_curve_2 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.03,
            RateDefinition::default(),
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
            .with_frequency(Frequency::Annual)?;

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

        market_store.mut_exchange_rate_store().add_exchange_rate(
            Currency::CLP,
            Currency::USD,
            950.0,
        );
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
    fn test_model_npv_fixed_bullet() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .build()?;

        let model = SimpleModel::new(&market_store);
        let npv_visitor = NPVConstVisitor::new(&model, true);
        let npv = npv_visitor.visit(&instrument)?;
        assert!(npv.abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_model_npv_fixed_bullet_negative_rate() -> Result<()> {
        let market_store = create_store()?;
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            -0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .build()?;

        let model = SimpleModel::new(&market_store);
        let npv_visitor = NPVConstVisitor::new(&model, true);
        let npv = npv_visitor.visit(&instrument)?;

        assert!(npv.abs() > 70000.0);
        Ok(())
    }

    #[test]
    fn test_model_npv_floating_bullet() -> Result<()> {
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

        let mut instrument = MakeFloatingRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate_definition(rate_definition)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_spread(0.0)
            .bullet()
            .with_discount_curve_id(Some(0))
            .with_forecast_curve_id(Some(0))
            .with_notional(notional)
            .build()?;

        let model = SimpleModel::new(&market_store);

        let fixing_visitor =  FixingVisitor::new(&model);
        fixing_visitor.visit(&mut instrument)?;

        let npv_visitor = NPVConstVisitor::new(&model, true);
        let npv = npv_visitor.visit(&instrument)?;
        assert!(npv.abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_model_npv_fixed_equal_payment() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        let instrument = MakeFixedRateInstrument::new()
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

        let builder = MakeFixedRateInstrument::from(&instrument.clone());
        let instrument_rebuilt = builder.build()?;

        let model = SimpleModel::new(&market_store);
        let npv_visitor = NPVConstVisitor::new(&model, true);
        let npv = npv_visitor.visit(&instrument)?;
        let npv_rebuilt = npv_visitor.visit(&instrument_rebuilt)?;

        assert!(npv.abs() < 1e-6);
        assert!(npv_rebuilt.abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn generator_tests() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();

        let start_date = ref_date;
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let notional = 100_000.0;
        let rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Thirty360,
        );

        // par build
        let mut instruments: Vec<FixedRateInstrument> = (0..150000)
            .into_par_iter() // Create a parallel iterator
            .map(|_| {
                MakeFixedRateInstrument::new()
                    .with_start_date(start_date.clone()) // clone data if needed
                    .with_end_date(end_date.clone()) // clone data if needed
                    .with_rate(rate)
                    .with_payment_frequency(Frequency::Semiannual)
                    .with_side(Side::Receive)
                    .with_currency(Currency::USD)
                    .bullet()
                    .with_discount_curve_id(Some(2))
                    .with_notional(notional)
                    .build()
                    .unwrap()
            })
            .collect(); // Collect the results into a Vec<_>

        fn npv(instruments: &mut [FixedRateInstrument]) -> f64 {
            let store = create_store().unwrap();
            let mut npv = 0.0;

            let model = SimpleModel::new(&store);

            let npv_visitor = NPVConstVisitor::new(&model, true);
            instruments
                .iter()
                .for_each(|inst| npv += npv_visitor.visit(inst).unwrap());
            npv
        }

        instruments.par_rchunks_mut(1000).for_each(|chunk| {
            npv(chunk);
        });

        Ok(())
    }

    #[test]
    fn test_model_npv_visitor_forward() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();
        let pay_date = ref_date + Period::new(360, TimeUnit::Days);

        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;

        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(100.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(100.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?
            .with_receive_discount_curve_id(0)
            .with_pay_discount_curve_id(1);


        let model = SimpleModel::new(&market_store);

        let npv_visitor = NPVConstVisitor::new(&model, true);
        let npv = npv_visitor.visit(&fx_forward)?;
        assert!((npv - (100.0 / (1.02) - 100.0 / (1.03) * 950.0)).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_model_npv_visitor_forward_by_side() -> Result<()> {
        let market_store = create_store().unwrap();
        let ref_date = market_store.reference_date();
        let pay_date = ref_date + Period::new(360, TimeUnit::Days);

        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;

        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(100.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(100.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?
            .with_receive_discount_curve_id(0)
            .with_pay_discount_curve_id(1);

        let model = SimpleModel::new(&market_store);

        let npv_visitor = NPVConstVisitor::new(&model, true);
        let mtm_receive_side = npv_visitor.visit(fx_forward.receive_cashflows())?;
        assert!((mtm_receive_side - 100.0 / 1.02).abs() < 1e-6);
        println!("{}", mtm_receive_side);

        let mtm_pay_side = npv_visitor.visit(fx_forward.pay_cashflows())?;
        assert!((mtm_pay_side + 100.0 / 1.03).abs() < 1e-6);
        println!("{}", mtm_pay_side);

        let npv_visitor = NPVConstVisitor::new(&model, true).with_in_local_currency(true);
        let mtm_receive_side = npv_visitor.visit(fx_forward.receive_cashflows())?;
        assert!((mtm_receive_side - 100.0 / 1.02).abs() < 1e-6);

        let mtm_pay_side = npv_visitor.visit(fx_forward.pay_cashflows())?;
        assert!((mtm_pay_side + 950.0 * 100.0 / 1.03).abs() < 1e-6);

        Ok(())
    }
}
