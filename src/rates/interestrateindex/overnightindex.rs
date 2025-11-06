use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    currencies::enums::Currency,
    rates::{
        enums::Compounding,
        interestrate::RateDefinition,
        traits::{HasReferenceDate, YieldProvider},
        yieldtermstructure::traits::YieldTermStructureTrait,
    },
    time::{
        date::Date, daycounter::DayCounter, enums::{Frequency, TimeUnit}, period::Period
    },
    utils::errors::{AtlasError, Result},
};

use super::traits::{
    AdvanceInterestRateIndexInTime, FixingProvider, HasName, HasTenor, HasTermStructure,
    InterestRateIndexTrait, OverCurrency, RelinkableTermStructure,
};

/// # OvernightIndex
///
/// Overnight index, used for overnight rates. Uses a price index (such as ICP) to calculate the overnight rates.
///
/// ## Fields
/// * name - The name of the index.
/// * fixings - The fixings for the index.
/// * term_structure - The term structure for the index.
/// * rate_definition - The rate definition for the index.
/// * tenor - The tenor for the index.
/// * reference_date - The reference date for the index.
///
#[derive(Clone)]
pub struct OvernightIndex {
    name: Option<String>,
    currency: Option<Currency>,
    fixings: HashMap<Date, f64>,
    term_structure: Option<Arc<dyn YieldTermStructureTrait>>,
    rate_definition: RateDefinition,
    tenor: Period,
    reference_date: Date,
}

impl OvernightIndex {
    pub fn new(reference_date: Date) -> OvernightIndex {
        OvernightIndex {
            name: None,
            currency: None,
            fixings: HashMap::new(),
            term_structure: None,
            rate_definition: RateDefinition::default(),
            tenor: Period::new(1, TimeUnit::Days),
            reference_date: reference_date,
        }
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_currency(mut self, currency: Option<Currency>) -> Self {
        self.currency = currency;
        self
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
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

    pub fn average_rate(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let start_index = self.fixing(start_date)?;
        let end_index = self.fixing(end_date)?;

        let comp = end_index / start_index;
        let rate = self.rate_definition.implied_rate_from_dates(start_date, end_date, comp)?;
        return Ok(rate.rate());
    }
}

// Implement FixingProvider for OvernightIndex
impl FixingProvider for OvernightIndex {
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
        self.fixings.insert(date, rate);
    }

    fn set_fixings(&mut self, fixings: HashMap<Date, f64>) -> Result<()> {
        self.fixings = fixings;
        Ok(())
    }
}

// Implement HasReferenceDate for OvernightIndex
impl HasReferenceDate for OvernightIndex {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
}

// Implement HasTenor for OvernightIndex
impl HasTenor for OvernightIndex {
    fn tenor(&self) -> Period {
        self.tenor
    }
}

// Implement HasName for OvernightIndex
impl HasName for OvernightIndex {
    fn name(&self) -> Result<String> {
        self.name
            .clone()
            .ok_or(AtlasError::ValueNotSetErr("Name not set".to_string()))
    }
}

// Implement HasCurrency for OvernightIndex
impl OverCurrency for OvernightIndex {
    fn currency(&self) -> Result<Option<Currency>> {
        Ok(self.currency)
    }
} 

// Implement YieldProvider for OvernightIndex
impl YieldProvider for OvernightIndex {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        self.term_structure()?.discount_factor(date)
    }

    fn discount_factor_between_dates(&self, start_date: Date, end_date: Date) -> Result<f64> {
        self.term_structure()?.discount_factor_between_dates(start_date, end_date)
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
        day_counter: DayCounter,
    ) -> Result<f64> {
        // mixed case - return w.a.
        if start_date < self.reference_date() && end_date > self.reference_date() {
            let first_fixing = self.fixing(start_date)?;
            let second_fixing = self.fixing(self.reference_date())?;

            let df = self.term_structure()?.discount_factor(end_date)?;

            let third_fixing = second_fixing / df;

            let compounding= third_fixing / first_fixing;
            let rate_definition = RateDefinition::new(day_counter, comp, freq);
            let yf = day_counter.year_fraction(start_date, end_date);
            return Ok(rate_definition.implied_rate(compounding, yf)?.rate());
        }

        // past fixing case
        if start_date < self.reference_date() && end_date <= self.reference_date() {
            return self.average_rate(start_date, end_date);
        }

        // forecast case
        if start_date >= self.reference_date() && end_date > self.reference_date() {
            self.term_structure()?
                .forward_rate(start_date, end_date, comp, freq, day_counter)
        } else {
            Err(AtlasError::InvalidValueErr(format!(
                "Invalid dates: start_date: {:?}, end_date: {:?}",
                start_date, end_date
            )))
        }
    }
}

