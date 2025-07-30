use crate::cashflows::cashflow::Cashflow;
use crate::instruments::traits::RateType;
use crate::time::date::Date;
//use crate::instruments::traits::RateType;
use crate::visitors::traits::HasCashflows;
use crate::{core::traits::HasCurrency, currencies::enums::Currency};
use crate::utils::errors::{AtlasError, Result};
use super::leg::Leg;

/// # Vanilla IRS CrossCurrencySwap
/// A financial crosscurrencyswap derivative.
pub struct CrossCurrencySwap {
    first_leg: Leg,
    second_leg: Leg,
    first_leg_currency: Currency,
    second_leg_currency: Currency,
    payment_currency: Currency,
    initial_fx_rate: Option<f64>,
    id: Option<String>,
    first_rate_type: RateType,
    second_rate_type: RateType,
}

impl CrossCurrencySwap {
    /// Create a new crosscurrencyswap.
    pub fn new(first_leg: Leg, second_leg: Leg, payment_currency: Currency) -> Result<Self> {
        let _ = check_integrity(&first_leg, &second_leg);
        let first_leg_currency = first_leg.currency();
        let second_leg_currency = second_leg.currency();
        let first_rate_type = first_leg.rate_type();
        let second_rate_type = second_leg.rate_type();

        Ok(CrossCurrencySwap {
            first_leg,
            second_leg,
            first_leg_currency,
            second_leg_currency,
            payment_currency,
            initial_fx_rate: None,
            id: None,
            first_rate_type,
            second_rate_type,
        })
    }

    pub fn first_leg(&self) -> &Leg {
        &self.first_leg
    }
    pub fn second_leg(&self) -> &Leg {
        &self.second_leg
    }

    pub fn mut_first_leg(&mut self) -> &mut Leg {
        &mut self.first_leg
    }

    pub fn mut_second_leg(&mut self) -> &mut Leg {
        &mut self.second_leg
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn first_leg_currency(&self) -> Currency {
        self.first_leg_currency
    }

    pub fn second_leg_currency(&self) -> Currency {
        self.second_leg_currency
    }

    pub fn payment_currency(&self) -> Currency {
        self.payment_currency
    }

    pub fn initial_fx_rate(&self) -> Option<f64> {
        self.initial_fx_rate
    }

    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn first_rate_type(&self) -> RateType {
        self.first_rate_type
    }

    pub fn second_rate_type(&self) -> RateType {
        self.second_rate_type
    }

    pub fn last_payment_date(&self) -> Date {
        self.first_leg.last_payment_date().max(self.second_leg.last_payment_date())
    }

}

fn check_integrity(first_leg: &Leg, second_leg: &Leg) -> Result<()>{
   if first_leg.currency() == second_leg.currency() { 
        return Err(AtlasError::InvalidValueErr(
            "Currency needs to be different for each leg".to_string(),
        ));
   }

   if first_leg.side() == second_leg.side() {
        return Err(AtlasError::InvalidValueErr(
            "Both legs have the same side (e.g., both are Receive or Pay)".to_string(),
        ));
    }
   Ok(())
}

impl HasCurrency for CrossCurrencySwap {
    fn currency(&self) -> Result<Currency> {
        Ok(self.payment_currency)
    }
}

impl HasCashflows for CrossCurrencySwap {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.first_leg.cashflows().into_iter().chain(self.second_leg.cashflows().into_iter()))
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(
            self.first_leg
                .mut_cashflows()
                .chain(self.second_leg.mut_cashflows()),
        )
    }
}

use colored::*;
impl std::fmt::Display for CrossCurrencySwap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\n","CrossCurrencySwap  id: ".white().bold() ,self.id().unwrap_or("no set!".to_string()).cyan())?;
        write!(f, "{} \n {}", "-> First_leg: ".white().bold(), self.first_leg())?;
        write!(f, "{} \n {}", "-> Second_leg: ".white().bold(), self.second_leg())
    }
}