use super::traits::DayCountProvider;
use crate::time::date::Date;

/// # ActualActual
/// Actual/Actual day count convention.
/// Calculates the day count fraction according to the formula:
/// $$
/// \frac{ActualDays_of_leap_years}{366} + \frac{ActualDays_of_non_leap_years}{365}
/// $$
/// where ActualDays of leap years is the number of days between the start date and the end date in leap years
/// and ActualDays of non-leap years is the number of days between the start date and the end date in non-leap years.
/// # Example
/// ```
/// use rustatlas::prelude::*;
///
/// let start = Date::new(2020, 1, 1);
/// let end = Date::new(2020, 2, 1);
/// assert_eq!(ActualActual::day_count(start, end), 31);
/// assert_eq!(ActualActual::year_fraction(start, end), 31.0 / 366.0);
/// ```

pub struct ActualActual;

fn days_in_year(year: i32) -> i32 {
    if Date::is_leap_year(year as i32) {
        return 366;
    } else {
        return 365;
    }
}

impl DayCountProvider for ActualActual {
    fn day_count(start: Date, end: Date) -> i64 {
        return end - start;
    }

    fn year_fraction(start: Date, end: Date) -> f64 {
        let days = ActualActual::day_count(start, end);

        let y1 = start.year() as i32;
        let y2 = end.year() as i32;

        if y1 == y2 {
            return days as f64 / days_in_year(y1) as f64;
        } else {
            if y2 > y1 {
                let mut sum = 0.0;
                sum += (Date::new(y1 + 1 as i32, 1, 1) - start) as f64
                    / days_in_year(y1 as i32) as f64;
                for _year in y1 + 1..y2 - 1 {
                    sum += 1.0;
                }
                sum += (end - Date::new(y2 as i32, 1, 1)) as f64 / days_in_year(y2 as i32) as f64;

                return sum;
            } else {
                let mut sum = 0.0;
                sum -=
                    (Date::new(y2 + 1 as i32, 1, 1) - end) as f64 / days_in_year(y2 as i32) as f64;
                for _year in y2 + 1..y1 - 1 {
                    sum -= 1.0;
                }
                sum -= (start - Date::new(y1 as i32, 1, 1)) as f64 / days_in_year(y1 as i32) as f64;
                return sum;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::time::{
        date::Date,
        daycounters::{actualactual::ActualActual, traits::DayCountProvider},
    };

    #[test]
    fn test_actualactual_day_count() {
        let start = Date::new(2020, 1, 1);
        let end = Date::new(2020, 2, 1);
        assert_eq!(ActualActual::day_count(start, end), 31);
    }

    #[test]
    fn test_actualactual_year_fraction() {
        let start = Date::new(2020, 1, 1);
        let end = Date::new(2020, 2, 1);
        assert_eq!(ActualActual::year_fraction(start, end), 31.0 / 366.0);
    }

    #[test]
    fn test_actualactual_year_fraction2() {
        let start = Date::new(2020, 1, 1);
        let end = Date::new(2021, 1, 1);
        assert_eq!(ActualActual::year_fraction(start, end), 1.0);
    }

    #[test]
    fn test_actualactual_year_fraction3() {
        let start = Date::new(2021, 1, 1);
        let end = Date::new(2020, 1, 1);
        assert_eq!(ActualActual::year_fraction(start, end), -1.0);
    }

    #[test]
    fn test_actualactual_same_day() {
        let date = Date::new(2020, 5, 15);
        assert_eq!(ActualActual::day_count(date, date), 0);
        assert_eq!(ActualActual::year_fraction(date, date), 0.0);
    }

    #[test]
    fn test_actualactual_across_leap_and_non_leap_years() {
        let start = Date::new(2019, 7, 1);
        let end = Date::new(2020, 7, 1);
        let expected = (Date::new(2020, 1, 1) - start) as f64 / 365.0
            + (end - Date::new(2020, 1, 1)) as f64 / 366.0;
        assert!((ActualActual::year_fraction(start, end) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_actualactual_reverse_across_leap_and_non_leap_years() {
        let start = Date::new(2020, 7, 1);
        let end = Date::new(2019, 7, 1);
        let expected = -((Date::new(2020, 1, 1) - end) as f64 / 365.0
            + (start - Date::new(2020, 1, 1)) as f64 / 366.0);
        assert!((ActualActual::year_fraction(start, end) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_actualactual_full_non_leap_year() {
        let start = Date::new(2021, 1, 1);
        let end = Date::new(2022, 1, 1);
        assert_eq!(ActualActual::year_fraction(start, end), 1.0);
    }

    #[test]
    fn test_actualactual_partial_year_non_leap() {
        let start = Date::new(2021, 6, 1);
        let end = Date::new(2021, 12, 1);
        let expected = (end - start) as f64 / 365.0;
        assert!((ActualActual::year_fraction(start, end) - expected).abs() < 1e-10);
    }
}
