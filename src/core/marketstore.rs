use crate::currencies::enums::Currency;
use crate::time::date::Date;
use crate::time::enums::TimeUnit;
use crate::time::period::Period;
use core::fmt;
use std::sync::{Arc, RwLock};

use crate::{
    currencies::{
        exchangeratestore::ExchangeRateStore,
        traits::{AdvanceExchangeRateStoreInTime, CurrencyDetails},
    },
    rates::{
        indexstore::{IndexStore, ReadIndex},
        interestrateindex::traits::InterestRateIndexTrait,
        traits::HasReferenceDate,
    },
    utils::errors::{AtlasError, Result},
};

/// # MarketStore
/// A store for market data.
///
/// ## Parameters
/// * `reference_date` - The reference date of the market store
/// * `local_currency` - The local currency of the market store
/// * `exchange_rate_store` - The exchange rate store
/// * `index_store` - The index store
#[derive(Clone)]
pub struct MarketStore {
    reference_date: Date,
    local_currency: Currency,
    exchange_rate_store: ExchangeRateStore,
    index_store: IndexStore,
}

impl MarketStore {
    pub fn new(reference_date: Date, local_currency: Currency) -> MarketStore {
        MarketStore {
            reference_date,
            local_currency,
            exchange_rate_store: ExchangeRateStore::new(reference_date),
            index_store: IndexStore::new(reference_date),
        }
    }

    pub fn set_exchange_rate_store(
        &mut self,
        exchange_rate_store: ExchangeRateStore,
    ) -> Result<()> {
        if exchange_rate_store.reference_date() != self.reference_date {
            return Err(AtlasError::InvalidValueErr(format!(
                "Exchange rate store reference date {} does not match market store reference date {}",
                exchange_rate_store.reference_date(),
                self.reference_date
            )));
        }
        self.exchange_rate_store = exchange_rate_store;
        Ok(())
    }

    pub fn set_index_store(&mut self, index_store: IndexStore) -> Result<()> {
        if index_store.reference_date() != self.reference_date {
            return Err(AtlasError::InvalidValueErr(format!(
                "Index store reference date {} does not match market store reference date {}",
                index_store.reference_date(),
                self.reference_date
            )));
        }
        self.index_store = index_store;
        Ok(())
    }

    pub fn local_currency(&self) -> Currency {
        self.local_currency
    }

    pub fn set_local_currency(&mut self, local_currency: Currency) {
        self.local_currency = local_currency;
    }

    pub fn exchange_rate_store(&self) -> &ExchangeRateStore {
        &self.exchange_rate_store
    }

    pub fn mut_exchange_rate_store(&mut self) -> &mut ExchangeRateStore {
        &mut self.exchange_rate_store
    }

    pub fn index_store(&self) -> &IndexStore {
        &self.index_store
    }

    pub fn mut_index_store(&mut self) -> &mut IndexStore {
        &mut self.index_store
    }

    pub fn get_exchange_rate(
        &self,
        first_currency: Currency,
        second_currency: Option<Currency>,
    ) -> Result<f64> {
        let second_currency = match second_currency {
            Some(ccy) => ccy,
            None => self.local_currency,
        };
        return self
            .exchange_rate_store
            .get_exchange_rate(first_currency, second_currency);
    }

    pub fn get_index(&self, id: usize) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>> {
        return self.index_store.get_index(id);
    }

    pub fn advance_to_period(&self, period: Period) -> Result<MarketStore> {
        if period.length() < 0 {
            return Err(AtlasError::InvalidValueErr(format!(
                "Negative periods are not allowed when advancing market store in time ({:?})",
                period
            )));
        }
        let new_reference_date = self.reference_date + period;
        let new_exchange_rate_store = self
            .exchange_rate_store
            .advance_to_period(period, &self.index_store)?;
        let new_index_store = self.index_store.advance_to_period(period)?;

        Ok(MarketStore {
            reference_date: new_reference_date,
            local_currency: self.local_currency,
            exchange_rate_store: new_exchange_rate_store,
            index_store: new_index_store,
        })
    }

    pub fn advance_to_date(&self, date: Date) -> Result<MarketStore> {
        if date < self.reference_date {
            return Err(AtlasError::InvalidValueErr(format!(
                "Date {} is before reference date {}",
                date, self.reference_date
            )));
        }
        let days = (date - self.reference_date) as i32;
        let period = Period::new(days, TimeUnit::Days);
        self.advance_to_period(period)
    }

    /// Get the pillar dates of a term structure by index ID
    pub fn get_pillar_dates(&self, id: usize) -> Result<Vec<Date>> {
        self.index_store.get_pillar_dates(id)
    }
}

// Implement HasReferenceDate for MarketStore
impl HasReferenceDate for MarketStore {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
}

