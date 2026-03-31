use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    currencies::enums::Currency,
    rates::{
        interestrateindex::overnightindex::OvernightIndex,
        yieldtermstructure::synthetictermstructure::SyntheticTermStructure,
    },
    time::{date::Date, enums::TimeUnit, period::Period},
    utils::errors::{AtlasError, Result},
};

use super::{
    interestrateindex::traits::InterestRateIndexTrait,
    yieldtermstructure::traits::YieldTermStructureTrait,
};

/// # IndexStore
/// A store for interest rate indices.
///
/// ## Parameters
/// * `reference_date` - The reference date of the index store
/// * `index_map` - A map of indices
/// * `currency_curve` - A map of currency curves used for forecasting
#[derive(Clone)]
pub struct IndexStore {
    reference_date: Date,
    index_map: HashMap<usize, Arc<RwLock<dyn InterestRateIndexTrait>>>,
    currency_curve: HashMap<Currency, usize>,
}

impl IndexStore {
    pub fn new(reference_date: Date) -> IndexStore {
        IndexStore {
            reference_date,
            index_map: HashMap::new(),
            currency_curve: HashMap::new(),
        }
    }

    pub fn set_currency_curves(&mut self, currency_curve: HashMap<Currency, usize>) -> Result<()> {
        self.currency_curve = currency_curve;
        Ok(())
    }

