use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};

use crate::{
    core::meta::{DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest, MarketRequest},
    currencies::enums::Currency,
    math::interpolation::enums::Interpolator,
    models::traits::Model,
    rates::{
        enums::Compounding,
        interestrate::InterestRate,
        traits::{HasReferenceDate, YieldProvider},
    },
    time::{date::Date, daycounter::DayCounter, enums::Frequency},
    utils::errors::{AtlasError, Result},
};

/// # BootstrappingMarketStore
/// BootstrappingMarketStore is a market store used during the bootstrapping process.
/// It holds the reference date, local currency, curves map, currency curves map, and exchange rates map.
/// It provides methods to add curves, get discount factors, forward rates, and exchange rates.
/// ## Fields
/// * `reference_date` - The reference date of the market store
/// * `local_currency` - The local currency of the market store
/// * `curves_map` - A map of curves indexed by their IDs
/// * `currency_curve` - A map of curves used por currency projections if is needed
/// * `exchange_rate_map` - A map of exchange rates between currencies
/// * `exchange_rate_cache` - A cache for exchange rates to avoid recomputation
pub struct BootstrappingMarketStore {
    reference_date: Date,
    local_currency: Currency,
    curves_map: HashMap<usize, BootstrappingCurve>,
    currency_curve: HashMap<Currency, usize>,
    exchange_rate_map: HashMap<(Currency, Currency), f64>,
    exchange_rate_cache: Arc<Mutex<HashMap<(Currency, Currency), f64>>>,
}

