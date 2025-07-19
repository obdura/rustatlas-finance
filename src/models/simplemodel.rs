use crate::{
    core::{
        marketstore::MarketStore,
        meta::{DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest, MarketRequest},
    },
    rates::{indexstore::ReadIndex, traits::HasReferenceDate},
    time::date::Date,
    utils::errors::Result,
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
}

impl<'a> SimpleModel<'a> {
    pub fn new(market_store: &'a MarketStore) -> SimpleModel {
        SimpleModel { market_store }
    }
}

impl<'a> Model for SimpleModel<'a> {
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
        let index = self.market_store.get_index(id)?;
        let curve = index.read_index()?.term_structure()?;
        let df = curve.discount_factor(date)?;

        let currency_forescast_factor = match df_request.discount_currency() {
            Some(currency) => match index.read_index()?.currency()? {
                Some(currency_index) => self.market_store.index_store().currency_forescast_factor(
                    currency_index,
                    currency,
                    date,
                )?,
                None => 1.0,
            },
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

        let index = self.market_store.get_index(id)?;
        let fwd_rate_provider = index.read_index()?;
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
            .exchange_rate_store()
            .get_exchange_rate(first_currency, second_currency)?;

        match fx.reference_date() {
            Some(date) => {
                if date > self.reference_date() {
                    let currency_forescast_factor = self
                        .market_store
                        .index_store()
                        .currency_forescast_factor(first_currency, second_currency, date)?;
                    Ok(spot * currency_forescast_factor)
                }
                else {
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
mod test {
    use std::sync::{Arc, RwLock};

    use crate::{
        core::{
            marketstore::MarketStore,
            meta::{DiscountFactorRequest, ExchangeRateRequest},
        },
        currencies::enums::Currency,
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
            .with_frequency(Frequency::Annual)
            .with_currency(Some(Currency::USD))
            .with_name(Some("USD_curve".to_string()));

        let ibor_index_2 = IborIndex::new(forecast_curve_2.reference_date())
            .with_term_structure(forecast_curve_2)
            .with_frequency(Frequency::Annual)
            .with_currency(Some(Currency::CLP))
            .with_name(Some("CLP_curve".to_string()));

        let ibor_index_3 = IborIndex::new(forecast_curve_3.reference_date())
            .with_term_structure(forecast_curve_3)
            .with_frequency(Frequency::Annual)
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
        let df = model.gen_df_data(df_request).unwrap();
        assert!((df - (1.0 / (1.0 + 0.02 * 366.0 / 360.0))).abs() < 1e-6);

        let mut df_request = DiscountFactorRequest::new(0, date);
        df_request.set_discount_currency(Currency::USD);
        let df = model.gen_df_data(df_request).unwrap();
        assert!((df - (1.0 / (1.0 + 0.02 * 366.0 / 360.0))).abs() < 1e-6);

        let mut df_request = DiscountFactorRequest::new(0, date);
        df_request.set_discount_currency(Currency::CLP);
        let df = model.gen_df_data(df_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::USD), Some(date));
        let fx_clp_usd = model.gen_fx_data(fx_request).unwrap();
        assert!((fx_clp_usd * df - (1.0 / (1.0 + 0.02 * 366.0 / 360.0)) * 800.0).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_simple_model_fx_request() -> Result<()> {
        let market_store = create_marketstore().unwrap();
        let model = SimpleModel::new(&market_store);
        let date = Date::new(2021, 1, 1);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLP), Some(date));
        let fx_usd_clp = model.gen_fx_data(fx_request).unwrap();
        assert!((fx_usd_clp - 0.00123766779880317).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::USD), Some(date));
        let fx_clp_usd = model.gen_fx_data(fx_request).unwrap();
        assert!((fx_clp_usd * fx_usd_clp - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::CLF), Some(date));
        let fx_clp_clf = model.gen_fx_data(fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::CLP), Some(date));
        let fx_clf_clp = model.gen_fx_data(fx_request).unwrap();

        assert!((fx_clf_clp * fx_clp_clf - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLF), Some(date));
        let fx_usd_clf = model.gen_fx_data(fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::USD), Some(date));
        let fx_clf_usd = model.gen_fx_data(fx_request).unwrap();

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
        let fx_usd_clp = model.gen_fx_data(fx_request).unwrap();
        assert!((fx_usd_clp - 0.00123766779880317).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::USD), Some(date));
        let fx_clp_usd = model.gen_fx_data(fx_request).unwrap();
        assert!((fx_clp_usd * fx_usd_clp - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::CLP, Some(Currency::CLF), Some(date));
        let fx_clp_clf = model.gen_fx_data(fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::CLP), Some(date));
        let fx_clf_clp = model.gen_fx_data(fx_request).unwrap();

        assert!((fx_clf_clp * fx_clp_clf - 1.0).abs() < 1e-6);

        let fx_request = ExchangeRateRequest::new(Currency::USD, Some(Currency::CLF), Some(date));
        let fx_usd_clf = model.gen_fx_data(fx_request).unwrap();

        let fx_request = ExchangeRateRequest::new(Currency::CLF, Some(Currency::USD), Some(date));
        let fx_clf_usd = model.gen_fx_data(fx_request).unwrap();

        assert!((fx_clf_usd * fx_usd_clf - 1.0).abs() < 1e-6);
        Ok(())
    }
}
