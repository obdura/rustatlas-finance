use super::enums::Compounding;
use crate::{
    time::{date::Date, enums::Frequency},
    utils::errors::Result,
};

/// # HasReferenceDate
/// Implement this trait for a struct that has a reference date.
/// 
/// * reference_date - Get the reference date.
pub trait HasReferenceDate {
    fn reference_date(&self) -> Date;
}

/// # YieldProvider
/// Implement this trait for a struct that provides yield information.
/// 
/// * discount_factor - Get the discount factor for a given date.
/// * forward_rate - Get the forward rate between two dates.
/// 
pub trait YieldProvider: HasReferenceDate {
    fn discount_factor(&self, date: Date) -> Result<f64>;
    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<f64>;
}
