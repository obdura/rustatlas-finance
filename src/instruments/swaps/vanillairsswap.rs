use std::collections::BTreeMap;

use super::leg::Leg;
use crate::utils::errors::Result;
use crate::{
    cashflows::{cashflow::Cashflow, traits::InterestAccrual},
    core::traits::HasCurrency,
    currencies::enums::Currency,
    instruments::traits::RateType,
    time::date::Date,
    utils::errors::AtlasError,
    visitors::traits::HasCashflows,
};

/// # Vanilla IRS VanillaIRSSwap
/// A financial vanillairsswap derivative.
pub struct VanillaIRSSwap {
    first_leg: Leg,
    second_leg: Leg,
    currency: Currency,
    payment_currency: Currency,
    id: Option<String>,
    first_rate_type: RateType,
    second_rate_type: RateType,
}

impl VanillaIRSSwap {
    /// Create a new vanillairsswap.
    pub fn new(first_leg: Leg, second_leg: Leg, payment_currency: Currency) -> Result<Self> {
        let _ = check_integrity(&first_leg, &second_leg)?;
        let currency = first_leg.currency();
        let first_rate_type = first_leg.rate_type();
        let second_rate_type = second_leg.rate_type();

        let vanillairsswap = VanillaIRSSwap {
            first_leg,
            second_leg,
            currency: currency,
            payment_currency,
            id: None,
            first_rate_type,
            second_rate_type,
        };
        Ok(vanillairsswap)
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

    pub fn payment_currency(&self) -> Currency {
        self.payment_currency
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
}

impl HasCurrency for VanillaIRSSwap {
    fn currency(&self) -> Result<Currency> {
        Ok(self.currency)
    }
}

impl InterestAccrual for VanillaIRSSwap {
    fn accrual_start_date(&self) -> Result<Date> {
        self.first_leg.accrual_start_date()
    }

    fn accrual_end_date(&self) -> Result<Date> {
        self.first_leg.accrual_end_date()
    }

    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let fixed_leg_accrued = self.first_leg.accrued_amount(start_date, end_date)?;
        let floating_leg_accrued = self.first_leg.accrued_amount(start_date, end_date)?;
        Ok(fixed_leg_accrued + floating_leg_accrued)
    }

    fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>> {
        let map = self
            .cashflows()
            .try_fold(BTreeMap::new(), |mut acc, cf| -> Result<_> {
                let cf_map = cf.accrued_amount_map()?;
                for (date, amount) in cf_map {
                    let entry = acc.entry(date).or_insert(0.0);
                    *entry += amount;
                }
                Ok(acc)
            })?;
        Ok(map)
    }
}

impl HasCashflows for VanillaIRSSwap {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(
            self.first_leg
                .cashflows()
                .into_iter()
                .chain(self.second_leg.cashflows().into_iter()),
        )
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(
            self.first_leg
                .mut_cashflows()
                .chain(self.second_leg.mut_cashflows()),
        )
    }
}

fn check_integrity(first_leg: &Leg, second_leg: &Leg) -> Result<()> {
    if first_leg.currency() != second_leg.currency() {
        return Err(AtlasError::InvalidValueErr(
            "Currency mismatch between fixed and floating legs".to_string(),
        ));
    }

    if first_leg.side() == second_leg.side() {
        return Err(AtlasError::InvalidValueErr(
            "Both legs have the same side (e.g., both are Receive or Pay)".to_string(),
        ));
    }

    Ok(())
}

use colored::*;
impl std::fmt::Display for VanillaIRSSwap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {        
        write!(f, "{} {}\n","VanillaIRSSwap  id: ".white().bold() ,self.id().unwrap_or("no set!".to_string()).cyan())?;
        write!(f, "{} \n {}", "-> First_leg: ".white().bold(), self.first_leg())?;
        write!(f, "{} \n {}", "-> Second_leg: ".white().bold(), self.second_leg())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cashflows::{side::Side, traits::RequiresFixingRate},
        instruments::constructors::{
            makefixedrateleg::MakeFixedRateLeg, makefloatingrateleg::MakeFloatingRateLeg,
        },
        rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
        },
        time::{daycounter::DayCounter, enums::Frequency},
    };

    #[test]
    fn test_vanillairsswap_creation() -> Result<()> {
        let start_date = Date::new(2021, 1, 1);
        let end_date = Date::new(2023, 1, 1);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(0.05, rate_definition);
        let notional = 100.0;

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()
            .unwrap();

        let float_leg = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Quarterly)
            .with_spread(0.0)
            .with_rate_definition(rate_definition)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .with_notional(notional)
            .bullet()
            .build()
            .unwrap();

        let mut vanillairsswap = VanillaIRSSwap::new(fix_leg, float_leg, Currency::CLP)?;

        vanillairsswap.mut_cashflows().for_each(|cf| {
            cf.set_fixing_rate(0.05);
        });

        assert!(vanillairsswap.cashflows_as_vec().len() == 14);

        Ok(())
    }
}