    pub fn set_index_map(
        &mut self,
        index_map: HashMap<usize, Arc<RwLock<dyn InterestRateIndexTrait>>>,
    ) -> Result<()> {
        self.index_map = index_map;
        Ok(())
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn add_currency_curve(&mut self, currency: Currency, fx_curve: usize) {
        self.currency_curve.insert(currency, fx_curve);
    }

    pub fn get_currency_curve(&self, currency: Currency) -> Result<usize> {
        self.currency_curve
            .get(&currency)
            .cloned()
            .ok_or(AtlasError::NotFoundErr(format!(
                "Currency curve for currency {:?}",
                currency
            )))
    }

    pub fn link_term_structure(
        &self,
        id: usize,
        term_structure: Arc<dyn YieldTermStructureTrait>,
    ) -> Result<()> {
        self.index_map
            .get(&id)
            .ok_or(AtlasError::NotFoundErr(format!(
                "Index with id {} not found",
                id
            )))?
            .write()
            .map_err(|_| AtlasError::InvalidValueErr("Could not write index".to_string()))?
            .link_to(term_structure);
        Ok(())
    }

    pub fn rename_term_structure(&self, id: usize, name: String) -> Result<()> {
        self.index_map
            .get(&id)
            .ok_or(AtlasError::NotFoundErr(format!(
                "Index with id {} not found",
                id
            )))?
            .write()
            .map_err(|_| AtlasError::InvalidValueErr("Could not write index".to_string()))?
            .rename_to(name);
        Ok(())
    }

    pub fn duplicate_index(&mut self, id: usize, new_name: String) -> Result<usize> {
        // check if id exists
        if !self.index_map.contains_key(&id) {
            return Err(AtlasError::InvalidValueErr(format!(
                "Index with id {} does not exist",
                id
            )));
        }

        // check if name already exists
        for index in self.index_map.values() {
            if index.read_index()?.name()? == new_name {
                return Err(AtlasError::InvalidValueErr(format!(
                    "Index with name {} already exists",
                    new_name
                )));
            }
        }

        let cloned_index = {
            let index = self.index_map.get(&id).unwrap();
            let index = index.read().unwrap();
            index.clone_box()
        };

        let new_id = self.next_available_id();
        self.add_index(new_id, cloned_index)?;

        self.rename_term_structure(new_id, new_name)?;
        Ok(new_id)
    }

    pub fn add_index(
        &mut self,
        id: usize,
        index: Arc<RwLock<dyn InterestRateIndexTrait>>,
    ) -> Result<()> {
        if self.reference_date != index.read_index()?.reference_date() {
            return Err(AtlasError::InvalidValueErr(
                format!(
                    "Index ({:?}) reference date ({}) does not match index store reference date ({})",
                    index.read_index()?.name(),
                    index.read_index()?.reference_date(),
                    self.reference_date
                )
                .to_string(),
            ));
        }
        // check if name already exists
        if self.index_map.contains_key(&id) {
            return Err(AtlasError::InvalidValueErr(format!(
                "Index with id {} already exists",
                id
            )));
        }

        self.index_map.insert(id, index);

        Ok(())
    }

    pub fn add_synthetic_index(
        &mut self,
        discount_factor_numerator_ids: Vec<usize>,
        discount_factor_denominator_ids: Vec<usize>,
        name: String,
        id: Option<usize>,
        currency: Currency,
    ) -> Result<()> {
        // check if ids exist
        for id in discount_factor_numerator_ids.iter() {
            if !self.index_map.contains_key(id) {
                return Err(AtlasError::InvalidValueErr(format!(
                    "Index with id {} does not exist",
                    id
                )));
            }
        }

        for id in discount_factor_denominator_ids.iter() {
            if !self.index_map.contains_key(id) {
                return Err(AtlasError::InvalidValueErr(format!(
                    "Index with id {} does not exist",
                    id
                )));
            }
        }

        // check if name already exists
        for index in self.index_map.values() {
            if index.read_index()?.name()? == name {
                return Err(AtlasError::InvalidValueErr(format!(
                    "Index with name {} already exists",
                    name
                )));
            }
        }

        let id = if let Some(id) = id {
            if self.index_map.contains_key(&id) {
                return Err(AtlasError::InvalidValueErr(format!(
                    "Index with id {} already exists",
                    id
                )));
            };
            Ok::<_, AtlasError>(id)
        } else {
            Ok(self.next_available_id())
        }?;


        // create synthetic term structure
        let mut discount_factor_numerator = Vec::new();
        let mut discount_factor_denominator = Vec::new();

        for id in discount_factor_numerator_ids.iter() {
            let index = self.index_map.get(id).unwrap();
            let index = index.read().unwrap().term_structure()?;
            discount_factor_numerator.push(index);
        }

        for id in discount_factor_denominator_ids.iter() {
            let index = self.index_map.get(id).unwrap();
            let index = index.read().unwrap().term_structure()?;
            discount_factor_denominator.push(index);
        }

        let synthetic_term_structure = SyntheticTermStructure::new(
            self.reference_date,
            discount_factor_numerator,
            discount_factor_denominator,
        );

        // create overnight index with synthetic term structure
        let overnight_index = OvernightIndex::new(self.reference_date)
            .with_term_structure(Arc::new(synthetic_term_structure))
            .with_name(Some(name))
            .with_currency(Some(currency));

        self.index_map.insert(
            id,
            Arc::new(RwLock::new(overnight_index)),
        );
        Ok(())
    }

    pub fn replace_index(
        &mut self,
        id: usize,
        index: Arc<RwLock<dyn InterestRateIndexTrait>>,
    ) -> Result<()> {
        if self.reference_date != index.read_index()?.reference_date() {
            return Err(AtlasError::InvalidValueErr(
                format!(
                    "Index ({:?}) reference date ({}) does not match index store reference date ({})",
                    index.read_index()?.name(),
                    index.read_index()?.reference_date(),
                    self.reference_date
                )
                .to_string(),
            ));
        }
        // check if name already exists
        if !self.index_map.contains_key(&id) {
            return Err(AtlasError::InvalidValueErr(format!(
                "Index with id {} does not exist",
                id
            )));
        }

        self.index_map.insert(id, index);

        Ok(())
    }

    pub fn get_index(&self, id: usize) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        self.index_map
            .get(&id)
            .cloned()
            .ok_or(AtlasError::NotFoundErr(format!(
                "Index with id {} not found",
                id
            )))
    }

    pub fn get_currency_of_index_id(&self, id: usize) -> Result<Option<Currency>> {
        self.get_index(id)?.read_index()?.currency()
    }

