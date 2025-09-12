use crate::{
    currencies::enums::Currency, rates::{
        enums::Compounding,
        interestrate::RateDefinition,
        traits::{HasReferenceDate, YieldProvider},
        yieldtermstructure::traits::YieldTermStructureTrait,
    }, time::{
        date::Date,
        enums::{Frequency, TimeUnit},
        period::Period,
    }, utils::errors::{AtlasError, Result}
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::traits::{
    AdvanceInterestRateIndexInTime, FixingProvider, OverCurrency, HasName, HasTenor, HasTermStructure, InterestRateIndexTrait, RelinkableTermStructure
};

/// # IborIndex
/// Struct that defines an Ibor index.
/// 
/// ## Fields
/// * name - The name of the index.
/// * tenor - The tenor of the index.
/// * rate_definition - The rate definition of the index.
/// * fixings - The fixings of the index.
/// * term_structure - The term structure of the index.
/// * reference_date - The reference date of the index.
/// 
///
/// # Example
/// ```
/// use rustatlas::prelude::*;
/// let ref_date = Date::new(2021, 1, 1);
/// let tenor = Period::new(1, TimeUnit::Months);
/// let rate_definition = RateDefinition::new(DayCounter::Actual360, Compounding::Simple, Frequency::Annual);
/// let ibor_index = IborIndex::new(ref_date).with_tenor(tenor).with_rate_definition(rate_definition);
/// assert_eq!(ibor_index.tenor(), tenor);
/// assert_eq!(ibor_index.rate_definition().compounding(), Compounding::Simple);
/// assert_eq!(ibor_index.rate_definition().frequency(), Frequency::Annual);
/// assert_eq!(ibor_index.rate_definition().day_counter(), DayCounter::Actual360);
/// ```
#[derive(Clone)]
pub struct IborIndex {
    name: Option<String>,
    currency: Option<Currency>,
    tenor: Period,
    rate_definition: RateDefinition,
    fixings: HashMap<Date, f64>,
    term_structure: Option<Arc<dyn YieldTermStructureTrait>>,
    reference_date: Date,
}

impl IborIndex {
    pub fn new(reference_date: Date) -> IborIndex {
        IborIndex {
            name: None,
            currency: None,
            reference_date: reference_date,
            tenor: Period::empty(),
            rate_definition: RateDefinition::default(),
            fixings: HashMap::new(),
            term_structure: None,
        }
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_tenor(mut self, tenor: Period) -> Self {
        self.tenor = tenor;
        self
    }

    pub fn with_currency(mut self, currency: Option<Currency>) -> Self {
        self.currency = currency;
        self
    }
    
    pub fn with_frequency(mut self, frequency: Frequency) -> Self {
        self.tenor = Period::from_frequency(frequency).expect("Invalid frequency");
        self
    }

    pub fn with_rate_definition(mut self, rate_definition: RateDefinition) -> Self {
        self.rate_definition = rate_definition;
        self
    }

    pub fn with_fixings(mut self, fixings: HashMap<Date, f64>) -> Self {
        self.fixings = fixings;
        self
    }

    pub fn with_term_structure(mut self, term_structure: Arc<dyn YieldTermStructureTrait>) -> Self {
        self.term_structure = Some(term_structure);
        self
    }
}

// Implement FixingProvider for IborIndex
impl FixingProvider for IborIndex {
    fn fixing(&self, date: Date) -> Result<f64> {
        self.fixings
            .get(&date)
            .cloned()
            .ok_or(AtlasError::NotFoundErr(format!(
                "No fixing for date {} for index {:?}",
                date, self.name
            )))
    }

    fn fixings(&self) -> &HashMap<Date, f64> {
        &self.fixings
    }

    fn add_fixing(&mut self, date: Date, rate: f64) {
        if date > self.reference_date() {
            panic!("Date must be less than reference date");
        }
        self.fixings.insert(date, rate);
    }

    fn set_fixings(&mut self, fixings: HashMap<Date, f64>) -> Result<()> {
        self.fixings = fixings;
        Ok(())
    }
}

// Implement HasReferenceDate for IborIndex
impl HasReferenceDate for IborIndex {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
}

// Implement HasTenor for IborIndex
impl HasTenor for IborIndex {
    fn tenor(&self) -> Period {
        self.tenor
    }
}

// Implement HasName for IborIndex
impl HasName for IborIndex {
    fn name(&self) -> Result<String> {
        self.name
            .clone()
            .ok_or(AtlasError::ValueNotSetErr("Name not set".to_string()))
    }
}

// Implement HasCurrency for IborIndex
impl OverCurrency for IborIndex {
    fn currency(&self) -> Result<Option<Currency>> {
        Ok(self.currency)
    }
}

// Implement YieldProvider for IborIndex
impl YieldProvider for IborIndex {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        self.term_structure()?.discount_factor(date)
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<f64> {
        if end_date < start_date {
            return Err(AtlasError::InvalidValueErr(format!(
                "End date {:?} is before start date {:?}",
                end_date, start_date
            )));
        }
        if start_date < self.reference_date() {
            return self.fixing(start_date);
        } else {
            return self
                .term_structure()?
                .forward_rate(start_date, end_date, comp, freq);
        }
    }
}

// Implement HasTermStructure for IborIndex
impl HasTermStructure for IborIndex {
    fn term_structure(&self) -> Result<Arc<dyn YieldTermStructureTrait>> {
        self.term_structure
            .clone()
            .ok_or(AtlasError::ValueNotSetErr(
                "Term structure not set".to_string(),
            ))
    }
}

// Implement RelinkableTermStructure for IborIndex
impl RelinkableTermStructure for IborIndex {
    fn link_to(&mut self, term_structure: Arc<dyn YieldTermStructureTrait>) {
        self.term_structure = Some(term_structure);
    }
    fn rename_to(&mut self, name: String) {
        self.name = Some(name);
    }
}

// Implement Clone for IborIndex
impl InterestRateIndexTrait for IborIndex {
    fn clone_box(&self) -> Arc<RwLock<dyn InterestRateIndexTrait>> {
        Arc::new(RwLock::new(self.clone()))
    }
}

// Implement AdvanceInterestRateIndexInTime for IborIndex
impl AdvanceInterestRateIndexInTime for IborIndex {
    fn advance_to_period(&self, period: Period) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        let curve = self.term_structure()?;
        let mut fixings = self.fixings().clone();
        let mut seed = self.reference_date();
        let end_date = seed.advance(period.length(), period.units());

        while seed <= end_date {
            let rate = curve.forward_rate(
                seed,
                seed + self.tenor,
                self.rate_definition.compounding(),
                self.rate_definition.frequency(),
            )?;
            fixings.insert(seed, rate);
            seed = seed.advance(1, TimeUnit::Days);
        }
        let new_curve = curve.advance_to_period(period)?;
        Ok(Arc::new(RwLock::new(
            IborIndex::new(new_curve.reference_date())
                .with_tenor(self.tenor)
                .with_rate_definition(self.rate_definition)
                .with_fixings(fixings)
                .with_term_structure(new_curve)
                .with_name(self.name.clone())
                .with_currency(self.currency.clone()),
        )))
    }

    fn advance_to_date(&self, date: Date) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        let days = (date - self.reference_date()) as i32;
        if days < 0 {
            return Err(AtlasError::InvalidValueErr(format!(
                "Date {} is before reference date {}",
                date,
                self.reference_date()
            )));
        }
        let period = Period::new(days, TimeUnit::Days);
        self.advance_to_period(period)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::interpolation::enums::Interpolator,
        rates::{indexstore::ReadIndex, yieldtermstructure::{
            compositetermstructure::CompositeTermStructure,
            flatforwardtermstructure::FlatForwardTermStructure,
        }},
        time::{daycounter::DayCounter, enums::TimeUnit},
    };

    #[test]
    fn test_ibor_index() {
        let ref_date = Date::new(2021, 1, 1);
        let tenor = Period::new(1, TimeUnit::Months);
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );
        let ibor_index = IborIndex::new(ref_date)
            .with_tenor(tenor)
            .with_rate_definition(rate_definition);
        assert_eq!(ibor_index.tenor(), tenor);
        assert_eq!(
            ibor_index.rate_definition().compounding(),
            Compounding::Simple
        );
        assert_eq!(ibor_index.rate_definition().frequency(), Frequency::Annual);
        assert_eq!(
            ibor_index.rate_definition().day_counter(),
            DayCounter::Actual360
        );
    }

    #[test]
    fn test_fixing_interpolation_ibor() -> Result<()> {
        let fixing: HashMap<Date, f64> = [
            (Date::new(2023, 6, 1), 21938.71),
            (Date::new(2023, 6, 2), 21945.57),
            (Date::new(2023, 6, 5), 21966.14),
            (Date::new(2023, 6, 6), 21973.0),
        ]
        .iter()
        .cloned()
        .collect();
        let mut ibor_index = IborIndex::new(Date::new(2023, 11, 6)).with_fixings(fixing);
        ibor_index.fill_missing_fixings(Interpolator::Linear)?;
        assert!(ibor_index.fixings().get(&Date::new(2023, 6, 3)).unwrap() - 21952.4266666 < 0.001);
        Ok(())
    }

    #[test]
    fn test_relink_term_structure() {
        let ref_date = Date::new(2021, 1, 1);
        let eval_date = ref_date + Period::new(1, TimeUnit::Years);
        let tenor = Period::new(1, TimeUnit::Months);
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );
        let mut ibor_index = IborIndex::new(ref_date)
            .with_tenor(tenor)
            .with_rate_definition(rate_definition);

        let base_term_structure = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.05,
            RateDefinition::default(),
        ));

        let spread_term_structure = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.01,
            RateDefinition::default(),
        ));

        let new_term_structure = Arc::new(CompositeTermStructure::new(
            spread_term_structure.clone(),
            base_term_structure.clone(),
        ));

        ibor_index.link_to(base_term_structure.clone());
        let df = ibor_index
            .term_structure()
            .unwrap()
            .discount_factor(eval_date)
            .unwrap();

        assert_eq!(df, base_term_structure.discount_factor(eval_date).unwrap());

        ibor_index.link_to(new_term_structure.clone());

        let df = ibor_index
            .term_structure()
            .unwrap()
            .discount_factor(eval_date)
            .unwrap();

        assert_eq!(df, new_term_structure.discount_factor(eval_date).unwrap());
    }

    #[test]
    fn test_advance_to_period_with_currency() -> Result<()> {
        let date = Date::new(2021, 1, 1);
        let ibor_index = IborIndex::new(date)
            .with_currency(Some(Currency::USD))
            .with_name(Some("overnight_index_test".to_string()))
            .with_term_structure(Arc::new(FlatForwardTermStructure::new(
                date,
                0.2,
                RateDefinition::default(),
            )));

        let ibor_index = ibor_index.advance_to_period(Period::new(1, TimeUnit::Years))?;
        assert!(ibor_index.read_index()?.currency()? == Some(Currency::USD));
        Ok(())
    }
}
