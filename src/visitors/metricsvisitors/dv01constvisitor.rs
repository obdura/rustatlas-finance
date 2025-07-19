use crate::{
    cashflows::{cashflow::Cashflow, traits::{InterestAccrual, Payable}}, core::{meta::MarketData, traits::Registrable}, time::daycounter::DayCounter, utils::errors::{AtlasError, Result}, visitors::traits::{ConstVisit, HasCashflows}
};


/// # DV01ConstVisitor
/// DV01ConstVisitor is a visitor that calculates the DV01 of an instrument.
/// It assumes that the cashflows of the instrument have already been indexed and fixed.
/// It is analytical calculation using de derviative of the NPV with respect to the interest rat and df 
/// The calculos assume no lineality in rates and df 
/// 
/// ## Parameters
/// * `market_data` - The market data to use for NPV calculation
/// * `include_today_cashflows` - Flag to include cashflows with payment date equal to the reference date
pub struct DV01ConstVisitor<'a> {
    market_data: &'a [MarketData],
}

impl<'a> DV01ConstVisitor<'a> {
    pub fn new(market_data: &'a [MarketData]) -> Self {
        DV01ConstVisitor {
            market_data: market_data,
        }
    }
}

impl<'a, T: HasCashflows> ConstVisit<T> for DV01ConstVisitor<'a> {
    type Output = Result<f64>;
    fn visit(&self, visitable: &T) -> Self::Output {
        let npv = visitable.cashflows().try_fold(0.0, |acc, cf| {
            let id = cf.id()?;
            let cf_market_data =
                self.market_data
                    .get(id)
                    .ok_or(AtlasError::NotFoundErr(format!(
                        "Market data for cashflow with id {}",
                        id
                    )))?;

            if cf.payment_date() <= cf_market_data.reference_date()
            {
                return Ok(acc);
            }
                            
            let year_fraction = DayCounter::Actual365.year_fraction(cf_market_data.reference_date(), cf.payment_date());
            let df = cf_market_data.df()?;
            let flag = cf.side().sign();
            let amount = cf.amount()?;

            let mut dv01 = - year_fraction * amount* df *0.0001*flag;

            match cf {
                Cashflow::FloatingRateCoupon(frc) => {
                    let day_counter = frc.rate_definition().day_counter();
                    let delta_year_fraction = if  cf_market_data.reference_date() > frc.accrual_start_date()? {
                        day_counter.year_fraction(cf_market_data.reference_date(), frc.accrual_end_date()?)
                    } else {
                        day_counter.year_fraction(frc.accrual_start_date()?, frc.accrual_end_date()?)
                    };
                    dv01 += frc.notional() * delta_year_fraction * df *0.0001*flag;
                }
                _ => {}
            }


            Ok(acc + dv01)

        });
        return npv;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };


    use crate::{
        cashflows::side::Side, core::marketstore::MarketStore, currencies::enums::Currency, instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument, models::{simplemodel::SimpleModel, traits::Model}, rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
            interestrateindex::{iborindex::IborIndex, overnightindex::OvernightIndex},
            traits::HasReferenceDate,
            yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
        }, time::{
            date::Date, daycounter::DayCounter, enums::{Frequency, TimeUnit}, period::Period
        }, visitors::{indexingvisitors::indexingvisitor::IndexingVisitor, traits::Visit}, 
    };

    use super::*;

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
            RateDefinition::new(
                DayCounter::Thirty360,
                Compounding::Compounded,
                Frequency::Annual,
            )
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
    fn test_dv01_fixed_bullet() -> Result<()> {
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

        let mut instrument = MakeFixedRateInstrument::new()
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

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let dv01 = DV01ConstVisitor::new(&data);
        let dv01 = dv01.visit(&instrument)?;

        println!("DV01: {}", dv01);

        todo!("Implement test");

    }

}

