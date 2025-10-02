use crate::{
    cashflows::{cashflow::Cashflow, traits::RequiresFixingRate},
    models::traits::Model,
    utils::errors::Result,
};

use super::super::traits::{HasCashflows, Visit};

// /// # FixingVisitor
// /// FixingVisitor is a visitor that fixes the rate of a floating rate cashflow.
// ///
// /// ## Parameters
// /// * `market_data` - The market data to use for fixing
// /// * `decimals_to_round` - The number of decimals to round the fixing rate
// ///
// #[deprecated(
//     since = "1.0.0",
//     note = "This struct is deprecated and will be removed in future versions. Please use the new implementation."
// )]
// pub struct FixingVisitor<'a> {
//     market_data: &'a [MarketData],
//     decimals_to_round: usize,
//     truncated: bool,
// }

// impl<'a> FixingVisitor<'a> {
//     pub fn new(market_data: &'a [MarketData]) -> Self {
//         FixingVisitor {
//             market_data: market_data,
//             decimals_to_round: 10,
//             truncated: false,
//         }
//     }

//     pub fn with_decimals_to_round(mut self, decimals_to_round: usize) -> Self {
//         self.decimals_to_round = decimals_to_round;
//         self.truncated = true;
//         self
//     }

//     fn visit_cashflows(&self, has_cashflows: &mut dyn HasCashflows) -> Result<()> {
//         has_cashflows
//             .mut_cashflows()
//             .try_for_each(|cf| -> Result<()> {
//                 if let Cashflow::FloatingRateCoupon(frcf) = cf {
//                     let id = frcf.id()?;
//                     let cf_market_data =
//                         self.market_data
//                             .get(id)
//                             .ok_or(AtlasError::NotFoundErr(format!(
//                                 "Market data for cashflow with id {}",
//                                 id
//                             )))?;
//                     let mut fixing_rate = cf_market_data.fwd()?;
//                     if self.truncated {
//                         fixing_rate = (fixing_rate * 10_f64.powi(self.decimals_to_round as i32))
//                             .round()
//                             / 10_f64.powi(self.decimals_to_round as i32);
//                         frcf.set_fixing_rate(fixing_rate);
//                     } else {
//                         frcf.set_fixing_rate(fixing_rate);
//                     }
//                 }
//                 Ok(())
//             })?;
//         Ok(())
//     }
// }

// // Implementation for types concrete
// impl<'a, T: HasCashflows> Visit<T> for FixingVisitor<'a> {
//     type Output = Result<()>;
//     fn visit(&self, has_cashflows: &mut T) -> Self::Output {
//         self.visit_cashflows(has_cashflows)
//     }
// }

// // Implementation for trait object in Box
// impl<'a> Visit<&mut Box<dyn HasCashflows>> for FixingVisitor<'a> {
//     type Output = Result<()>;
//     fn visit(&self, has_cashflows: &mut &mut Box<dyn HasCashflows>) -> Self::Output {
//         self.visit_cashflows(has_cashflows.as_mut())
//     }
// }

/// # ModelFixingVisitor
/// ModelFixingVisitor is a visitor that fixes the rate of a floating rate cashflo using implementations of the model trait
///
/// ## Parameters
/// * `model` - The model to use for fixing
/// * `decimals_to_round` - The number of decimals to round the fixing rate
pub struct ModelFixingVisitor<'a> {
    model: &'a dyn Model,
    decimals_to_round: usize,
    truncated: bool,
}

impl<'a> ModelFixingVisitor<'a> {
    pub fn new(model: &'a dyn Model) -> Self {
        ModelFixingVisitor {
            model,
            decimals_to_round: 10,
            truncated: false,
        }
    }
    pub fn with_decimals_to_round(mut self, decimals_to_round: usize) -> Self {
        self.decimals_to_round = decimals_to_round;
        self.truncated = true;
        self
    }

    fn visit_cashflows(&self, has_cashflows: &mut dyn HasCashflows) -> Result<()> {
        has_cashflows
            .mut_cashflows()
            .try_for_each(|cf| -> Result<()> {
                if let Cashflow::FloatingRateCoupon(frcf) = cf {
                    let fwd = frcf.fixing_request()?;
                    let mut fixing_rate = self.model.gen_fwd_data(&fwd)?;
                    if self.truncated {
                        fixing_rate = (fixing_rate * 10_f64.powi(self.decimals_to_round as i32))
                            .round()
                            / 10_f64.powi(self.decimals_to_round as i32);
                        frcf.set_fixing_rate(fixing_rate);
                    } else {
                        frcf.set_fixing_rate(fixing_rate);
                    }
                }
                Ok(())
            })?;
        Ok(())
    }
}

// Implementation for types concrete
impl<'a, T: HasCashflows> Visit<T> for ModelFixingVisitor<'a> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut T) -> Self::Output {
        self.visit_cashflows(has_cashflows)
    }
}

// Implementation for trait object in Box
impl<'a> Visit<&mut Box<dyn HasCashflows>> for ModelFixingVisitor<'a> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut &mut Box<dyn HasCashflows>) -> Self::Output {
        self.visit_cashflows(has_cashflows.as_mut())
    }
}
