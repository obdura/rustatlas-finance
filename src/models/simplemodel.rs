use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    core::{
        marketstore::MarketStore,
        meta::{DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest},
    },
    currencies::{enums::Currency, exchangerategeneration::ExchangeGenerationMethod},
    rates::{indexstore::ReadIndex, traits::HasReferenceDate},
    time::date::Date,
    utils::errors::{AtlasError, Result},
};

use super::traits::Model;

/// # SimpleModel
/// A simple model that provides market data based on the current market state. Uses the
/// market store to get the market data (curves, currencies and others). All values are calculated using the
/// reference date and local currency of the market store.
///
/// ## Parameters
/// * `market_store` - The market store.
/// * `transform_currencies` - If true, the model will transform the currencies to the local currency of the market store.
#[derive(Clone)]
pub struct SimpleModel<'a> {
    market_store: &'a MarketStore,
    discount_factors_cache: Arc<RwLock<HashMap<DiscountFactorRequest, f64>>>,
    forward_rates_cache: Arc<RwLock<HashMap<ForwardRateRequest, f64>>>,
    fx_cache: Arc<RwLock<HashMap<ExchangeRateRequest, f64>>>,
}

impl<'a> SimpleModel<'a> {
    pub fn new(market_store: &'a MarketStore) -> SimpleModel<'a> {
        SimpleModel {
            market_store,
            discount_factors_cache: Arc::new(RwLock::new(HashMap::new())),
            forward_rates_cache: Arc::new(RwLock::new(HashMap::new())),
            fx_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn discount_factors_cache(&self) -> &Arc<RwLock<HashMap<DiscountFactorRequest, f64>>> {
        &self.discount_factors_cache
    }

    pub fn forward_rates_cache(&self) -> &Arc<RwLock<HashMap<ForwardRateRequest, f64>>> {
        &self.forward_rates_cache
    }

    pub fn fx_cache(&self) -> &Arc<RwLock<HashMap<ExchangeRateRequest, f64>>> {
        &self.fx_cache
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.discount_factors_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.forward_rates_cache.write() {
            cache.clear();
        }
        if let Ok(mut cache) = self.fx_cache.write() {
            cache.clear();
        }
    }

    pub fn currency_parity(
        &self,
        first_currency: Currency,
        second_currency: Currency,
        date: Date,
        history: bool,
    ) -> Result<f64> {
        if first_currency == second_currency {
            return Ok(1.0);
        }

        let spot = self
            .market_store
            .exchange_rate_store()
            .get_exchange_rate(first_currency, second_currency)?;

        if date == self.reference_date() {
            return Ok(spot);
        }

        if date > self.reference_date() {
            let currency_forescast_factor = self
                .market_store
                .index_store()
                .currency_forescast_factor(first_currency, second_currency, date)?;
            return Ok(spot * currency_forescast_factor);
        }

        if date < self.reference_date() {
            if history {
                return self
                    .market_store
                    .exchange_rate_store()
                    .get_exchange_rate_history(first_currency, second_currency, date);
            } else {
                return Err(AtlasError::InvalidValueErr(
                    "Date is before reference date".to_string(),
                ));
            }
        }

        return Ok(0.0);
    }
}

impl<'a> Model for SimpleModel<'a> {
    fn reference_date(&self) -> Date {
        self.market_store.reference_date()
    }

    fn gen_df_data(&self, df_request: &DiscountFactorRequest) -> Result<f64> {
        // 1.- Check cache and return if hit
        if let Some(hit) = {
            self.discount_factors_cache
                .read()
                .map_err(|_| AtlasError::PoisonedCacheErr("discount_factors_cache".to_string()))?
                .get(df_request)
                .copied()
        } {
            return Ok(hit);
        }

        // 2.- no cache hit, gen data
        let date = df_request.date();
        let ref_date = self.market_store.reference_date();

        let computed = if ref_date > date {
            0.0
        } else if ref_date == date {
            1.0
        } else {
            let id = df_request.provider_id();
            let index = self.market_store.get_index(id)?;
            let curve = index.read_index()?.term_structure()?;
            let df = curve.discount_factor(date)?;

            let currency_forescast_factor = match df_request.discount_currency() {
                Some(currency) => match index.read_index()?.currency()? {
                    Some(currency_index) => self
                        .market_store
                        .index_store()
                        .currency_forescast_factor(currency_index, currency, date)?,
                    None => 1.0,
                },
                None => 1.0,
            };
            df * currency_forescast_factor
        };

        // 3.- Cache data
        let mut cache = self
            .discount_factors_cache
            .write()
            .map_err(|_| AtlasError::PoisonedCacheErr("discount_factors_cache".to_string()))?;
        let entry = cache.entry(df_request.clone()).or_insert(computed);
        Ok(*entry)
    }

    fn gen_fwd_data(&self, fwd: &ForwardRateRequest) -> Result<f64> {
        // 1.- Check cache and return if hit
        if let Some(hit) = {
            self.forward_rates_cache
                .read()
                .map_err(|_| AtlasError::PoisonedCacheErr("forward_rates_cache".to_string()))?
                .get(fwd)
                .copied()
        } {
            return Ok(hit);
        }

        // 2.- no cache hit, gen data
        let ref_date = self.market_store.reference_date();
        let computed = if fwd.end_date() <= ref_date {
            0.0
        } else {
            let index = self.market_store.get_index(fwd.provider_id())?;
            let index_guard = index.read_index()?;
            index_guard.forward_rate(
                fwd.start_date(),
                fwd.end_date(),
                fwd.compounding(),
                fwd.frequency(),
                fwd.day_counter(),
            )?
        };

        // 3.- Cache data
        let mut cache = self
            .forward_rates_cache
            .write()
            .map_err(|_| AtlasError::PoisonedCacheErr("forward_rates_cache".into()))?;
        let entry = cache.entry(fwd.clone()).or_insert(computed);
        Ok(*entry)
    }

    fn gen_fx_data(&self, fx: &ExchangeRateRequest) -> Result<f64> {
        // 1.- Check cache and return if hit
        if let Some(hit) = {
            self.fx_cache
                .read()
                .map_err(|_| AtlasError::PoisonedCacheErr("fx_cache".into()))?
                .get(fx)
                .copied()
        } {
            return Ok(hit);
        }

        // 2.- no cache hit, gen data
        let first_ccy = fx.first_currency();
        let second_ccy = fx
            .second_currency()
            .unwrap_or_else(|| self.market_store.local_currency());

        let computed = if first_ccy == second_ccy {
            1.0
        } else {
            match fx.generation_method() {
                Some(ExchangeGenerationMethod::SingleDate(single)) => {
                    self.currency_parity(first_ccy, second_ccy, single.date(), false)?
                }
                Some(ExchangeGenerationMethod::DateWindow(window)) => {
                    let total = window
                        .date_window()
                        .iter()
                        .map(|date| self.currency_parity(first_ccy, second_ccy, *date, true))
                        .collect::<Result<Vec<f64>>>()?
                        .into_iter()
                        .sum::<f64>();
                    total / window.number_of_dates() as f64
                }
                Some(ExchangeGenerationMethod::Triangulation(tri)) => {
                    let pivot = tri.triangulation_curency();
                    let first = self.currency_parity(first_ccy, pivot, tri.first_date(), true)?;
                    let second =
                        self.currency_parity(second_ccy, pivot, tri.second_date(), true)?;
                    first / second
                }
                None => self
                    .market_store
                    .exchange_rate_store()
                    .get_exchange_rate(first_ccy, second_ccy)?,
            }
        };

        let mut cache = self
            .fx_cache
            .write()
            .map_err(|_| AtlasError::PoisonedCacheErr("fx_cache".into()))?;
        let entry = cache.entry(fx.clone()).or_insert(computed);
        Ok(*entry)
    }

    // fn gen_numerarie(&self, _: &MarketRequest) -> Result<f64> {
    //     Ok(1.0)
    // }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use crate::{
        core::{
            marketstore::MarketStore,
            meta::{DiscountFactorRequest, ExchangeRateRequest},
        },
        currencies::{
            enums::Currency,
            exchangerategeneration::{
                DateWindow, ExchangeGenerationMethod, SingleDate, Triangulation,
            },
        },
        models::{simplemodel::SimpleModel, traits::Model},
        rates::{
            interestrate::RateDefinition, interestrateindex::iborindex::IborIndex,
            traits::HasReferenceDate,
            yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
        },
        time::{
            date::Date,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
    };

    pub fn create_marketstore() -> Result<MarketStore> {
        let ref_date = Date::new(2020, 1, 1);
        let local_currency = Currency::CLP;
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

        let forecast_curve_3 = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            0.04,
            RateDefinition::default(),
        ));

        let ibor_index_1 = IborIndex::new(forecast_curve_1.reference_date())
            .with_term_structure(forecast_curve_1)
            .with_frequency(Frequency::Annual)?
            .with_currency(Some(Currency::USD))
            .with_name(Some("USD_curve".to_string()));

        let ibor_index_2 = IborIndex::new(forecast_curve_2.reference_date())
            .with_term_structure(forecast_curve_2)
            .with_frequency(Frequency::Annual)?
            .with_currency(Some(Currency::CLP))
            .with_name(Some("CLP_curve".to_string()));

        let ibor_index_3 = IborIndex::new(forecast_curve_3.reference_date())
            .with_term_structure(forecast_curve_3)
            .with_frequency(Frequency::Annual)?
            .with_currency(Some(Currency::CLF))
            .with_name(Some("CLF_curve".to_string()));

        market_store
            .mut_index_store()
            .add_index(0, Arc::new(RwLock::new(ibor_index_1)))?;

        market_store
            .mut_index_store()
            .add_index(1, Arc::new(RwLock::new(ibor_index_2)))?;

        market_store
            .mut_index_store()
            .add_index(2, Arc::new(RwLock::new(ibor_index_3)))?;

        market_store
            .mut_index_store()
            .add_currency_curve(Currency::USD, 0);

        market_store
            .mut_index_store()
            .add_currency_curve(Currency::CLP, 1);

        market_store
            .mut_index_store()
            .add_currency_curve(Currency::CLF, 2);

        let exchange_rate_store = market_store.mut_exchange_rate_store();
        exchange_rate_store.add_exchange_rate(Currency::CLP, Currency::USD, 800.0);
        exchange_rate_store.add_exchange_rate(Currency::CLP, Currency::CLF, 35_000.0);

        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::USD,
            ref_date - 1,
            799.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::USD,
            ref_date - 2,
            798.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::USD,
            ref_date - 3,
            797.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::USD,
            ref_date - 4,
            796.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::USD,
            ref_date - 5,
            795.0,
        );

        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::CLF,
            ref_date - 1,
            34_500.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::CLF,
            ref_date - 2,
            34_000.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::CLF,
            ref_date - 3,
            33_500.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::CLF,
            ref_date - 4,
            33_000.0,
        );
        exchange_rate_store.add_exchange_rate_history(
            Currency::CLP,
            Currency::CLF,
            ref_date - 5,
            32_500.0,
        );

