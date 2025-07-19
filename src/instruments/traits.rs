use std::fmt::Display;

use crate::{
    cashflows::
        cashflow::CashflowType
    ,
    utils::errors::{AtlasError, Result},
};
use serde::{Deserialize, Serialize};

/// # Structure
/// A struct that contains the information needed to define a structure.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Structure {
    Bullet,
    EqualRedemptions,
    Zero,
    EqualPayments,
    Other,
}

impl TryFrom<String> for Structure {
    type Error = AtlasError;

    fn try_from(s: String) -> Result<Self> {
        match s.as_str() {
            "Bullet" => Ok(Structure::Bullet),
            "EqualRedemptions" => Ok(Structure::EqualRedemptions),
            "Zero" => Ok(Structure::Zero),
            "EqualPayments" => Ok(Structure::EqualPayments),
            "Other" => Ok(Structure::Other),
            _ => Err(AtlasError::InvalidValueErr(format!(
                "Invalid structure: {}",
                s
            ))),
        }
    }
}

impl From<Structure> for String {
    fn from(structure: Structure) -> Self {
        match structure {
            Structure::Bullet => "Bullet".to_string(),
            Structure::EqualRedemptions => "EqualRedemptions".to_string(),
            Structure::Zero => "Zero".to_string(),
            Structure::EqualPayments => "EqualPayments".to_string(),
            Structure::Other => "Other".to_string(),
        }
    }
}

impl TryFrom<String> for CashflowType {
    type Error = AtlasError;

    fn try_from(s: String) -> Result<Self> {
        match s.as_str() {
            "Redemption" => Ok(CashflowType::Redemption),
            "Disbursement" => Ok(CashflowType::Disbursement),
            "FixedRateCoupon" => Ok(CashflowType::FixedRateCoupon),
            "FloatingRateCoupon" => Ok(CashflowType::FloatingRateCoupon),
            _ => Err(AtlasError::InvalidValueErr(format!(
                "Invalid cashflow type: {}",
                s
            ))),
        }
    }
}

impl Display for Structure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}


/// # RateType
/// Represents the type of rate.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RateType {
    Fixed,
    Floating,
    FixedThenFloating,
    FloatingThenFixed,
    FixedThenFixed,
    Suffled,
}

impl TryFrom<String> for RateType {
    type Error = AtlasError;
    fn try_from(s: String) -> Result<Self> {
        match s.as_str() {
            "Fixed" => Ok(RateType::Fixed),
            "Floating" => Ok(RateType::Floating),
            "FixedThenFloating" => Ok(RateType::FixedThenFloating),
            "FloatingThenFixed" => Ok(RateType::FloatingThenFixed),
            "FixedThenFixed" => Ok(RateType::FixedThenFixed),
            "Suffled" => Ok(RateType::Suffled),
            _ => Err(AtlasError::InvalidValueErr(format!(
                "Invalid rate type: {}",
                s
            ))),
        }
    }
}

impl From<RateType> for String {
    fn from(rate_type: RateType) -> Self {
        match rate_type {
            RateType::Fixed => "Fixed".to_string(),
            RateType::Floating => "Floating".to_string(),
            RateType::FixedThenFloating => "FixedThenFloating".to_string(),
            RateType::FloatingThenFixed => "FloatingThenFixed".to_string(),
            RateType::FixedThenFixed => "FixedThenFixed".to_string(),
            RateType::Suffled => "Suffled".to_string(),
        }
    }
}

impl Display for RateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}