// Implement AdvanceInterestRateIndexInTime for OvernightIndex
impl AdvanceInterestRateIndexInTime for OvernightIndex {
    fn advance_to_period(&self, period: Period) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        let mut fixings = self.fixings().clone();
        let mut seed = self.reference_date();
        let end_date = seed.advance(period.length(), period.units());
        let curve = self.term_structure()?;
        let name = self.name()?;

        if !fixings.is_empty() {
            let mut last_fixing_date = fixings.iter().map(|(a, _)| a).max().cloned().unwrap();
            if seed > last_fixing_date {
                let last_fixing = *fixings.get(&last_fixing_date).unwrap();
                let first_df = curve.discount_factor(seed)?;
                let second_df = curve.discount_factor(seed.advance(1, TimeUnit::Days))?;
                while seed > last_fixing_date {
                    last_fixing_date = last_fixing_date.advance(1, TimeUnit::Days);
                    fixings.insert(last_fixing_date, last_fixing * first_df / second_df);
                }
            }

            while seed < end_date {
                let first_df = curve.discount_factor(seed)?;
                let last_fixing = fixings.get(&seed).ok_or(AtlasError::NotFoundErr(format!(
                    "No fixing for {} and date {}",
                    name, seed
                )))?;
                seed = seed.advance(1, TimeUnit::Days);
                let second_df = curve.discount_factor(seed)?;
                let comp = last_fixing * first_df / second_df;
                fixings.insert(seed, comp);
            }
        }

        let new_curve = curve.advance_to_period(period)?;

        Ok(Arc::new(RwLock::new(
            OvernightIndex::new(new_curve.reference_date())
                .with_rate_definition(self.rate_definition)
                .with_fixings(fixings)
                .with_term_structure(new_curve)
                .with_name(self.name.clone())
                .with_currency(self.currency.clone()),
        )))
    }

    fn advance_to_date(&self, date: Date) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        let days = (date - self.reference_date()) as i32;
        let period = Period::new(days, TimeUnit::Days);
        self.advance_to_period(period)
    }
}

// Implement HasTermStructure for OvernightIndex
impl HasTermStructure for OvernightIndex {
    fn term_structure(&self) -> Result<Arc<dyn YieldTermStructureTrait>> {
        self.term_structure
            .clone()
            .ok_or(AtlasError::ValueNotSetErr(
                "Term structure not set".to_string(),
            ))
    }
}

// Implement RelinkableTermStructure for OvernightIndex
impl RelinkableTermStructure for OvernightIndex {
    fn link_to(&mut self, term_structure: Arc<dyn YieldTermStructureTrait>) {
        self.term_structure = Some(term_structure);
    }
    fn rename_to(&mut self, name: String) {
        self.name = Some(name);
    }
}

// Implement InterestRateIndexTrait for OvernightIndex
impl InterestRateIndexTrait for OvernightIndex {
    fn clone_box(&self) -> Arc<RwLock<dyn InterestRateIndexTrait>> {
        Arc::new(RwLock::new(self.clone()))
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        math::interpolation::enums::Interpolator,
        rates::{indexstore::ReadIndex, yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure},
    };

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_new_overnight_index() {
        let date = Date::new(2021, 1, 1);
        let overnight_index = OvernightIndex::new(date);
        assert!(overnight_index.fixings.is_empty());
        assert!(overnight_index.term_structure.is_none());
    }

    #[test]
    fn test_with_rate_definition() {
        let date = Date::new(2021, 1, 1);
        let overnight_index =
            OvernightIndex::new(date).with_rate_definition(RateDefinition::default());
        assert_eq!(overnight_index.rate_definition, RateDefinition::default());
    }

