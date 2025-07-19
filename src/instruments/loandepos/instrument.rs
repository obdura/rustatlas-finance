use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    cashflows::{
        cashflow::Cashflow,
        side::Side,
        traits::{InterestAccrual, Scalable},
    },
    core::traits::HasCurrency,
    currencies::enums::Currency,
    instruments::traits::{RateType, Structure},
    rates::interestrate::RateDefinition,
    time::{date::Date, enums::Frequency},
    utils::errors::Result,
    visitors::traits::HasCashflows,
};

use super::{
    doublerateinstrument::DoubleRateInstrument, fixedrateinstrument::FixedRateInstrument,
    floatingrateinstrument::FloatingRateInstrument, hybridrateinstrument::HybridRateInstrument,
};

/// # Instrument
/// Represents an instrument. This is a wrapper around the FixedRateInstrument and FloatingRateInstrument.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Instrument {
    FixedRateInstrument(FixedRateInstrument),
    FloatingRateInstrument(FloatingRateInstrument),
    HybridRateInstrument(HybridRateInstrument),
    DoubleRateInstrument(DoubleRateInstrument),
}

impl HasCashflows for Instrument {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.cashflows(),
            Instrument::FloatingRateInstrument(fri) => fri.cashflows(),
            Instrument::HybridRateInstrument(hri) => hri.cashflows(),
            Instrument::DoubleRateInstrument(dri) => dri.cashflows(),
        }
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.mut_cashflows(),
            Instrument::FloatingRateInstrument(fri) => fri.mut_cashflows(),
            Instrument::HybridRateInstrument(hri) => hri.mut_cashflows(),
            Instrument::DoubleRateInstrument(dri) => dri.mut_cashflows(),
        }
    }
}

impl Instrument {
    pub fn notional(&self) -> f64 {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.notional(),
            Instrument::FloatingRateInstrument(fri) => fri.notional(),
            Instrument::HybridRateInstrument(hri) => hri.notional(),
            Instrument::DoubleRateInstrument(dri) => dri.notional(),
        }
    }

    pub fn start_date(&self) -> Date {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.start_date(),
            Instrument::FloatingRateInstrument(fri) => fri.start_date(),
            Instrument::HybridRateInstrument(hri) => hri.start_date(),
            Instrument::DoubleRateInstrument(dri) => dri.start_date(),
        }
    }

    pub fn end_date(&self) -> Date {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.end_date(),
            Instrument::FloatingRateInstrument(fri) => fri.end_date(),
            Instrument::HybridRateInstrument(hri) => hri.end_date(),
            Instrument::DoubleRateInstrument(dri) => dri.end_date(),
        }
    }

    pub fn id(&self) -> Option<String> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.id(),
            Instrument::FloatingRateInstrument(fri) => fri.id(),
            Instrument::HybridRateInstrument(hri) => hri.id(),
            Instrument::DoubleRateInstrument(dri) => dri.id(),
        }
    }

    pub fn structure(&self) -> Structure {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.structure(),
            Instrument::FloatingRateInstrument(fri) => fri.structure(),
            Instrument::HybridRateInstrument(hri) => hri.structure(),
            _ => Structure::Other,
        }
    }

    pub fn payment_frequency(&self) -> Frequency {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.payment_frequency(),
            Instrument::FloatingRateInstrument(fri) => fri.payment_frequency(),
            Instrument::HybridRateInstrument(hri) => hri.payment_frequency(),
            Instrument::DoubleRateInstrument(dri) => dri.payment_frequency(),
        }
    }

    pub fn side(&self) -> Option<Side> {
        match self {
            Instrument::FixedRateInstrument(fri) => Some(fri.side()),
            Instrument::FloatingRateInstrument(fri) => Some(fri.side()),
            Instrument::HybridRateInstrument(hri) => hri.side(),
            Instrument::DoubleRateInstrument(dri) => Some(dri.side()),
        }
    }

    pub fn issue_date(&self) -> Option<Date> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.issue_date(),
            Instrument::FloatingRateInstrument(fri) => fri.issue_date(),
            Instrument::HybridRateInstrument(hri) => hri.issue_date(),
            Instrument::DoubleRateInstrument(dri) => dri.issue_date(),
        }
    }

    pub fn rate_type(&self) -> RateType {
        match self {
            Instrument::FixedRateInstrument(_) => RateType::Fixed,
            Instrument::FloatingRateInstrument(_) => RateType::Floating,
            Instrument::HybridRateInstrument(hri) => hri.rate_type(),
            Instrument::DoubleRateInstrument(dri) => dri.rate_type(),
        }
    }

    pub fn rate(&self) -> Option<f64> {
        match self {
            Instrument::FixedRateInstrument(fri) => Some(fri.rate().rate()),
            Instrument::FloatingRateInstrument(_) => None,
            Instrument::HybridRateInstrument(_) => None,
            Instrument::DoubleRateInstrument(_) => None,
        }
    }

    pub fn spread(&self) -> Option<f64> {
        match self {
            Instrument::FixedRateInstrument(_) => None,
            Instrument::FloatingRateInstrument(fri) => Some(fri.spread()),
            Instrument::HybridRateInstrument(_) => None,
            Instrument::DoubleRateInstrument(_) => None,
        }
    }

    pub fn forecast_curve_id(&self) -> Option<usize> {
        match self {
            Instrument::FixedRateInstrument(_) => None,
            Instrument::FloatingRateInstrument(fri) => fri.forecast_curve_id(),
            Instrument::HybridRateInstrument(hri) => hri.forecast_curve_id(),
            Instrument::DoubleRateInstrument(dri) => dri.forecast_curve_id(),
        }
    }

    pub fn discount_curve_id(&self) -> Option<usize> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.discount_curve_id(),
            Instrument::FloatingRateInstrument(fri) => fri.discount_curve_id(),
            Instrument::HybridRateInstrument(hri) => hri.discount_curve_id(),
            Instrument::DoubleRateInstrument(dri) => dri.discount_curve_id(),
        }
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.set_discount_curve_id(id),
            Instrument::FloatingRateInstrument(fri) => fri.set_discount_curve_id(id),
            Instrument::HybridRateInstrument(hri) => hri.set_discount_curve_id(id),
            Instrument::DoubleRateInstrument(dri) => dri.set_discount_curve_id(id),
        }
    }

    pub fn set_forecast_curve_id(&mut self, id: usize) {
        match self {
            Instrument::FloatingRateInstrument(fri) => fri.set_forecast_curve_id(id),
            Instrument::HybridRateInstrument(hri) => hri.set_forecast_curve_id(id),
            Instrument::DoubleRateInstrument(dri) => dri.set_forecast_curve_id(id),
            _ => {}
        }
    }

    pub fn first_rate_definition(&self) -> Option<RateDefinition> {
        match self {
            Instrument::FixedRateInstrument(fri) => Some(fri.rate().rate_definition()),
            Instrument::FloatingRateInstrument(fri) => Some(fri.rate_definition()),
            Instrument::HybridRateInstrument(hri) => hri.first_rate_definition(),
            Instrument::DoubleRateInstrument(dri) => dri.first_rate_definition(),
        }
    }

    pub fn second_rate_definition(&self) -> Option<RateDefinition> {
        match self {
            Instrument::FixedRateInstrument(_) => None,
            Instrument::FloatingRateInstrument(_) => None,
            Instrument::HybridRateInstrument(hri) => hri.second_rate_definition(),
            Instrument::DoubleRateInstrument(dri) => dri.second_rate_definition(),
        }
    }
}