    pub fn get_index_by_name(
        &self,
        name: String,
    ) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        for (id, index) in self.index_map.iter() {
            if index.read_index()?.name()? == name {
                return self.get_index(*id);
            }
        }
        Err(AtlasError::NotFoundErr(format!(
            "Index with name {} not found",
            name
        )))
    }

    pub fn get_index_names(&self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for index in self.index_map.values() {
            names.push(index.read_index()?.name().unwrap());
        }
        Ok(names)
    }

    pub fn get_index_map(&self) -> Result<HashMap<String, usize>> {
        let mut map = HashMap::new();
        for (id, index) in self.index_map.iter() {
            map.insert(index.read_index()?.name().unwrap(), *id);
        }
        Ok(map)
    }

    pub fn get_all_indices(&self) -> Vec<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        let mut indices = Vec::new();
        for index in self.index_map.values() {
            indices.push(index.clone());
        }
        indices
    }

    pub fn next_available_id(&self) -> usize {
        let keys = self.index_map.keys();
        let mut max = 0;
        for key in keys {
            if *key > max {
                max = *key;
            }
        }
        max + 1
    }

    pub fn advance_to_period(&self, period: Period) -> Result<IndexStore> {
        let reference_date = self.reference_date + period;
        let mut store = IndexStore::new(reference_date);
        for (id, index) in self.index_map.iter() {
            let new_index = index.read_index()?.advance_to_period(period)?;
            store.add_index(*id, new_index)?;
        }

        for (currency, curve) in self.currency_curve.iter() {
            store.add_currency_curve(*currency, *curve);
        }

        Ok(store)
    }

    pub fn advance_to_date(&self, date: Date) -> Result<IndexStore> {
        let days = (date - self.reference_date) as i32;
        self.advance_to_period(Period::new(days, TimeUnit::Days))
    }

    /// # swaps the index with the given id to the given index
    pub fn swap_index_by_id(&mut self, from: usize, to: usize) {
        let index = self.index_map.remove(&from).unwrap();
        self.index_map.insert(to, index);
    }

    pub fn currency_forescast_factor(
        &self,
        first_currency: Currency,
        second_currency: Currency,
        date: Date,
    ) -> Result<f64> {
        if first_currency == second_currency {
            return Ok(1.0);
        }
        let first_id = self.get_currency_curve(first_currency)?;
        let second_id = self.get_currency_curve(second_currency)?;

        let first_curve = self.get_index(first_id)?;
        let second_curve = self.get_index(second_id)?;

        let first_df = first_curve.read_index()?.discount_factor(date)?;
        let second_df = second_curve.read_index()?.discount_factor(date)?;

        Ok(second_df / first_df)
    }

    /// Get the pillar dates of a term structure by index ID
    pub fn get_pillar_dates(&self, id: usize) -> Result<Vec<Date>> {
        let index = self.get_index(id)?;
        let term_structure = index.read_index()?.term_structure()?;
        term_structure.pillar_dates().ok_or(AtlasError::NotFoundErr(
            format!("No pillar dates available for index with id {}", id)
        ))
    }
}

// Implement the ReadIndex trait for Arc<RwLock<dyn InterestRateIndexTrait>>
pub trait ReadIndex {
    fn read_index(&self) -> Result<RwLockReadGuard<dyn InterestRateIndexTrait>>;
}