        print!("{}", market_store);
        return Ok(market_store);
    }

    #[test]
    fn test_simple_mode_df_request() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);
        let current_date = market_store.reference_date();

        let date = current_date + Period::new(1, TimeUnit::Years);
        let df_request = DiscountFactorRequest::new(0, date);
        let df = model.gen_df_data(&df_request).unwrap();
        assert!((df - (1.0 / (1.0 + 0.02 * 366.0 / 360.0))).abs() < 1e-6);

        let mut df_request = DiscountFactorRequest::new(0, date);
        df_request.set_discount_currency(Currency::USD);
        let df = model.gen_df_data(&df_request).unwrap();
        assert!((df - (1.0 / (1.0 + 0.02 * 366.0 / 360.0))).abs() < 1e-6);

        let mut df_request = DiscountFactorRequest::new(0, date);
        df_request.set_discount_currency(Currency::CLP);
        let df = model.gen_df_data(&df_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::USD), Some(date));
        let fx_clp_usd = model.gen_fx_data(&fx_request).unwrap();
        assert!((fx_clp_usd * df - (1.0 / (1.0 + 0.02 * 366.0 / 360.0)) * 800.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_simple_model_fx_request() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);
        let date = Date::new(2021, 1, 1);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLP), Some(date));
        let fx_usd_clp = model.gen_fx_data(&fx_request).unwrap();
        assert!((fx_usd_clp - 0.00123766779880317).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::USD), Some(date));
        let fx_clp_usd = model.gen_fx_data(&fx_request).unwrap();
        assert!((fx_clp_usd * fx_usd_clp - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::CLF), Some(date));
        let fx_clp_clf = model.gen_fx_data(&fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::CLP), Some(date));
        let fx_clf_clp = model.gen_fx_data(&fx_request).unwrap();

        assert!((fx_clf_clp * fx_clp_clf - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLF), Some(date));
        let fx_usd_clf = model.gen_fx_data(&fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::USD), Some(date));
        let fx_clf_usd = model.gen_fx_data(&fx_request).unwrap();

        assert!((fx_clf_usd * fx_usd_clf - 1.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_simple_model_with_advance_marketstore() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let advace_date = Date::new(2020, 6, 1);
        let advance_matket_store = market_store.advance_to_date(advace_date)?;

        let model = SimpleModel::new(&advance_matket_store);
        let date = Date::new(2021, 1, 1);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLP), Some(date));
        let fx_usd_clp = model.gen_fx_data(&fx_request).unwrap();
        assert!((fx_usd_clp - 0.00123766779880317).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::USD), Some(date));
        let fx_clp_usd = model.gen_fx_data(&fx_request).unwrap();
        assert!((fx_clp_usd * fx_usd_clp - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::CLF), Some(date));
        let fx_clp_clf = model.gen_fx_data(&fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::CLP), Some(date));
        let fx_clf_clp = model.gen_fx_data(&fx_request).unwrap();

        assert!((fx_clf_clp * fx_clp_clf - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLF), Some(date));
        let fx_usd_clf = model.gen_fx_data(&fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::USD), Some(date));
        let fx_clf_usd = model.gen_fx_data(&fx_request).unwrap();

        assert!((fx_clf_usd * fx_usd_clf - 1.0).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_model_fx_with_triangulation() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);

        let request = ExchangeRateRequest::new(
            Currency::USD,
            Some(Currency::CLP),
            Some(Date::new(2022, 1, 1)),
        );
        let fx1 = model.gen_fx_data(&request).unwrap();

        let request = ExchangeRateRequest::new(
            Currency::CLF,
            Some(Currency::CLP),
            Some(Date::new(2022, 1, 1)),
        );
        let fx2 = model.gen_fx_data(&request).unwrap();

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2022, 1, 1),
            Date::new(2022, 1, 1),
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();

        assert!((fx1 / fx2 - fx3).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_model_fx_with_triangulation_2() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);

        let request = ExchangeRateRequest::new(
            Currency::USD,
            Some(Currency::CLP),
            Some(Date::new(2022, 1, 1)),
        );
        let fx1 = model.gen_fx_data(&request).unwrap();

        let request = ExchangeRateRequest::new(
            Currency::CLF,
            Some(Currency::CLP),
            Some(Date::new(2022, 1, 1) + 1),
        );
        let fx2 = model.gen_fx_data(&request).unwrap();

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2022, 1, 1),
            Date::new(2022, 1, 1) + 1,
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();

        assert!((fx1 / fx2 - fx3).abs() < 1e-6);

        let request = ExchangeRateRequest::new(
            Currency::USD,
            Some(Currency::CLP),
            Some(Date::new(2022, 1, 1) + 1),
        );
        let fx1 = model.gen_fx_data(&request).unwrap();

        let request = ExchangeRateRequest::new(
            Currency::CLF,
            Some(Currency::CLP),
            Some(Date::new(2022, 1, 1)),
        );
        let fx2 = model.gen_fx_data(&request).unwrap();

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2022, 1, 1) + 1,
            Date::new(2022, 1, 1),
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();

        assert!((fx1 / fx2 - fx3).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_model_fx_with_triangulation_3() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);

        let request = ExchangeRateRequest::new(
            Currency::USD,
            Some(Currency::CLP),
            Some(Date::new(2020, 1, 1)),
        );
        let fx1 = model.gen_fx_data(&request).unwrap();

        let request = ExchangeRateRequest::new(
            Currency::CLF,
            Some(Currency::CLP),
            Some(Date::new(2020, 1, 1) + 1),
        );
        let fx2 = model.gen_fx_data(&request).unwrap();

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2020, 1, 1),
            Date::new(2020, 1, 1) + 1,
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();

        assert!((fx1 / fx2 - fx3).abs() < 1e-6);

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2020, 1, 1),
            Date::new(2020, 1, 1) - 1,
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();
        assert!((34_500.0 / 800.0 - fx3).abs() < 1e-6);

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2020, 1, 1),
            Date::new(2020, 1, 1) - 2,
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();
        assert!((34_000.0 / 800.0 - fx3).abs() < 1e-6);

        let genereation_method = ExchangeGenerationMethod::Triangulation(Triangulation::new(
            Currency::CLP,
            Date::new(2020, 1, 1) - 1,
            Date::new(2020, 1, 1) - 2,
        ));
        let request = ExchangeRateRequest::new_with_method(
            Currency::USD,
            Some(Currency::CLF),
            Some(genereation_method),
        );
        let fx3 = model.gen_fx_data(&request).unwrap();
        assert!((34_000.0 / 799.0 - fx3).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_model_fx_with_single_date() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);
        let method = ExchangeGenerationMethod::SingleDate(SingleDate::new(Date::new(2020, 1, 1)));
        let request =
            ExchangeRateRequest::new_with_method(Currency::USD, Some(Currency::CLF), Some(method));
        let fx1 = model.gen_fx_data(&request).unwrap();
        assert!(35_000.0 / 800.0 - fx1 < 1e-6);

        let method =
            ExchangeGenerationMethod::SingleDate(SingleDate::new(Date::new(2020, 1, 1) - 1));
        let request =
            ExchangeRateRequest::new_with_method(Currency::USD, Some(Currency::CLF), Some(method));
        let fx2 = model.gen_fx_data(&request);
        assert!(fx2.is_err());

        Ok(())
    }

    #[test]
    fn test_model_fx_with_window() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);

        let method = ExchangeGenerationMethod::DateWindow(DateWindow::new(vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 1, 1) - 1,
            Date::new(2020, 1, 1) - 2,
            Date::new(2020, 1, 1) - 3,
        ]));

        let request =
            ExchangeRateRequest::new_with_method(Currency::USD, Some(Currency::CLP), Some(method));
        let fx1 = model.gen_fx_data(&request).unwrap();

        println!("{}", fx1);
        assert!((1.0 / (800.0 - 6.0 / 4.0) - fx1).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_model_fx_with_window_2() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);

        let method = ExchangeGenerationMethod::DateWindow(DateWindow::new(vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 1, 1) - 1,
            Date::new(2020, 1, 1) - 2,
            Date::new(2020, 1, 1) - 3,
            Date::new(2020, 1, 1) - 100,
        ]));

        let request =
            ExchangeRateRequest::new_with_method(Currency::USD, Some(Currency::CLP), Some(method));
        let fx1 = model.gen_fx_data(&request);

        assert!(fx1.is_err());
        Ok(())
    }

    #[test]
    fn test_model_cache() -> Result<()> {
        let market_store = create_marketstore()?;
        let model = SimpleModel::new(&market_store);

        let request = ExchangeRateRequest::new(
            Currency::USD,
            Some(Currency::CLP),
            Some(Date::new(2020, 1, 1)),
        );
        let fx1 = model.gen_fx_data(&request).unwrap();

        let fx_cache = model.fx_cache();
        let fx_cache = fx_cache.read().unwrap();
        assert!(fx_cache.contains_key(&request));
        assert!((fx1 - fx_cache.get(&request).copied().unwrap()).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn test_model_cache_clear() -> Result<()> {
            let market_store = create_marketstore()?;
        let model = SimpleModel::new(&market_store);

        let request = ExchangeRateRequest::new(
            Currency::USD,
            Some(Currency::CLP),
            Some(Date::new(2020, 1, 1)),
        );
        let _ = model.gen_fx_data(&request).unwrap();
        model.clear_cache();
        let fx_cache = model.fx_cache();
        let fx_cache = fx_cache.read().unwrap();
        assert!(!fx_cache.contains_key(&request));
        Ok(())
    }
}