impl HasCurrency for Instrument {
    fn currency(&self) -> Result<Currency> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.currency(),
            Instrument::FloatingRateInstrument(fri) => fri.currency(),
            Instrument::HybridRateInstrument(hri) => hri.currency(),
            Instrument::DoubleRateInstrument(dri) => dri.currency(),
        }
    }
}

impl InterestAccrual for Instrument {
    fn accrual_start_date(&self) -> Result<Date> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.accrual_start_date(),
            Instrument::FloatingRateInstrument(fri) => fri.accrual_start_date(),
            Instrument::HybridRateInstrument(hri) => hri.accrual_start_date(),
            Instrument::DoubleRateInstrument(dri) => dri.accrual_start_date(),
        }
    }

    fn accrual_end_date(&self) -> Result<Date> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.accrual_end_date(),
            Instrument::FloatingRateInstrument(fri) => fri.accrual_end_date(),
            Instrument::HybridRateInstrument(hri) => hri.accrual_end_date(),
            Instrument::DoubleRateInstrument(dri) => dri.accrual_end_date(),
        }
    }
    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.accrued_amount(start_date, end_date),
            Instrument::FloatingRateInstrument(fri) => fri.accrued_amount(start_date, end_date),
            Instrument::HybridRateInstrument(hri) => hri.accrued_amount(start_date, end_date),
            Instrument::DoubleRateInstrument(dri) => dri.accrued_amount(start_date, end_date),
        }
    }

    fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.accrued_amount_map(),
            Instrument::FloatingRateInstrument(fri) => fri.accrued_amount_map(),
            Instrument::HybridRateInstrument(hri) => hri.accrued_amount_map(),
            Instrument::DoubleRateInstrument(dri) => dri.accrued_amount_map(),
        }
    }
}

impl Scalable for Instrument {
    fn scale(&mut self, factor: f64) -> Result<()> {
        match self {
            Instrument::FixedRateInstrument(fri) => fri.scale(factor)?,
            Instrument::FloatingRateInstrument(fri) => fri.scale(factor)?,
            Instrument::HybridRateInstrument(hri) => hri.scale(factor)?,
            Instrument::DoubleRateInstrument(dri) => dri.scale(factor)?,
        }
        Ok(())
    }
}

/// # Display
/// Display implementation for Instrument.
impl Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instrument::FixedRateInstrument(fri) => write!(f, "{}", fri),
            Instrument::FloatingRateInstrument(fri) => write!(f, "{}", fri),
            Instrument::DoubleRateInstrument(dri) => write!(f, "{}", dri),
            Instrument::HybridRateInstrument(_) => {
                write!(f, "HybridRateInstrument not displayable")
            }
        }
    }
}