// Implement the ReadIndex trait for Arc<RwLock<dyn InterestRateIndexTrait>>
impl ReadIndex for Arc<RwLock<dyn InterestRateIndexTrait>> {
    fn read_index(&self) -> Result<RwLockReadGuard<dyn InterestRateIndexTrait>> {
        self.read()
            .map_err(|_| AtlasError::InvalidValueErr("Could not read index".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        math::interpolation::enums::Interpolator,
        rates::{
            interestrateindex::iborindex::IborIndex,
            yieldtermstructure::discounttermstructure::DiscountTermStructure,
        },
        time::daycounter::DayCounter,
    };

    #[test]
    fn test_rename_index() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                dates,
                discount_factors,
                day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let discount_index = IborIndex::new(ref_date)
            .with_term_structure(discount_term_structure)
            .with_name(Some("discount_index_test".to_string()));

        let mut index_store = IndexStore::new(Date::new(2020, 1, 1));
        index_store.add_index(0, Arc::new(RwLock::new(discount_index)))?;
        index_store.rename_term_structure(0, "discount_index_test_renamed".to_string())?;

        assert_eq!(
            index_store.get_index(0)?.read_index()?.name()?,
            "discount_index_test_renamed"
        );
        Ok(())
    }

    #[test]
    fn test_relink_term_structure() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let mut fixings: HashMap<Date, f64> = HashMap::new();
        fixings.insert(Date::new(2019, 12, 31), 0.12);
        fixings.insert(Date::new(2019, 12, 30), 0.13);
        fixings.insert(Date::new(2019, 12, 29), 0.14);

        let discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                dates,
                discount_factors,
                day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let discount_index = IborIndex::new(ref_date)
            .with_term_structure(discount_term_structure.clone())
            .with_name(Some("discount_index_test".to_string()))
            .with_fixings(fixings);

        let mut index_store = IndexStore::new(Date::new(2020, 1, 1));
        index_store.add_index(0, Arc::new(RwLock::new(discount_index)))?;

        let fixings_out = index_store.get_index(0)?.read_index()?.fixings().clone();

        // check fixings
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 31)).unwrap(), &0.12);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 30)).unwrap(), &0.13);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 29)).unwrap(), &0.14);

        // check discount factors
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 1, 1))?,
            1.0
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 4, 1))?,
            0.99
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 7, 1))?,
            0.98
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 10, 1))?,
            0.97
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2021, 1, 1))?,
            0.96
        );

        // create new term structure
        let new_dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let new_discount_factors = vec![1.0, 0.89, 0.88, 0.87, 0.86];
        let new_day_counter = DayCounter::Actual360;

        let new_discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                new_dates,
                new_discount_factors,
                new_day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        index_store.link_term_structure(0, new_discount_term_structure)?;

        let fixings_out = index_store.get_index(0)?.read_index()?.fixings().clone();

        // check fixings
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 31)).unwrap(), &0.12);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 30)).unwrap(), &0.13);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 29)).unwrap(), &0.14);

        // check discount factors
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 1, 1))?,
            1.0
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 4, 1))?,
            0.89
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 7, 1))?,
            0.88
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2020, 10, 1))?,
            0.87
        );
        assert_eq!(
            index_store
                .get_index(0)?
                .read_index()?
                .discount_factor(Date::new(2021, 1, 1))?,
            0.86
        );

        Ok(())
    }

    #[test]
    fn test_duplicate() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let mut fixings: HashMap<Date, f64> = HashMap::new();
        fixings.insert(Date::new(2019, 12, 31), 0.12);
        fixings.insert(Date::new(2019, 12, 30), 0.13);
        fixings.insert(Date::new(2019, 12, 29), 0.14);

        let discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                dates,
                discount_factors,
                day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let discount_index = IborIndex::new(ref_date)
            .with_term_structure(discount_term_structure.clone())
            .with_name(Some("discount_index_test".to_string()))
            .with_fixings(fixings);

        let mut index_store = IndexStore::new(Date::new(2020, 1, 1));
        index_store.add_index(0, Arc::new(RwLock::new(discount_index)))?;

        index_store.duplicate_index(0, "discount_index_test_duplicated".to_string())?;
        index_store.duplicate_index(1, "discount_index_test_duplicated_2".to_string())?;

        // check if index exists
        assert_eq!(
            index_store.get_index(0)?.read_index()?.name()?,
            "discount_index_test"
        );
        assert_eq!(
            index_store.get_index(1)?.read_index()?.name()?,
            "discount_index_test_duplicated"
        );
        assert_eq!(
            index_store.get_index(2)?.read_index()?.name()?,
            "discount_index_test_duplicated_2"
        );

        // check fixings
        let fixings_out = index_store.get_index(0)?.read_index()?.fixings().clone();
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 31)).unwrap(), &0.12);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 30)).unwrap(), &0.13);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 29)).unwrap(), &0.14);

        let fixings_out = index_store.get_index(1)?.read_index()?.fixings().clone();
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 31)).unwrap(), &0.12);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 30)).unwrap(), &0.13);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 29)).unwrap(), &0.14);

        let fixings_out = index_store.get_index(2)?.read_index()?.fixings().clone();
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 31)).unwrap(), &0.12);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 30)).unwrap(), &0.13);
        assert_eq!(fixings_out.get(&Date::new(2019, 12, 29)).unwrap(), &0.14);

        Ok(())
    }

    #[test]
    fn test_advance_currency_curves_map() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let mut index_store = IndexStore::new(ref_date);

        index_store.add_currency_curve(Currency::BRL, 1);
        index_store.add_currency_curve(Currency::CLP, 2);

        let index_store = index_store.advance_to_period(Period::new(1, TimeUnit::Days))?;

        assert!(index_store.reference_date() == Date::new(2020, 1, 2));

        let currency_curves = index_store.get_currency_curve(Currency::CLP)?;
        assert!(currency_curves == 2);

        let currency_curves = index_store.get_currency_curve(Currency::BRL)?;
        assert!(currency_curves == 1);

        let index_store = index_store.advance_to_period(Period::new(1, TimeUnit::Years))?;

        assert!(index_store.reference_date() == Date::new(2021, 1, 2));

        let currency_curves = index_store.get_currency_curve(Currency::CLP)?;
        assert!(currency_curves == 2);

        let currency_curves = index_store.get_currency_curve(Currency::BRL)?;
        assert!(currency_curves == 1);

        Ok(())
    }

    #[test]
    fn test_synthetic_term_structure_1() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let mut index_store = IndexStore::new(ref_date);
        
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                dates,
                discount_factors,
                day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let discount_index = IborIndex::new(ref_date)
            .with_term_structure(discount_term_structure.clone())
            .with_name(Some("discount_index_test_1".to_string())); 
        index_store.add_index(0, Arc::new(RwLock::new(discount_index)))?;

        // create new term structure
        let new_dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let new_discount_factors = vec![1.0, 0.89, 0.88, 0.87, 0.86];
        let new_day_counter = DayCounter::Actual360;

        let new_discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                new_dates,
                new_discount_factors,
                new_day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let new_discount_index = IborIndex::new(ref_date)
            .with_term_structure(new_discount_term_structure)
            .with_name(Some("discount_index_test_2".to_string()));

        index_store.add_index(1, Arc::new(RwLock::new(new_discount_index)))?;

        index_store.add_synthetic_index(vec![0], vec![1], String::from("synthetic_index_test"), None, Currency::USD)?;

        let synthetic_index = index_store.get_index(2)?;
        let synthetic_index = synthetic_index.read_index()?;

        let df = synthetic_index.discount_factor(Date::new(2020, 4, 1))?;
        println!("df: {:?}", df);
        assert!((df - 0.99/0.89).abs() < 0.00001);

        Ok(())
    }

    
    #[test]
    fn test_synthetic_term_structure_2() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let mut index_store = IndexStore::new(ref_date);
        
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                dates,
                discount_factors,
                day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let discount_index = IborIndex::new(ref_date)
            .with_term_structure(discount_term_structure.clone())
            .with_name(Some("discount_index_test_1".to_string())); 
        index_store.add_index(0, Arc::new(RwLock::new(discount_index)))?;

        // create new term structure
        let new_dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let new_discount_factors = vec![1.0, 0.89, 0.88, 0.87, 0.86];
        let new_day_counter = DayCounter::Actual360;

        let new_discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                new_dates,
                new_discount_factors,
                new_day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let new_discount_index = IborIndex::new(ref_date)
            .with_term_structure(new_discount_term_structure)
            .with_name(Some("discount_index_test_2".to_string()));

        index_store.add_index(1, Arc::new(RwLock::new(new_discount_index)))?;

        index_store.add_synthetic_index(vec![0, 1], vec![], String::from("synthetic_index_test"), None, Currency::USD)?;

        let synthetic_index = index_store.get_index(2)?;
        let synthetic_index = synthetic_index.read_index()?;

        let df = synthetic_index.discount_factor(Date::new(2020, 4, 1))?;
        println!("df: {:?}", df);
        assert!((df - 0.99*0.89).abs() < 0.00001);

        Ok(())
    }

    #[test]
    fn test_synthetic_term_structure_3() -> Result<()> {
        let ref_date = Date::new(2020, 1, 1);
        let mut index_store = IndexStore::new(ref_date);
        
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                dates,
                discount_factors,
                day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let discount_index = IborIndex::new(ref_date)
            .with_term_structure(discount_term_structure.clone())
            .with_name(Some("discount_index_test_1".to_string())); 
        index_store.add_index(0, Arc::new(RwLock::new(discount_index)))?;

        // create new term structure
        let new_dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let new_discount_factors = vec![1.0, 0.89, 0.88, 0.87, 0.86];
        let new_day_counter = DayCounter::Actual360;

        let new_discount_term_structure = Arc::new(
            DiscountTermStructure::new(
                new_dates,
                new_discount_factors,
                new_day_counter,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let new_discount_index = IborIndex::new(ref_date)
            .with_term_structure(new_discount_term_structure)
            .with_name(Some("discount_index_test_2".to_string()));

        index_store.add_index(1, Arc::new(RwLock::new(new_discount_index)))?;

        index_store.add_synthetic_index(vec![], vec![1, 0], String::from("synthetic_index_test"), None, Currency::USD)?;

        let synthetic_index = index_store.get_index(2)?;
        let synthetic_index = synthetic_index.read_index()?;

        let df = synthetic_index.discount_factor(Date::new(2020, 4, 1))?;
        println!("df: {:?}", df);
        assert!((df - 1.0 / (0.99*0.89)).abs() < 0.00001);

        Ok(())
    }

}