// Implement Display for MarketStore
use colored::*; // Import the colored crate
impl fmt::Display for MarketStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\n{}",
            "=====================================".blue().bold()
        )?;
        writeln!(
            f,
            "{}",
            "======= MarketStore features! =======".blue().bold()
        )?;
        writeln!(
            f,
            "{}",
            "=====================================".blue().bold()
        )?;
        writeln!(
            f,
            "{} {}",
            "> Reference Date:".green().bold(),
            self.reference_date
        )?;
        writeln!(
            f,
            "{}",
            "-------------------------------------".blue().bold()
        )?;
        writeln!(
            f,
            "{} {}",
            "> Currency:".green().bold(),
            self.local_currency.code()
        )?;
        writeln!(
            f,
            "{}",
            "-------------------------------------".blue().bold()
        )?;

        let index_store = self.index_store();
        let all_indices = index_store.get_all_indices();
        let indices_map = index_store.get_index_map().unwrap();

        let mut indices_names: Vec<(String, usize)> = all_indices
            .iter()
            .filter_map(|indice| {
                let ind = indice.read_index().ok()?;
                let indice_long_name = ind.name_long_detail().ok()?;
                let indice_name = ind.name().ok()?;
                let id = indices_map.get(&indice_name)?;
                Some((indice_long_name, *id))
            })
            .collect();

        indices_names.sort_by(|a, b| a.1.cmp(&b.1));

        writeln!(
            f,
            "{} ({})",
            "> Indices:".green().bold(),
            indices_names.len()
        )?;
        for (indice_name, id) in indices_names {
            writeln!(
                f,
                "  >> {} -> {}",
                id.to_string().yellow().bold(),
                indice_name.cyan()
            )?;
        }

        let exchange_rate_store = self.exchange_rate_store();
        let exchange_rate_map = exchange_rate_store.get_exchange_rate_map();
        writeln!(
            f,
            "{}",
            "-------------------------------------".blue().bold()
        )?;
        writeln!(
            f,
            "{} ({})",
            "> Currency pairs:".green().bold(),
            exchange_rate_map.len()
        )?;
        for (currencies, value) in exchange_rate_map {
            writeln!(
                f,
                "  >> {} -> {}: {}",
                currencies.0.code().yellow().bold(),
                currencies.1.code().yellow().bold(),
                value.to_string().magenta()
            )?;
        }

        writeln!(
            f,
            "{}",
            "=====================================".blue().bold()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_market_store() -> MarketStore {
        let reference_date = Date::new(2024, 1, 1);
        let local_currency = Currency::USD;
        MarketStore::new(reference_date, local_currency)
    }

    #[test]
    fn test_new_market_store() {
        let reference_date = Date::new(2024, 1, 1);
        let local_currency = Currency::USD;
        let market_store = MarketStore::new(reference_date, local_currency);

        assert_eq!(market_store.reference_date(), reference_date);
        assert_eq!(market_store.local_currency(), local_currency);
    }

    #[test]
    fn test_get_exchange_rate_local_currency() {
        let mut market_store = setup_market_store();
        // Add a fake exchange rate for testing
        market_store
            .mut_exchange_rate_store()
            .add_exchange_rate(Currency::USD, Currency::EUR, 1.1);
        let rate = market_store
            .get_exchange_rate(Currency::USD, Some(Currency::EUR))
            .unwrap();
        assert!((rate - 1.1).abs() < 1e-8);
    }

    #[test]
    fn test_get_exchange_rate_default_to_local() {
        let mut market_store = setup_market_store();
        market_store
            .mut_exchange_rate_store()
            .add_exchange_rate(Currency::EUR, Currency::USD, 1.2);
        let rate = market_store.get_exchange_rate(Currency::EUR, None).unwrap();
        assert!((rate - 1.2).abs() < 1e-8);
    }

    #[test]
    fn test_advance_to_period_positive() {
        let market_store = setup_market_store();
        let period = Period::new(10, TimeUnit::Days);
        let advanced = market_store.advance_to_period(period).unwrap();
        assert_eq!(
            advanced.reference_date(),
            market_store.reference_date() + period
        );
    }

    #[test]
    fn test_advance_to_period_negative() {
        let market_store = setup_market_store();
        let period = Period::new(-5, TimeUnit::Days);
        let result = market_store.advance_to_period(period);
        assert!(result.is_err());
    }

    #[test]
    fn test_advance_to_date_future() {
        let market_store = setup_market_store();
        let new_date = market_store.reference_date() + Period::new(5, TimeUnit::Days);
        let advanced = market_store.advance_to_date(new_date).unwrap();
        assert_eq!(advanced.reference_date(), new_date);
    }

    #[test]
    fn test_advance_to_date_past() {
        let market_store = setup_market_store();
        let past_date = market_store.reference_date() - Period::new(1, TimeUnit::Days);
        let result = market_store.advance_to_date(past_date);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_trait() {
        let market_store = setup_market_store();
        let output = format!("{}", market_store);
        assert!(output.contains("MarketStore features"));
        assert!(output.contains("Reference Date"));
        assert!(output.contains("Currency"));
    }
}
