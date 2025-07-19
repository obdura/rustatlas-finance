use crate::{core::meta::*, time::date::Date, utils::errors::Result};

/// # Model
/// A model that provides market data based in the current market state.
/// 
/// ## Methods
/// * `reference_date` - The reference date of the model.
/// * `gen_df_data` - Generates the discount factor data.
/// * `gen_fx_data` - Generates the exchange rate data.
/// * `gen_fwd_data` - Generates the forward rate data.
/// * `gen_numerarie` - Generates the numerarie data.
/// * `gen_node` - Generates a single market data node.
/// * `gen_market_data` - Generates a list of market data nodes.
pub trait Model {
    fn reference_date(&self) -> Date;
    fn gen_df_data(&self, df: DiscountFactorRequest) -> Result<f64>;
    fn gen_fx_data(&self, fx: ExchangeRateRequest) -> Result<f64>;
    fn gen_fwd_data(&self, fwd: ForwardRateRequest) -> Result<f64>;
    fn gen_numerarie(&self, market_request: &MarketRequest) -> Result<f64>;
    
    fn gen_node(&self, market_request: &MarketRequest) -> Result<MarketData> {
        let id = market_request.id();
        let df = match market_request.df() {
            Some(df) => Some(self.gen_df_data(df)?),
            None => None,
        };

        let fwd = match market_request.fwd() {
            Some(fwd) => Some(self.gen_fwd_data(fwd)?),
            None => None,
        };

        let fx = match market_request.fx() {
            Some(fx) => Some(self.gen_fx_data(fx)?),
            None => None,
        };

        let fx_fwd = match market_request.fx_fwd() {
            Some(fx_fwd) => Some(self.gen_fx_data(fx_fwd)?),
            None => None,
        };

        let numerarie = self.gen_numerarie(market_request)?;

        return Ok(MarketData::new(
            id,
            self.reference_date(),
            df,
            fwd,
            fx,
            fx_fwd,
            numerarie,
        ));
    }

    fn gen_market_data(&self, market_request: &[MarketRequest]) -> Result<Vec<MarketData>> {
        market_request.iter().map(|x| self.gen_node(x)).collect()
    }
}
