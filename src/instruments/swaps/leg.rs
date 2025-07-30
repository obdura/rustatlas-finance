use crate::utils::errors::Result;
use crate::{
    cashflows::{
        cashflow::Cashflow,
        side::Side,
        traits::{InterestAccrual, Payable},
    },
    core::traits::HasCurrency,
    currencies::enums::Currency,
    instruments::{traits::RateType, traits::Structure},
    rates::interestrate::RateDefinition,
    time::{date::Date, enums::Frequency},
    visitors::traits::HasCashflows,
};
use std::fmt::Display;

/// # Leg
/// A financial leg. Contains a stream of cashflows. Instruments have one or more legs.
#[derive(Debug, Clone)]
pub struct Leg {
    negotiation_date: Option<Date>,
    start_date: Date,
    end_date: Date, // maturity date
    last_payment_date: Date, // last payment date
    notional: f64,
    payment_frequency: Frequency,
    structure: Structure,
    rate_type: RateType,
    rate_value: f64,
    rate_definition: RateDefinition,
    currency: Currency,
    side: Side,
    discount_curve_id: Option<usize>,
    forecast_curve_id: Option<usize>,
    cashflows: Vec<Cashflow>,
    mtm: Option<f64>
}

impl Leg {
    pub fn new(
        negotiation_date: Option<Date>,
        start_date: Date,
        end_date: Date,
        last_payment_date: Date,
        notional: f64,
        payment_frequency: Frequency,
        structure: Structure,
        rate_type: RateType,
        rate_value: f64,
        rate_definition: RateDefinition,
        currency: Currency,
        side: Side,
        discount_curve_id: Option<usize>,
        forecast_curve_id: Option<usize>,
        cashflows: Vec<Cashflow>,
    ) -> Self {
        Leg {
            negotiation_date,
            start_date,
            end_date,
            last_payment_date, 
            notional,
            payment_frequency,
            structure,
            rate_type,
            rate_value,
            rate_definition,
            currency,
            side,
            discount_curve_id,
            forecast_curve_id,
            cashflows,
            mtm: None
        }
    }

    pub fn start_date(&self) -> Date {
        self.start_date
    }

    pub fn negociation_date(&self) -> Option<Date> {
        self.negotiation_date
    }

    pub fn end_date(&self) -> Date {
        self.end_date
    }

    pub fn last_payment_date(&self) -> Date {
        self.last_payment_date
    }   

    pub fn notional(&self) -> f64 {
        self.notional
    }

    pub fn payment_frequency(&self) -> Frequency {
        self.payment_frequency
    }

    pub fn structure(&self) -> Structure {
        self.structure
    }

    pub fn rate_type(&self) -> RateType {
        self.rate_type
    }

    pub fn rate_value(&self) -> f64 {
        self.rate_value
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn discount_curve_id(&self) -> Option<usize> {
        self.discount_curve_id
    }

    pub fn forecast_curve_id(&self) -> Option<usize> {
        self.forecast_curve_id
    }

    pub fn mtm(&self) -> Option<f64> {
        self.mtm
    }

    pub fn clear(&mut self) {
        self.cashflows.clear();
    }

    pub fn set_rate_value(mut self, rate_value: f64) -> Self {
        self.rate_value = rate_value;
        self.mut_cashflows().for_each(|cashflow| {
            match cashflow {
                Cashflow::FixedRateCoupon(coupon) => {
                    coupon.set_rate_value(rate_value);
                }
                Cashflow::FloatingRateCoupon(coupon) => {
                    coupon.set_spread(rate_value);
                }
                _ => {}
            }
        });
        self
    }

    pub fn set_mtm(&mut self, mtm: f64) {
        self.mtm = Some(mtm);
    }
}

impl HasCurrency for Leg {
    fn currency(&self) -> Result<Currency> {
        Ok(self.currency)
    }
}

impl HasCashflows for Leg {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.cashflows.iter())
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(self.cashflows.iter_mut())
    }
}

impl InterestAccrual for Leg {
    fn accrual_start_date(&self) -> Result<Date> {
        Ok(self.start_date)
    }
    fn accrual_end_date(&self) -> Result<Date> {
        Ok(self.end_date)
    }
    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let total_accrued_amount = self.cashflows.iter().fold(0.0, |acc, cf| {
            acc + cf.accrued_amount(start_date, end_date).unwrap_or(0.0)
        });
        Ok(total_accrued_amount)
    }

    fn accrued_amount_map(&self) -> Result<std::collections::BTreeMap<Date, f64>> {
        let map = self
            .cashflows
            .iter()
            .fold(std::collections::BTreeMap::new(), |mut acc, cf| {
                let cf_map = cf.accrued_amount_map().unwrap();
                for (date, amount) in cf_map {
                    let entry = acc.entry(date).or_insert(0.0);
                    *entry += amount;
                }
                acc
            });
        Ok(map)
    }
}

