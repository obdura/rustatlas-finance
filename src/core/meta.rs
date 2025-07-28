use crate::currencies::enums::Currency;
use crate::rates::enums::Compounding;
use crate::time::date::Date;
use crate::time::enums::Frequency;

use crate::utils::errors::{AtlasError, Result};

/// # ExchangeRateRequest
/// Meta data for an exchange rate. Holds the first currency, the second currency and the reference
/// date required to fetch the exchange rate.
///
/// ## Parameters
/// * `first_currency` - The first currency of the exchange rate.
/// * `second_currency` - The second currency of the exchange rate.
/// * `reference_date` - The reference date of the exchange rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExchangeRateRequest {
    first_currency: Currency,
    second_currency: Option<Currency>,
    reference_date: Option<Date>,
}

impl ExchangeRateRequest {
    pub fn new(
        first_currency: Currency,
        second_currency: Option<Currency>,
        reference_date: Option<Date>,
    ) -> ExchangeRateRequest {
        ExchangeRateRequest {
            first_currency,
            second_currency,
            reference_date,
        }
    }

    pub fn first_currency(&self) -> Currency {
        self.first_currency
    }

    pub fn second_currency(&self) -> Option<Currency> {
        self.second_currency
    }

    pub fn reference_date(&self) -> Option<Date> {
        self.reference_date
    }
}

/// # DiscountFactorRequest
/// Meta data for a discount factor. Holds the discount curve id and the reference date required to
/// fetch the discount factor.
///
/// ## Parameters
/// * `discount_curve_id` - The discount curve id of the discount factor.
/// * `date` - The reference date of the discount factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscountFactorRequest {
    provider_id: usize,
    date: Date,
    discount_currency: Option<Currency>,
}

impl DiscountFactorRequest {
    pub fn new(provider_id: usize, date: Date) -> DiscountFactorRequest {
        DiscountFactorRequest { provider_id, date, discount_currency: None }
    }

    pub fn new_with_discount_currency(provider_id: usize, date: Date, discount_currency: Currency) -> DiscountFactorRequest {
        DiscountFactorRequest { provider_id, date, discount_currency: Some(discount_currency) }
    }

    pub fn set_discount_currency(&mut self, discount_currency: Currency) -> &mut Self {
        self.discount_currency = Some(discount_currency);
        self
    }

    pub fn provider_id(&self) -> usize {
        self.provider_id
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn discount_currency(&self) -> Option<Currency> {
        self.discount_currency
    }
}

/// # ForwardRateRequest
/// Meta data for a forward rate. Holds the forward curve id and the start and end dates required
/// to fetch the forward rate.
///
/// ## Parameters
/// * `provider_id` - The forward curve id of the forward rate.
/// * `start_date` - The start date of the forward rate.
/// * `end_date` - The end date of the forward rate.
/// * `compounding` - The compounding of the forward rate.
/// * `frequency` - The frequency of the forward rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForwardRateRequest {
    provider_id: usize,
    start_date: Date,
    end_date: Date,
    compounding: Compounding,
    frequency: Frequency,
}

impl ForwardRateRequest {
    pub fn new(
        provider_id: usize,
        start_date: Date,
        end_date: Date,
        compounding: Compounding,
        frequency: Frequency,
    ) -> ForwardRateRequest {
        ForwardRateRequest {
            provider_id,
            start_date,
            end_date,
            compounding,
            frequency,
        }
    }

    pub fn provider_id(&self) -> usize {
        self.provider_id
    }

    pub fn start_date(&self) -> Date {
        self.start_date
    }

    pub fn end_date(&self) -> Date {
        self.end_date
    }

    pub fn compounding(&self) -> Compounding {
        self.compounding
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }
}

/// # MarketRequest
/// Meta data for market data. Holds all the meta data required to fetch the market data.
///
/// ## Parameters
/// * `id` - The id of the market data.
/// * `df` - The discount factor meta data.
/// * `fwd` - The forward rate meta data.
/// * `fx` - The exchange rate meta data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketRequest {
    id: usize,
    df: Option<DiscountFactorRequest>,
    fwd: Option<ForwardRateRequest>,
    fx: Option<ExchangeRateRequest>,
    fx_fwd: Option<ExchangeRateRequest>,
}

