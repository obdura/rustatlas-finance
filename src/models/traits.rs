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
pub trait Model: Send + Sync {
    fn reference_date(&self) -> Date;
    fn gen_df_data(&self, df: &DiscountFactorRequest) -> Result<f64>;
    fn gen_fx_data(&self, fx: &ExchangeRateRequest) -> Result<f64>;
    fn gen_fwd_data(&self, fwd: &ForwardRateRequest) -> Result<f64>;
}
