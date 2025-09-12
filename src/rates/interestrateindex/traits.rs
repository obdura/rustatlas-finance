use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::{
    currencies::enums::Currency, math::interpolation::enums::Interpolator, rates::{
        traits::{HasReferenceDate, YieldProvider},
        yieldtermstructure::traits::YieldTermStructureTrait,
    }, time::{date::Date, enums::TimeUnit, period::Period}, utils::errors::Result
};

/// # FixingProvider
/// Implement this trait for a struct that provides fixing information.
/// 
/// * fixing - Get the fixing for a given date.
/// * fixings - Get all the fixings.
/// * add_fixing - Add a fixing for a given date.
/// * set_fixings - Set the fixings 
/// * fill_missing_fixings - Fill missing fixings using interpolation.
/// 
pub trait FixingProvider {
    fn fixing(&self, date: Date) -> Result<f64>;
    fn fixings(&self) -> &HashMap<Date, f64>;
    fn add_fixing(&mut self, date: Date, rate: f64);
    fn set_fixings(&mut self, fixings: HashMap<Date, f64>) -> Result<()>;
    fn fill_missing_fixings(&mut self, interpolator: Interpolator) -> Result<()> {
        if !self.fixings().is_empty() {
            let first_date = self.fixings().keys().min().unwrap().clone();
            let last_date = self.fixings().keys().max().unwrap().clone();

            let aux_btreemap = self
                .fixings()
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect::<BTreeMap<Date, f64>>();

            let x: Vec<f64> = aux_btreemap
                .keys()
                .map(|&d| (d - first_date) as f64)
                .collect::<Vec<f64>>();

            let y = aux_btreemap.values().map(|r| *r).collect::<Vec<f64>>();

            let mut current_date = first_date.clone();

            while current_date <= last_date {
                if !self.fixings().contains_key(&current_date) {
                    let days = (current_date - first_date) as f64;
                    let rate = interpolator.interpolate(days, &x, &y, false)?;
                    self.add_fixing(current_date, rate);
                }
                current_date = current_date + Period::new(1, TimeUnit::Days);
            }
        }

        Ok(())
    }
}

/// # AdvanceInterestRateIndexInTime
/// 
/// Trait for advancing in time a given object. Returns a represation of the object
/// as it would be after the given period/time.
/// 
/// * advance_to_period - Advance to a given period.
/// * advance_to_date - Advance to a given date.
pub trait AdvanceInterestRateIndexInTime {
    fn advance_to_period(&self, period: Period) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>>;
    fn advance_to_date(&self, date: Date) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>>;
}
/// # HasTenor
/// 
/// Implement this trait for a struct that holds a tenor.
/// 
/// * tenor - Get the tenor.
/// 
pub trait HasTenor {
    fn tenor(&self) -> Period;
}

/// # HasTermStructure
/// Implement this trait for a struct that holds a term structure.
/// 
/// * term_structure - Get the term structure.
/// 
pub trait HasTermStructure {
    fn term_structure(&self) -> Result<Arc<dyn YieldTermStructureTrait>>;
}

/// # HasName
/// Implement this trait for a struct that holds a name.
/// 
/// * name - Get the name.
///
pub trait HasName {
    fn name(&self) -> Result<String>;
}

/// # RelinkableTermStructure
/// Allows to link a term structure to another.
/// 
/// * link_to - Link to another term structure.
pub trait RelinkableTermStructure {
    fn link_to(&mut self, term_structure: Arc<dyn YieldTermStructureTrait>);
    fn rename_to(&mut self, name: String);
}


/// # HasCurrency
/// Implement this trait for a struct that holds currency information.
/// 
/// * currency - Get the currency.
pub trait OverCurrency {
    fn currency(&self) -> Result<Option<Currency>>;
}
/// # InterestRateIndexTrait
/// Implement this trait for a struct that holds interest rate index information.
/// 
/// It trait is a combination of the following traits:
/// * FixingProvider
/// * YieldProvider
/// * HasReferenceDate
/// * AdvanceInterestRateIndexInTime
/// * HasTermStructure
/// * RelinkableTermStructure
/// * HasTenor
/// * HasName
/// * Send
/// * Sync
/// 
/// * clone_box - Clone the object as a boxed object.
/// 
pub trait InterestRateIndexTrait:
    FixingProvider
    + YieldProvider
    + HasReferenceDate
    + AdvanceInterestRateIndexInTime
    + HasTermStructure
    + RelinkableTermStructure
    + HasTenor
    + HasName
    + OverCurrency
    + Send
    + Sync
{
    fn clone_box(&self) -> Arc<RwLock<dyn InterestRateIndexTrait>>;
    fn name_long_detail(&self) -> Result<String> {
        let name = self.name()?;
        let currency = self.currency()?;

        let mut details = format!("{}", name);
        if let Some(ccy) = currency {
            details.push_str(&format!("  ({})", ccy));
        }
        Ok(details)
    }
}
