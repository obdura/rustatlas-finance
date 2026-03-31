use crate::{
    cashflows::{cashflow::Cashflow, traits::Payable},
    core::traits::Registrable,
    math::solver::{brentopt::BrentOpt, traits::CostFunction},
    models::traits::Model,
    rates::interestrate::{InterestRate, RateDefinition},
    utils::errors::{AtlasError, Result},
    visitors::traits::{ConstVisit, HasCashflows},
};

/// # ZSpreadConstVisitor
/// ZSpreadConstVisitor is a visitor that calculates the ZSpread of a generic instrument.
///
/// ## Parameters
/// * `market_data` - The market data to use for evaluation
/// * `rate_definition` - The rate definition to use for the given spread
/// * `target` - The target npv to match the spread calculation
pub struct ZSpreadConstVisitor<'a> {
    model: &'a dyn Model,
    rate_definition: RateDefinition,
    target: f64,
}

impl<'a> ZSpreadConstVisitor<'a> {
    pub fn new(
        model: &'a dyn Model,
        rate_definition: RateDefinition,
        target: f64,
    ) -> Self {
        ZSpreadConstVisitor {
            model,
            rate_definition,
            target,
        }
    }
}

struct SpreadedNPV<'a, T> {
    eval: &'a T,
    model: &'a dyn Model,
    rate_definition: RateDefinition,
    target: f64,
}

impl<'a, T> SpreadedNPV<'a, T>
where
    T: HasCashflows,
{
    fn cashflow_npv(&self, cf: &Cashflow, spread: f64) -> Result<f64> {
        let t = self
            .rate_definition
            .day_counter()
            .year_fraction(self.model.reference_date(), cf.payment_date());

        if t < 0.0 {
            return Ok(0.0);
        }

        let df_request = &cf.df_request()?.ok_or(AtlasError::NotFoundErr(format!(
                "Discount factor request not found"
            )))?;
        let df = self.model.gen_df_data(df_request)?;
        let implied_df_rate = self.rate_definition.implied_rate(1.0 / df, t)?;

        let composite_rate = InterestRate::new(
            implied_df_rate.rate() + spread,
            self.rate_definition.compounding(),
            self.rate_definition.frequency(),
            self.rate_definition.day_counter(),
        );
        let final_df = 1.0 / composite_rate.compound_factor_from_yf(t);
        let flag = cf.side().sign();
        let cf_npv = cf.amount()? * final_df * flag;
        Ok(cf_npv)
    }
}

impl<'a, T> CostFunction for SpreadedNPV<'a, T>
where
    T: HasCashflows,
{
    fn cost(&self, param: &f64) -> Result<f64> {
        let npv = self
            .eval
            .cashflows()
            .try_fold(0.0, |acc, cf| -> Result<f64> {
                match cf {
                    Cashflow::Disbursement(_) => return Ok(acc),
                    _ => {
                        let cf_npv = self.cashflow_npv(cf, *param)?;
                        Ok(acc + cf_npv)
                    }
                }
            })?;
        Ok((npv - self.target).abs())
    }
}

impl<'a, T> ConstVisit<T> for ZSpreadConstVisitor<'a>
where
    T: HasCashflows,
{
    type Output = Result<f64>;
    fn visit(&self, visitable: &T) -> Self::Output {
        let npv = SpreadedNPV {
            eval: visitable,
            model: self.model,
            rate_definition: self.rate_definition,
            target: self.target,
        };
        let solver = BrentOpt::new(npv, -1.0, 1.0);
        let res = solver.solve()?;

        Ok(res.argmin)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, RwLock};

    use crate::{
        cashflows::side::Side,
        core::marketstore::MarketStore,
        currencies::enums::Currency,
        instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument,
        models::simplemodel::SimpleModel,
        rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
            interestrateindex::iborindex::IborIndex,
            traits::HasReferenceDate,
            yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
        },
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
        visitors::
            traits::ConstVisit
        ,
    };

    use super::ZSpreadConstVisitor;

    #[allow(dead_code)]
    pub fn create_store() -> Result<MarketStore> {
        let ref_date = Date::new(2021, 9, 1);
        let local_currency = Currency::USD;
        let mut market_store = MarketStore::new(ref_date, local_currency);

        let discount_curve = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.05,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Continuous,
                Frequency::Semiannual,
            ),
        ));

        let ibor_index = IborIndex::new(discount_curve.reference_date())
            .with_term_structure(discount_curve)
            .with_frequency(Frequency::Semiannual)?;

        market_store
            .mut_index_store()
            .add_index(0, Arc::new(RwLock::new(ibor_index)))?;
        Ok(market_store)
    }

    #[test]
    fn test() -> Result<()> {
        let start_date = Date::new(2021, 9, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);

        let rate = InterestRate::from_rate_definition(
            0.06,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Continuous,
                Frequency::Semiannual,
            ),
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(100.0)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Semiannual)
            .bullet()
            .build()?;

        let market_store = create_store()?;

        let model = SimpleModel::new(&market_store);
        let zspread_rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Continuous,
            Frequency::Semiannual,
        );
        let zspread_visitor = ZSpreadConstVisitor::new(&model, zspread_rate_definition, 100.0);

        let zspread = zspread_visitor.visit(&instrument)?;
        println!("ZSpread: {}", zspread * 100.0);
        Ok(())
    }
}
