use std::any::Any;
use std::sync::Arc;

use crate::{
    rates::traits::{HasReferenceDate, YieldProvider},
    time::{date::Date, period::Period},
    utils::errors::Result,
};
/// # AdvanceTermStructureInTime
/// Trait for advancing in time a given object. Returns a represation of the object
/// as it would be after the given period.
/// 
/// * advance_to_period - Advance to a given period.
/// * advance_to_date - Advance to a given date.
pub trait AdvanceTermStructureInTime {
    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait>>;
    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait>>;
}

/// # YieldTermStructureTrait
/// Trait that defines a yield term structure.
///
/// ## Note
/// This trait is a combination of the following traits:
/// - YieldProvider
/// - HasReferenceDate
/// - AdvanceTermStructureInTime
/// - Send
///
/// Send is required to be able to send the trait to another thread.
pub trait YieldTermStructureTrait:
    YieldProvider + HasReferenceDate + AdvanceTermStructureInTime + Send + Sync
{
    /// Método para downcast seguro
    fn as_any(&self) -> &dyn Any;
}