/// # Display
///
/// Implement the display leg
use colored::*;
impl Display for Leg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t{} {}", "rate type:".bold().magenta(), self.rate_type().to_string().cyan())?;
        writeln!(f, "\t{} {}", "side:".bold().magenta(), self.side().to_string().cyan())?;
        writeln!(f, "\t{} {}", "notional:".bold().magenta(), self.notional().to_string().cyan())?;
        writeln!(f, "\t{} {}", "currency:".bold().magenta(), self.currency().to_string().cyan())?;
        writeln!(f, "\t{} {}", "start_date:".bold().magenta(), self.start_date().to_string().cyan())?;
        writeln!(f, "\t{} {}", "end_date:".bold().magenta(), self.end_date().to_string().cyan())?;
        writeln!(f, "\t{} {}", "structure:".bold().magenta(), self.structure().to_string().cyan())?;
        writeln!(f, "\t{} {}", "payment_frequency:".bold().magenta(), self.payment_frequency().to_string().cyan())?;
        writeln!(f, "\t{} {}", "rate definition:".bold().magenta(), self.rate_definition().to_string().cyan())?;
        writeln!(f, "\t{} {}", "rate/spread value:".bold().magenta(), self.rate_value().to_string().cyan())?;
        writeln!(f, "\t{}", "cashflows:".bold().magenta())?;
        // Group cashflows by payment date and print them in order
        let btree_map =
            self.cashflows
                .iter()
                .fold(std::collections::BTreeMap::new(), |mut acc, cf| {
                    let date = cf.payment_date();
                    acc.entry(date).or_insert(vec![]).push(cf);
                    acc
                });
        for (_, cashflows) in btree_map.iter() {
            for cf in cashflows {
                write!(f,"{}","\t\t-> ".white().bold())?;
                writeln!(f, "{}", cf)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    };

    use crate::utils::errors::Result;
    use crate::{
        cashflows::side::Side,
        core::marketstore::MarketStore,
        currencies::enums::Currency,
        instruments::constructors::makefixedrateleg::MakeFixedRateLeg,
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
        visitors::{
            indexingvisitors::indexingvisitor::IndexingVisitor,
            npvvisitors::npvconstvisitor::NPVConstVisitor,
            parvaluevisitors::traits::ParValueConstVisitor,
            traits::{ConstVisit, Visit},
        },
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
    fn test_npv_leg_receive() -> Result<()> {
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

        let mut instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_notional(notional)
            .bullet()
            .with_discount_curve_id(Some(2))
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let npv_visitor = NPVConstVisitor::new(&data, true);
        let npv = npv_visitor.visit(&instrument)?;
        assert!((npv - 100_000.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_npv_leg_pay() -> Result<()> {
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

        let mut instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .with_notional(notional)
            .bullet()
            .with_discount_curve_id(Some(2))
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let npv_visitor = NPVConstVisitor::new(&data, true);
        let npv = npv_visitor.visit(&instrument)?;
        assert!((npv + 100_000.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_parvalue_leg_receive() -> Result<()> {
        let market_store = create_store()?;
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

        let mut instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(notional)
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

        let nvp_visitor = NPVConstVisitor::new(&data, true);
        let npv = nvp_visitor.visit(&instrument)?;

        assert!((npv - 100_000.0).abs() < 1e-6);
        
        let parvaluevisitor = ParValueConstVisitor::new_with_target_cost(&data, notional);
        let par_value = parvaluevisitor.visit(&instrument)?;

        assert!((par_value - 0.05).abs() < 1e-6);          
        Ok(())
    }

    
    #[test]
    fn test_parvalue_leg_pay() -> Result<()> {
        let market_store = create_store()?;
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

        let mut instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(notional)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .bullet()
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .build()?;

        let indexer = IndexingVisitor::new();
        indexer.visit(&mut instrument)?;

        let model = SimpleModel::new(&market_store);
        let data = model.gen_market_data(&indexer.request())?;

        let nvp_visitor = NPVConstVisitor::new(&data, true);
        let npv = nvp_visitor.visit(&instrument)?;

        assert!((npv + 100_000.0).abs() < 1e-6);
        
        let parvaluevisitor = ParValueConstVisitor::new_with_target_cost(&data, npv);
        let par_value = parvaluevisitor.visit(&instrument)?;
        assert!((par_value - 0.05).abs() < 1e-6);          
        Ok(())
    }

    #[test]
    fn test_leg_display() -> Result<()> {
        let market_store = create_store()?;
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

        let instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate(rate)
            .with_notional(notional)
            .with_payment_frequency(Frequency::Semiannual)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .bullet()
            .with_discount_curve_id(Some(2))
            .with_notional(notional)
            .build()?;

        println!("{}", instrument);

        Ok(())
    }
}
