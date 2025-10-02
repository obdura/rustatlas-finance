use crate::{
    cashflows::traits::Payable,
    core::traits::Registrable,
    models::traits::Model,
    time::date::Date,
    utils::errors::{AtlasError, Result},
    visitors::traits::{ConstVisit, HasCashflows},
};
use std::collections::BTreeMap;

/// # NPVByDateConstVisitor
/// NPVByDateConstVisitor is a visitor that calculates the NPV of an instrument and returns the result in a BTreeMap
/// where the key is the payment date of the cashflow and the value is the NPV of the cashflow.
/// It assumes that the cashflows of the instrument have already been fixed.
pub struct NPVByDateConstVisitor<'a> {
    model: &'a dyn Model,
    include_today_cashflows: bool,
    reference_date: Date,
    in_local_currency: bool,
}

impl<'a> NPVByDateConstVisitor<'a> {
    pub fn new(reference_date: Date, model: &'a dyn Model, include_today_cashflows: bool) -> Self {
        NPVByDateConstVisitor {
            reference_date,
            model,
            include_today_cashflows,
            in_local_currency: false,
        }
    }
    pub fn set_include_today_cashflows(&mut self, include_today_cashflows: bool) {
        self.include_today_cashflows = include_today_cashflows;
    }
    pub fn set_in_local_currency(&mut self, in_local_currency: bool) {
        self.in_local_currency = in_local_currency;
    }
}

impl<'a, T: HasCashflows> ConstVisit<T> for NPVByDateConstVisitor<'a> {
    type Output = Result<BTreeMap<Date, f64>>;
    fn visit(&self, visitable: &T) -> Self::Output {
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
        
        let mut npv_result = BTreeMap::new();
        npv_result.insert(self.reference_date, 0.0);
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

            let npv = if in_local_currency {
                let fx_request = &cf.fx_request()?.ok_or(AtlasError::NotFoundErr(format!(
                    "Exchange rate request not found"
                )))?;
                let fx = self.model.gen_fx_data(fx_request)?;
                amount * fx_fwd * df * flag / fx
            } else {
                amount * fx_fwd * df * flag
            };

            let acc = npv_result.entry(cf.payment_date()).or_insert(0.0);
            *acc += npv
        }
        Ok(npv_result)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use super::*;
    use crate::{
        cashflows::side::Side,
        core::marketstore::MarketStore,
        currencies::enums::Currency,
        instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument,
        models::simplemodel::SimpleModel,
        rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
            interestrateindex::{iborindex::IborIndex, overnightindex::OvernightIndex},
            traits::HasReferenceDate,
            yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
        },
        time::{
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
    };

    pub fn create_store() -> Result<MarketStore> {
        let ref_date = Date::new(2021, 9, 1);
        let local_currency = Currency::USD;
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
            RateDefinition::default(),
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
    fn test_model_npv_by_date_const_visitor_expired_instrument() -> Result<()> {
        let market_store = create_store().unwrap();

        let start_date = Date::new(2010, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);

        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let instrument_1 = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(100.0)
            .with_discount_curve_id(Some(0))
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        let instrument_2 = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Monthly)
            .with_rate(rate)
            .with_notional(100.0)
            .with_discount_curve_id(Some(0))
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        let model = SimpleModel::new(&market_store);

        let npv_visitor =
            NPVByDateConstVisitor::new(market_store.reference_date(), &model, false);
        let npv_result_inst_1 = npv_visitor.visit(&instrument_1)?;
        let npv_result_inst_2 = npv_visitor.visit(&instrument_2)?;

        assert_eq!(npv_result_inst_1.len(), 1);
        assert_eq!(npv_result_inst_2.len(), 1);

        Ok(())
    }

    #[test]
    fn test_npv_by_date_const_visitor() -> Result<()> {
        let market_store = create_store().unwrap();

        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);

        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let instrument_1 = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(100.0)
            .with_discount_curve_id(Some(0))
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;

        let instrument_2 = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Monthly)
            .with_rate(rate)
            .with_notional(100.0)
            .with_discount_curve_id(Some(0))
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()?;


        let model = SimpleModel::new(&market_store);

        let npv_visitor = NPVByDateConstVisitor::new(market_store.reference_date(), &model, false);
        let npv_result_inst_1 = npv_visitor.visit(&instrument_1)?;
        let npv_result_inst_2 = npv_visitor.visit(&instrument_2)?;

        assert_eq!(npv_result_inst_1.len(), 8);
        assert_eq!(npv_result_inst_2.len(), 41);

        Ok(())
    }
}
