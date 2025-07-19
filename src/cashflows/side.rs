use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize};
use crate::utils::errors::{AtlasError,Result};

/// # Side
/// Enum that represents the side of a cashflow.
#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Side {
    Pay,
    Receive,
}

impl Side {
    pub fn sign(&self) -> f64 {
        match self {
            Side::Pay => -1.0,
            Side::Receive => 1.0,
        }
    }

    pub fn inverse(&self) -> Side {
        match self {
            Side::Pay => Side::Receive,
            Side::Receive => Side::Pay,
        }
    }
}

impl TryFrom<String> for Side {
    type Error = AtlasError;

    fn try_from(s: String) -> Result<Self> {
        match s.as_str() {
            "Pay" => Ok(Side::Pay),
            "Receive" => Ok(Side::Receive),
            _ => Err(AtlasError::InvalidValueErr(format!("Invalid side: {}", s))),
        }
    }
}

// Implement the conversion from &str to Side
impl<'de> Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Side, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Pay" => Ok(Side::Pay),
            "Receive" => Ok(Side::Receive),
            "pay" => Ok(Side::Pay),
            "receive" => Ok(Side::Receive),
            "ACT" => Ok(Side::Receive),
            "PAS" => Ok(Side::Pay),
            _ => Err(serde::de::Error::custom(format!("Invalid side: {}", s))),
        }
    }
}

impl From<Side> for String {
    fn from(side: Side) -> Self {
        match side {
            Side::Pay => "Pay".to_string(),
            Side::Receive => "Receive".to_string(),
        }
    }
}


impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Pay => write!(f, "Pay"),
            Side::Receive => write!(f, "Receive"),
        }
    }
}