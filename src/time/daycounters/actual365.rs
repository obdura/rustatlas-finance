use super::traits::DayCountProvider;
use crate::time::date::Date;

/// # Actual365 (Fixed)
/// Actual/365 day count convention.
/// Calculates the day count fraction according to the formula:
/// $$
/// \frac{ActualDays}{365}
/// $$
/// where ActualDays is the number of days between the start date and the end date.
/// # Example
/// ```
/// use rustatlas::prelude::*;
///
/// let start = Date::new(2020, 1, 1);
/// let end = Date::new(2020, 2, 1);
/// assert_eq!(Actual365::day_count(start, end), 31);
/// assert_eq!(Actual365::year_fraction(start, end), 31.0 / 365.0);
/// ```
pub struct Actual365;

impl DayCountProvider for Actual365 {
    fn day_count(start: Date, end: Date) -> i64 {
        return end - start;
    }

    fn year_fraction(start: Date, end: Date) -> f64 {
        return Actual365
        ::day_count(start, end) as f64 / 365.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_count_same_day() {
        let date = Date::new(2023, 5, 10);
        assert_eq!(Actual365::day_count(date, date), 0);
        assert_eq!(Actual365::year_fraction(date, date), 0.0);
    }

    #[test]
    fn test_day_count_one_day() {
        let start = Date::new(2023, 5, 10);
        let end = Date::new(2023, 5, 11);
        assert_eq!(Actual365::day_count(start, end), 1);
        assert_eq!(Actual365::year_fraction(start, end), 1.0 / 365.0);
    }

    #[test]
    fn test_day_count_across_years() {
        let start = Date::new(2022, 12, 31);
        let end = Date::new(2023, 1, 1);
        assert_eq!(Actual365::day_count(start, end), 1);
        assert_eq!(Actual365::year_fraction(start, end), 1.0 / 365.0);
    }

    #[test]
    fn test_day_count_full_year_non_leap() {
        let start = Date::new(2021, 1, 1);
        let end = Date::new(2022, 1, 1);
        assert_eq!(Actual365::day_count(start, end), 365);
        assert_eq!(Actual365::year_fraction(start, end), 1.0);
    }

    #[test]
    fn test_day_count_full_year_leap() {
        let start = Date::new(2019, 2, 28);
        let end = Date::new(2020, 2, 28);
        assert_eq!(Actual365::day_count(start, end), 365);
        assert_eq!(Actual365::year_fraction(start, end), 1.0);
    }

    #[test]
    fn test_day_count_reverse_dates() {
        let start = Date::new(2023, 5, 11);
        let end = Date::new(2023, 5, 10);
        assert_eq!(Actual365::day_count(start, end), -1);
        assert_eq!(Actual365::year_fraction(start, end), -1.0 / 365.0);
    }
}
