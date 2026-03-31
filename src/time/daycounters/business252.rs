use super::traits::DayCountProvider;
use crate::time::{
    calendar::Calendar,
    calendars::{
        brazil::{Brazil, BrazilMarket},
        traits::ImplCalendar,
    },
    date::Date,
};

/// # Business252
/// Business/252 day count convention.
/// Calculates the number of business days between two dates.
/// # Details
/// The number of business days between two dates is the number of days that are
/// considered business days in the calendar.
/// 
/// It is Considere Start Date + 1 to End Date.
/// 
/// # Example
/// ```
/// use rustatlas::prelude::*;
///
/// let start = Date::new(2020, 1, 1);
/// let end = Date::new(2020, 2, 1);
/// assert_eq!(Business252::day_count(start, end), 22);
/// assert_eq!(Business252::year_fraction(start, end), 22.0 / 252.0);
/// ```
pub struct Business252;

impl DayCountProvider for Business252 {
    fn day_count(start: Date, end: Date) -> i64 {
        let calendar = Calendar::Brazil(Brazil::new(BrazilMarket::Settlement));

        if end < start {
            return -(calendar.business_day_list(end+1, start).len() as i64);
        } else {
            return calendar.business_day_list(start+1, end).len() as i64;
        }
    }

    fn year_fraction(start: Date, end: Date) -> f64 {
        Self::day_count(start, end) as f64 / 252.0
    }
}

#[cfg(test)]
mod tests {
    use crate::time::{
        date::Date,
        daycounters::{business252::Business252, traits::DayCountProvider},
    };

    #[test]
    fn test_business252() {
        let start = Date::new(2020, 1, 1);
        let end = Date::new(2020, 2, 1);
        assert_eq!(Business252::day_count(start, end), 22);
        assert_eq!(Business252::year_fraction(start, end), 22.0 / 252.0);
    }

    #[test]
    fn test_business252_reverse_dates() {
        let start = Date::new(2020, 2, 1);
        let end = Date::new(2020, 1, 1);
        assert_eq!(Business252::day_count(start, end), -22);
        assert_eq!(Business252::year_fraction(start, end), -22.0 / 252.0);
    }

    #[test]
    fn test_business252_same_day() {
        let date = Date::new(2020, 1, 1);
        assert_eq!(Business252::day_count(date, date), 0);
        assert_eq!(Business252::year_fraction(date, date), 0.0);
    }

    #[test]
    fn test_business252_weekend() {
        // Jan 4, 2020 is a Saturday, Jan 5, 2020 is a Sunday
        let start = Date::new(2020, 1, 4);
        let end = Date::new(2020, 1, 6);
        // Only Jan 6 is a business day
        assert_eq!(Business252::day_count(start, end), 1);
        assert_eq!(Business252::year_fraction(start, end), 1.0 / 252.0);
    }

    #[test]
    fn test_business252_holiday() {
        // Assuming Jan 1, 2020 is a holiday in Brazil
        let start = Date::new(2019, 12, 30);
        let end = Date::new(2020, 1, 2);
        // Only Dec 31, 2019 and Jan 2, 2020 are business days
        assert_eq!(Business252::day_count(start, end), 2);
        assert_eq!(Business252::year_fraction(start, end), 2.0 / 252.0);
    }

    #[test]
    fn test_business252_holiday_2() {
        // Assuming Jan 1, 2020 is a holiday in Brazil
        let start = Date::new(2026, 3, 24);
        let end = Date::new(2026, 4, 1);
        assert_eq!(Business252::day_count(start, end), 6);
        assert_eq!(Business252::year_fraction(start, end), 6.0 / 252.0);
    }
}
