use crate::{
    cashflows::{cashflow::Cashflow, traits::RequiresFixingRate},
    core::{meta::MarketData, traits::Registrable},
    utils::errors::{AtlasError, Result},
};

use super::super::traits::{HasCashflows, Visit};

/// # FixingVisitor
/// FixingVisitor is a visitor that fixes the rate of a floating rate cashflow.
///
/// ## Parameters
/// * `market_data` - The market data to use for fixing
/// * `decimals_to_round` - The number of decimals to round the fixing rate
/// 
pub struct FixingVisitor<'a> {
    market_data: &'a [MarketData],
    decimals_to_round: usize,
}

impl<'a> FixingVisitor<'a> {
    pub fn new(market_data: &'a [MarketData]) -> Self {
        FixingVisitor {
            market_data: market_data,
            decimals_to_round: 6,
        }
    }

    pub fn with_decimals_to_round(mut self, decimals_to_round: usize) -> Self {
        self.decimals_to_round = decimals_to_round;
        self
    }

    fn visit_cashflows(&self, has_cashflows: &mut dyn HasCashflows) -> Result<()> {
        has_cashflows
            .mut_cashflows()
            .try_for_each(|cf| -> Result<()> {
                if let Cashflow::FloatingRateCoupon(frcf) = cf {
                    let id = frcf.id()?;
                    let cf_market_data = self.market_data.get(id)
                        .ok_or(AtlasError::NotFoundErr(format!(
                            "Market data for cashflow with id {}", id
                        )))?;
                    let mut fixing_rate = cf_market_data.fwd()?;
                    fixing_rate = (fixing_rate * 10_f64.powi(self.decimals_to_round as i32)).round()
                        / 10_f64.powi(self.decimals_to_round as i32);
                    frcf.set_fixing_rate(fixing_rate);
                }
                Ok(())
            })?;
        Ok(())
    }
}

// Implementación para tipos concretos
impl<'a, T: HasCashflows> Visit<T> for FixingVisitor<'a> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut T) -> Self::Output {
        self.visit_cashflows(has_cashflows)
    }
}

// Implementación para trait object en Box
impl<'a> Visit<&mut Box<dyn HasCashflows>> for FixingVisitor<'a> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut &mut Box<dyn HasCashflows>) -> Self::Output {
        self.visit_cashflows(has_cashflows.as_mut())
    }
}