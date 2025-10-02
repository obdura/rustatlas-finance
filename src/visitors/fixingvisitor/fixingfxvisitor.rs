use crate::{
    cashflows::cashflow::Cashflow,
    core::traits::Registrable,
    models::traits::Model,
    utils::errors::{AtlasError, Result},
};

use super::super::traits::{HasCashflows, Visit};

/// # FixingFxVisitor
/// FixingFxVisitor is a visitor that fixes the fx echange rate of a index fx cashflow
/// ## Parameters
/// * `model` - The model to use for fixing
/// * `decimals_to_round` - The number of decimals to round the fixing fx
pub struct FixinFxVisitor<'a> {
    model: &'a dyn Model,
    decimals_to_round: usize,
    truncated: bool,
}

impl<'a> FixinFxVisitor<'a> {
    pub fn new(model: &'a dyn Model) -> Self {
        FixinFxVisitor {
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
                if let Cashflow::IndexFxCashflow(ifc) = cf {
                    let fx = ifc.fx_fixing_request()?.ok_or(AtlasError::InvalidValueErr(
                        "No fixing fx request for index fx cashflow".to_string(),
                    ))?;
                    let mut fixing_fx = self.model.gen_fx_data(&fx)?;
                    if self.truncated {
                        fixing_fx = (fixing_fx * 10_f64.powi(self.decimals_to_round as i32))
                            .round()
                            / 10_f64.powi(self.decimals_to_round as i32);
                        ifc.set_fixing_fx(fixing_fx);
                    } else {
                        ifc.set_fixing_fx(fixing_fx);
                    }
                }
                Ok(())
            })?;
        Ok(())
    }
}

// Implementation for types concrete
impl<'a, T: HasCashflows> Visit<T> for FixinFxVisitor<'a> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut T) -> Self::Output {
        self.visit_cashflows(has_cashflows)
    }
}

// Implementation for trait object in Box
impl<'a> Visit<&mut Box<dyn HasCashflows>> for FixinFxVisitor<'a> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut &mut Box<dyn HasCashflows>) -> Self::Output {
        self.visit_cashflows(has_cashflows.as_mut())
    }
}