impl BootstrappingMarketStore {
    pub fn new(reference_date: Date, local_currency: Currency) -> BootstrappingMarketStore {
        BootstrappingMarketStore {
            reference_date,
            local_currency,
            curves_map: HashMap::new(),
            currency_curve: HashMap::new(),
            exchange_rate_map: HashMap::new(),
            exchange_rate_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn local_currency(&self) -> Currency {
        self.local_currency
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

    pub fn with_exchange_rates(
        &mut self,
        exchange_rate_map: HashMap<(Currency, Currency), f64>,
    ) -> &mut Self {
        self.exchange_rate_map = exchange_rate_map;
        self
    }

    pub fn add_exchange_rate(&mut self, currency1: Currency, currency2: Currency, rate: f64) {
        self.exchange_rate_map.insert((currency1, currency2), rate);
    }

    pub fn get_exchange_rate_map(&self) -> HashMap<(Currency, Currency), f64> {
        self.exchange_rate_map.clone()
    }

    pub fn get_exchange_rate(&self, first_ccy: Currency, second_ccy: Currency) -> Result<f64> {
        let first_ccy = first_ccy;
        let second_ccy = second_ccy;

        if first_ccy == second_ccy {
            return Ok(1.0);
        }

        let cache_key = (first_ccy, second_ccy);
        if let Some(cached_rate) = self.exchange_rate_cache.lock().unwrap().get(&cache_key) {
            return Ok(*cached_rate);
        }

        let mut q: VecDeque<(Currency, f64)> = VecDeque::new();
        let mut visited: HashSet<Currency> = HashSet::new();
        q.push_back((first_ccy, 1.0));
        visited.insert(first_ccy);

        let mut mutable_cache = self.exchange_rate_cache.lock().unwrap();
        while let Some((current_ccy, rate)) = q.pop_front() {
            for (&(source, dest), &map_rate) in &self.exchange_rate_map {
                if source == current_ccy && !visited.contains(&dest) {
                    let new_rate = rate * map_rate;
                    if dest == second_ccy {
                        mutable_cache.insert((first_ccy, second_ccy), new_rate);
                        mutable_cache.insert((second_ccy, first_ccy), 1.0 / new_rate);
                        return Ok(new_rate);
                    }
                    visited.insert(dest);
                    q.push_back((dest, new_rate));
                } else if dest == current_ccy && !visited.contains(&source) {
                    let new_rate = rate / map_rate;
                    if source == second_ccy {
                        mutable_cache.insert((first_ccy, second_ccy), new_rate);
                        mutable_cache.insert((second_ccy, first_ccy), 1.0 / new_rate);
                        return Ok(new_rate);
                    }
                    visited.insert(source);
                    q.push_back((source, new_rate));
                }
            }
        }
        Err(AtlasError::NotFoundErr(format!(
            "No exchange rate found between {:?} and {:?}",
            first_ccy, second_ccy
        )))
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

        let first_curve = self
            .curves_map
            .get(&first_id)
            .ok_or(AtlasError::BootstrappingErr(format!(
                "Curve with id {} not found",
                first_id
            )))?;

        let second_curve = self
            .curves_map
            .get(&second_id)
            .ok_or(AtlasError::BootstrappingErr(format!(
                "Curve with id {} not found",
                first_id
            )))?;

        let first_df = first_curve.discount_factor(date)?;
        let second_df = second_curve.discount_factor(date)?;

        Ok(second_df / first_df)
    }
}

impl BootstrappingMarketStore {
    pub fn add_curve(&mut self, id: usize, currency: Currency) -> Result<()> {
        if self.curves_map.contains_key(&id) {
            return Err(AtlasError::BootstrappingErr(format!(
                "Curve with id {} already exists",
                id
            )));
        }
        let curve = BootstrappingCurve::new(self.reference_date, id, currency);
        self.curves_map.insert(id, curve);
        Ok(())
    }

    pub fn curves_map(&self) -> &HashMap<usize, BootstrappingCurve> {
        &self.curves_map
    }

    pub fn curves_map_mut(&mut self) -> &mut HashMap<usize, BootstrappingCurve> {
        &mut self.curves_map
    }
}

/// # BootstrappingCurve
/// Represents a simple representation of a curve constructed during the bootstrapping process.
/// It holds the reference date, id, currency, dates, year fractions, discount factors, and other related data.
/// It implements the `HasReferenceDate` and `YieldProvider` traits to provide discount factors and forward rates.
///
/// ## Fields
/// * `reference_date` - The reference date of the curve
/// * `id` - The unique identifier of the curve
/// * `currency` - The currency of the curve
/// * `dates` - The dates of the curve
/// * `year_fractions` - The year fractions corresponding to the dates
/// * `discount_factors` - The discount factors corresponding to the dates
/// * `related_instrument_index` - The index of the discount factors for each date
/// * `day_counter` - The day counter used for calculating year fractions
/// * `interpolator` - The interpolator used for calculating discount factors
/// * `enable_extrapolation` - Whether to enable extrapolation for the interpolator
pub struct BootstrappingCurve {
    reference_date: Date,
    id: usize,
    currency: Currency,
    dates: Vec<Date>,
    year_fractions: Vec<f64>,
    discount_factors: Vec<f64>,
    related_instrument_index: Vec<Option<usize>>,
    day_counter: DayCounter,
    interpolator: Interpolator,
    enable_extrapolation: bool,
}

impl BootstrappingCurve {
    pub fn new(reference_date: Date, id: usize, currency: Currency) -> Self {
        BootstrappingCurve {
            reference_date,
            id,
            currency,
            dates: vec![reference_date],
            year_fractions: vec![0.0], // Start with 0.0 for the reference date
            discount_factors: vec![1.0], // Start with 1.0 for the reference date
            related_instrument_index: vec![None], // Index for the reference date
            day_counter: DayCounter::Actual360, // Default day counter
            interpolator: Interpolator::LogLinear, // Default interpolator
            enable_extrapolation: true, // Default to true
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn dates(&self) -> &Vec<Date> {
        &self.dates
    }

    pub fn year_fractions(&self) -> &Vec<f64> {
        &self.year_fractions
    }

    pub fn discount_factors(&self) -> &Vec<f64> {
        &self.discount_factors
    }

    pub fn discount_factors_mut(&mut self) -> &mut Vec<f64> {
        &mut self.discount_factors
    }

    pub fn related_instrument_index(&self) -> &Vec<Option<usize>> {
        &self.related_instrument_index
    }

    pub fn day_counter(&self) -> &DayCounter {
        &self.day_counter
    }

    pub fn interpolator(&self) -> &Interpolator {
        &self.interpolator
    }

    pub fn enable_extrapolation(&self) -> bool {
        self.enable_extrapolation
    }

    pub fn set_interpolator(&mut self, interpolator: Interpolator) {
        self.interpolator = interpolator;
    }

    pub fn set_day_counter(&mut self, day_counter: DayCounter) {
        self.day_counter = day_counter;
    }

    pub fn set_enable_extrapolation(&mut self, enable: bool) {
        self.enable_extrapolation = enable;
    }

    pub fn add_date(&mut self, date: Date, related_instrument_index: usize) -> Result<()> {
        if date < self.reference_date {
            return Err(AtlasError::BootstrappingErr(
                "Date in bootstrapping curve must be after the reference date".to_string(),
            ));
        }

        let year_fraction = self.day_counter.year_fraction(self.reference_date, date);

        match self.dates.binary_search(&date) {
            Ok(_) => {
                return Err(AtlasError::BootstrappingErr(format!(
                    "Date {} already exists in bootstrapping curve",
                    date
                )));
            }
            Err(pos) => {
                self.dates.insert(pos, date);
                self.discount_factors.insert(pos, 1.0); // Initialize with a default value 1.0
                self.year_fractions.insert(pos, year_fraction);
                self.related_instrument_index
                    .insert(pos, Some(related_instrument_index));
            }
        }
        Ok(())
    }
}

impl HasReferenceDate for BootstrappingCurve {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
}

impl YieldProvider for BootstrappingCurve {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        if date < self.reference_date() {
            return Err(AtlasError::BootstrappingErr(format!(
                "Date {} needs to be greater than reference date {}",
                date,
                self.reference_date()
            )));
        }
        if date == self.reference_date() {
            return Ok(1.0);
        }

        let year_fraction = self
            .day_counter()
            .year_fraction(self.reference_date(), date);

        let discount_factor = self.interpolator.interpolate(
            year_fraction,
            &self.year_fractions,
            &self.discount_factors,
            true,
        );
        return Ok(discount_factor);
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<f64> {
        let discount_factor_to_star = self.discount_factor(start_date)?;
        let discount_factor_to_end = self.discount_factor(end_date)?;

        let comp_factor = discount_factor_to_star / discount_factor_to_end;
        let t = self.day_counter().year_fraction(start_date, end_date);

        if comp_factor <= 0.0 {
            return Ok(0.0);
        }

        return Ok(
            InterestRate::implied_rate(comp_factor, *self.day_counter(), comp, freq, t)?.rate(),
        );
    }
}

/// # BootstrappingModel
/// BootstrappingModel is a model that uses the BootstrappingMarketStore to provide discount factors, forward rates, and exchange rates.
#[derive(Clone)]
pub struct BootstrappingModel<'a> {
    market_store: &'a BootstrappingMarketStore,
}

impl<'a> BootstrappingModel<'a> {
    pub fn new(market_store: &'a BootstrappingMarketStore) -> BootstrappingModel {
        BootstrappingModel { market_store }
    }
}

impl<'a> Model for BootstrappingModel<'a> {
    fn reference_date(&self) -> Date {
        self.market_store.reference_date()
    }

    fn gen_df_data(&self, df_request: DiscountFactorRequest) -> Result<f64> {
        let date = df_request.date();
        let ref_date = self.market_store.reference_date();

        // eval today or before ref date
        if ref_date > date {
            return Ok(0.0);
        } else if ref_date == date {
            return Ok(1.0);
        }

        let id = df_request.provider_id();
        let curve = self
            .market_store
            .curves_map()
            .get(&id)
            .ok_or(AtlasError::BootstrappingErr(format!(
                "Curve with id {} not found in bootstrapping market store",
                id
            )))?;

        let df = curve.discount_factor(date)?;

        let currency_forescast_factor = match df_request.discount_currency() {
            Some(currency) => {
                self.market_store
                    .currency_forescast_factor(curve.currency(), currency, date)?
            }
            None => 1.0,
        };

        Ok(df * currency_forescast_factor)
    }

    fn gen_fwd_data(&self, fwd: ForwardRateRequest) -> Result<f64> {
        let id = fwd.provider_id();
        let end_date = fwd.end_date();
        let ref_date = self.market_store.reference_date();
        if end_date <= ref_date {
            return Ok(0.0);
        }

        let fwd_rate_provider =
            self.market_store
                .curves_map()
                .get(&id)
                .ok_or(AtlasError::BootstrappingErr(format!(
                    "Curve with id {} not found in bootstrapping market store",
                    id
                )))?;

        let start_date = fwd.start_date();
        Ok(fwd_rate_provider.forward_rate(
            start_date,
            end_date,
            fwd.compounding(),
            fwd.frequency(),
        )?)
    }

    fn gen_fx_data(&self, fx: ExchangeRateRequest) -> Result<f64> {
        let first_currency = fx.first_currency();
        let second_currency = match fx.second_currency() {
            Some(ccy) => ccy,
            None => self.market_store.local_currency(),
        };

        if first_currency == second_currency {
            return Ok(1.0);
        }

        let spot = self
            .market_store
            .get_exchange_rate(first_currency, second_currency)?;

        match fx.reference_date() {
            Some(date) => {
                if date > self.reference_date() {
                    let currency_forescast_factor = self.market_store.currency_forescast_factor(
                        first_currency,
                        second_currency,
                        date,
                    )?;
                    Ok(spot * currency_forescast_factor)
                } else {
                    Ok(spot)
                }
            }
            None => Ok(spot),
        }
    }

    fn gen_numerarie(&self, _: &MarketRequest) -> Result<f64> {
        Ok(1.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        currencies::enums::Currency,
        rates::bootstrapping::bootstrappingmarketstore::{
            BootstrappingCurve, BootstrappingMarketStore,
        },
        time::date::Date,
        utils::errors::Result,
    };

    #[test]
    fn test_bootstrapping_curve_add_date() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut curve = BootstrappingCurve::new(ref_date, 1, Currency::USD);
        let date1 = Date::new(2023, 1, 1);
        let date2 = Date::new(2023, 6, 1);
        let date3 = Date::new(2023, 2, 1);

        curve.add_date(date1, 0)?;
        curve.add_date(date2, 1)?;
        curve.add_date(date3, 2)?;

        assert_eq!(curve.dates, vec![ref_date, date1, date3, date2]);
        assert_eq!(curve.discount_factors, vec![1.0, 1.0, 1.0, 1.0]);
        assert_eq!(
            curve.related_instrument_index,
            vec![None, Some(0), Some(2), Some(1)]
        );

        Ok(())
    }

    #[test]
    fn test_new_bootstrapping_index_store() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let store = BootstrappingMarketStore::new(ref_date, Currency::USD);
        assert_eq!(store.reference_date(), ref_date);
        assert!(store.curves_map.is_empty());
        assert!(store.currency_curve.is_empty());
        Ok(())
    }

    #[test]
    fn test_add_currency_curve() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut store = BootstrappingMarketStore::new(ref_date, Currency::USD);
        store.add_currency_curve(Currency::USD, 1);
        assert_eq!(store.get_currency_curve(Currency::USD)?, 1);
        Ok(())
    }

    #[test]
    fn test_add_curve_at_index() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut store = BootstrappingMarketStore::new(ref_date, Currency::USD);
        store.add_curve(1, Currency::USD)?;
        assert!(store.curves_map.contains_key(&1));
        assert_eq!(store.curves_map[&1].currency(), Currency::USD);
        Ok(())
    }
}