    #[test]
    fn test_with_fixings() {
        let date = Date::new(2021, 1, 1);
        let mut fixings = HashMap::new();
        fixings.insert(Date::new(2021, 1, 1), 0.02);
        let overnight_index = OvernightIndex::new(date).with_fixings(fixings.clone());
        assert_eq!(overnight_index.fixings, fixings);
    }

    #[test]
    fn test_average_rate() {
        let date = Date::new(2021, 1, 1);
        let mut fixings = HashMap::new();
        let start_date = Date::new(2021, 1, 1);
        let end_date = Date::new(2022, 1, 1);

        fixings.insert(start_date, 100.0);
        fixings.insert(end_date, 105.0);
        let overnight_index = OvernightIndex::new(date)
            .with_fixings(fixings)
            .with_rate_definition(RateDefinition::default());

        let average_rate = overnight_index.average_rate(start_date, end_date).unwrap();

        // Add your assertions here based on how average_rate is calculated
        assert!(average_rate > 0.0);
    }

    #[test]
    fn test_fixing() -> Result<()> {
        let date = Date::new(2021, 1, 1);
        let mut fixings = HashMap::new();
        fixings.insert(Date::new(2021, 1, 1), 0.02);
        let overnight_index = OvernightIndex::new(date).with_fixings(fixings);

        assert_eq!(overnight_index.fixing(Date::new(2021, 1, 1))?, 0.02);
        Ok(())
    }

    #[test]
    fn test_reference_date() {
        let date = Date::new(2021, 1, 1);
        let mut fixings = HashMap::new();
        let ref_date = Date::new(2021, 1, 1);

        fixings.insert(ref_date, 100.0);
        let overnight_index = OvernightIndex::new(date)
            .with_fixings(fixings.clone())
            .with_term_structure(Arc::new(FlatForwardTermStructure::new(
                ref_date,
                0.2,
                RateDefinition::default(),
            )));

        assert_eq!(overnight_index.reference_date(), ref_date);

        let next_date_2 = Date::new(2021, 1, 3);
        fixings.insert(next_date_2, 100.0);
        let overnight_index = OvernightIndex::new(next_date_2)
            .with_term_structure(Arc::new(FlatForwardTermStructure::new(
                next_date_2,
                0.2,
                RateDefinition::default(),
            )))
            .with_fixings(fixings);

        assert_eq!(overnight_index.reference_date(), next_date_2);
    }

    #[test]
    fn test_fixing_provider_overnight() -> Result<()> {
        let fixing: HashMap<Date, f64> = [
            (Date::new(2023, 6, 2), 21945.57),
            (Date::new(2023, 6, 5), 21966.14),
        ]
        .iter()
        .cloned()
        .collect();

        let mut overnight_index = OvernightIndex::new(Date::new(2023, 6, 5)).with_fixings(fixing);

        overnight_index.fill_missing_fixings(Interpolator::Linear)?;

        assert!(
            overnight_index
                .fixings()
                .get(&Date::new(2023, 6, 3))
                .unwrap()
                - 21952.4266666
                < 0.001
        );
        Ok(())
    }

    #[test]
    fn test_advance_to_period() -> Result<()> {
        let mut fixing: HashMap<Date, f64> = HashMap::new();
        fixing.insert(Date::new(2023, 6, 2), 21945.57);
        fixing.insert(Date::new(2023, 6, 5), 21966.14);

        let mut overnight_index = OvernightIndex::new(Date::new(2023, 7, 6)).with_fixings(fixing);
        overnight_index.fill_missing_fixings(Interpolator::Linear)?;

        Ok(())
    }

    #[test]
    fn test_advance_to_period_with_currency() -> Result<()> {
        let date = Date::new(2021, 1, 1);
        let overnight_index = OvernightIndex::new(date)
            .with_currency(Some(Currency::USD))
            .with_name(Some("overnight_index_test".to_string()))
            .with_term_structure(Arc::new(FlatForwardTermStructure::new(
                date,
                0.2,
                RateDefinition::default(),
            )));

        let overnight_index = overnight_index.advance_to_period(Period::new(1, TimeUnit::Years))?;
        assert!(overnight_index.read_index()?.currency()? == Some(Currency::USD));
        Ok(())
    }
}