impl MarketRequest {
    pub fn new(
        id: usize,
        df: Option<DiscountFactorRequest>,
        fwd: Option<ForwardRateRequest>,
        fx: Option<ExchangeRateRequest>,
        fx_fwd: Option<ExchangeRateRequest>,
    ) -> MarketRequest {
        MarketRequest { id, df, fwd, fx, fx_fwd}
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn df(&self) -> Option<DiscountFactorRequest> {
        self.df
    }

    pub fn fwd(&self) -> Option<ForwardRateRequest> {
        self.fwd
    }

    pub fn fx(&self) -> Option<ExchangeRateRequest> {
        self.fx
    }

    pub fn fx_fwd(&self) -> Option<ExchangeRateRequest> {
        self.fx_fwd
    }
}

/// # MarketDataNode
/// Market data. Holds all the data required to price a cashflow.
///
/// ## Parameters
/// * `id` - The id of the market data.
/// * `df` - The discount factor.
/// * `fwd` - The forward rate.
/// * `fx` - The exchange rate.
#[derive(Debug, Clone, Copy)]
pub struct MarketData {
    id: usize,
    reference_date: Date,
    df: Option<f64>,
    fwd: Option<f64>,
    fx: Option<f64>,
    fx_fwd: Option<f64>,
    numerarie: f64,
}

impl MarketData {
    pub fn new(
        id: usize,
        reference_date: Date,
        df: Option<f64>,
        fwd: Option<f64>,
        fx: Option<f64>,
        fx_fwd: Option<f64>,
        numerarie: f64,
    ) -> MarketData {
        MarketData {
            id,
            reference_date,
            df,
            fwd,
            fx,
            fx_fwd,
            numerarie,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn df(&self) -> Result<f64> {
        self.df.ok_or(AtlasError::ValueNotSetErr("df".to_string()))
    }

    pub fn fwd(&self) -> Result<f64> {
        self.fwd
            .ok_or(AtlasError::ValueNotSetErr("fwd".to_string()))
    }

    pub fn fx(&self) -> Result<f64> {
        self.fx.ok_or(AtlasError::ValueNotSetErr("fx".to_string()))
    }

    pub fn fx_fwd(&self) -> Result<f64> {
        self.fx_fwd.ok_or(AtlasError::ValueNotSetErr("fx_fwd".to_string()))
    }

    pub fn numerarie(&self) -> f64 {
        self.numerarie
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    fn sample_date() -> Date {
        Date::new(2024, 6, 1)
    }

    #[test]
    fn test_exchange_rate_request_new_and_accessors() {
        let req = ExchangeRateRequest::new(Currency::USD, Some(Currency::EUR), Some(sample_date()));
        assert_eq!(req.first_currency(), Currency::USD);
        assert_eq!(req.second_currency(), Some(Currency::EUR));
        assert_eq!(req.reference_date(), Some(sample_date()));
    }

    #[test]
    fn test_discount_factor_request_new_and_accessors() {
        let req = DiscountFactorRequest::new(42, sample_date());
        assert_eq!(req.provider_id(), 42);
        assert_eq!(req.date(), sample_date());
        assert_eq!(req.discount_currency(), None);

        let req2 = DiscountFactorRequest::new_with_discount_currency(7, sample_date(), Currency::JPY);
        assert_eq!(req2.provider_id(), 7);
        assert_eq!(req2.discount_currency(), Some(Currency::JPY));
    }

    #[test]
    fn test_discount_factor_request_set_discount_currency() {
        let mut req = DiscountFactorRequest::new(1, sample_date());
        req.set_discount_currency(Currency::GBP);
        assert_eq!(req.discount_currency(), Some(Currency::GBP));
    }

    #[test]
    fn test_forward_rate_request_new_and_accessors() {
        let req = ForwardRateRequest::new(
            99,
            sample_date(),
            sample_date(),
            Compounding::Simple,
            Frequency::Annual,
        );
        assert_eq!(req.provider_id(), 99);
        assert_eq!(req.start_date(), sample_date());
        assert_eq!(req.end_date(), sample_date());
        assert_eq!(req.compounding(), Compounding::Simple);
        assert_eq!(req.frequency(), Frequency::Annual);
    }

    #[test]
    fn test_market_request_new_and_accessors() {
        let df_req = DiscountFactorRequest::new(1, sample_date());
        let fwd_req = ForwardRateRequest::new(2, sample_date(), sample_date(), Compounding::Simple, Frequency::Annual);
        let fx_req = ExchangeRateRequest::new(Currency::USD, Some(Currency::EUR), Some(sample_date()));
        let fx_fwd_req = ExchangeRateRequest::new(Currency::USD, Some(Currency::JPY), Some(sample_date()));

        let req = MarketRequest::new(123, Some(df_req), Some(fwd_req), Some(fx_req), Some(fx_fwd_req));
        assert_eq!(req.id(), 123);
        assert_eq!(req.df(), Some(df_req));
        assert_eq!(req.fwd(), Some(fwd_req));
        assert_eq!(req.fx(), Some(fx_req));
        assert_eq!(req.fx_fwd(), Some(fx_fwd_req));
    }

    #[test]
    fn test_market_data_new_and_accessors() {
        let data = MarketData::new(
            10,
            sample_date(),
            Some(0.95),
            Some(0.02),
            Some(1.1),
            Some(1.15),
            100.0,
        );
        assert_eq!(data.id(), 10);
        assert_eq!(data.reference_date(), sample_date());
        assert_eq!(data.df().unwrap(), 0.95);
        assert_eq!(data.fwd().unwrap(), 0.02);
        assert_eq!(data.fx().unwrap(), 1.1);
        assert_eq!(data.fx_fwd().unwrap(), 1.15);
        assert_eq!(data.numerarie(), 100.0);
    }

    #[test]
    fn test_market_data_missing_values() {
        let data = MarketData::new(1, sample_date(), None, None, None, None, 1.0);
        assert!(data.df().is_err());
        assert!(data.fwd().is_err());
        assert!(data.fx().is_err());
        assert!(data.fx_fwd().is_err());
    }
}