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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign() {
        assert_eq!(Side::Pay.sign(), -1.0);
        assert_eq!(Side::Receive.sign(), 1.0);
    }

    #[test]
    fn test_inverse() {
        assert_eq!(Side::Pay.inverse(), Side::Receive);
        assert_eq!(Side::Receive.inverse(), Side::Pay);
    }

    #[test]
    fn test_try_from_string() {
        assert_eq!(Side::try_from("Pay".to_string()).unwrap(), Side::Pay);
        assert_eq!(Side::try_from("Receive".to_string()).unwrap(), Side::Receive);
        assert!(Side::try_from("Other".to_string()).is_err());
    }

    #[test]
    fn test_from_side_to_string() {
        let pay_str: String = Side::Pay.into();
        let receive_str: String = Side::Receive.into();
        assert_eq!(pay_str, "Pay");
        assert_eq!(receive_str, "Receive");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Side::Pay), "Pay");
        assert_eq!(format!("{}", Side::Receive), "Receive");
    }
}