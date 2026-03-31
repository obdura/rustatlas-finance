use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::time::date::{Date, NaiveDateExt};
use super::traits::{easter_monday, ImplCalendar, IsCalendar};

/// # Mexico Calendar
///
/// A calendar implementation for Mexican financial markets.
///
/// ## Disclaimer
/// This calendar is provided for reference purposes only and should be used exclusively
/// for instrument valuation estimates. It is NOT an official calendar and may not reflect
/// actual market holidays or business days. Users should verify against official sources
/// for production use.
///
/// ## Markets
/// - BMV: Bolsa Mexicana de Valores calendar

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MexicoMarket {
    BMV,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mexico {
    market: MexicoMarket,
    added_holidays: HashSet<Date>,
    removed_holidays: HashSet<Date>,
}

impl Mexico {
    pub fn new(market: MexicoMarket) -> Self {
        Mexico {
            market,
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        }
    }

    fn is_weekend(day: Weekday) -> bool {
        day == Weekday::Sat || day == Weekday::Sun
    }

    fn is_new_years_day(day: u32, month: u32) -> bool {
        day == 1 && month == 1
    }

    // Constitution Day: Feb 5 until 2005, first Monday of Feb from 2006
    fn is_constitution_day(day: u32, month: u32, year: i32) -> bool {
        if month != 2 {
            return false;
        }
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        (year <= 2005 && day == 5)
            || (year >= 2006 && day <= 7 && w == Weekday::Mon)
    }

    // Birthday of Benito Juarez: Mar 21 until 2005, third Monday of Mar from 2006
    fn is_benito_juarez(day: u32, month: u32, year: i32) -> bool {
        if month != 3 {
            return false;
        }
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        (year <= 2005 && day == 21)
            || (year >= 2006 && day >= 15 && day <= 21 && w == Weekday::Mon)
    }

    // Holy Thursday: four days before Easter Monday (em - 4 in day-of-year)
    fn is_holy_thursday(day: u32, month: u32, year: i32) -> bool {
        let em = easter_monday(year);
        let dd = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .day_of_year();
        dd == em - 4
    }

    // Good Friday: three days before Easter Monday (em - 3 in day-of-year)
    fn is_good_friday(day: u32, month: u32, year: i32) -> bool {
        let em = easter_monday(year);
        let dd = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .day_of_year();
        dd == em - 3
    }

    fn is_labour_day(day: u32, month: u32) -> bool {
        day == 1 && month == 5
    }

    fn is_national_day(day: u32, month: u32) -> bool {
        day == 16 && month == 9
    }

    // Inauguration Day: Oct 1 every 6 years starting 2024
    fn is_inauguration_day(day: u32, month: u32, year: i32) -> bool {
        day == 1 && month == 10 && year >= 2024 && (year - 2024) % 6 == 0
    }

    fn is_all_souls_day(day: u32, month: u32) -> bool {
        day == 2 && month == 11
    }

    // Revolution Day: Nov 20 until 2005, third Monday of Nov from 2006
    fn is_revolution_day(day: u32, month: u32, year: i32) -> bool {
        if month != 11 {
            return false;
        }
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        (year <= 2005 && day == 20)
            || (year >= 2006 && day >= 15 && day <= 21 && w == Weekday::Mon)
    }

    fn is_our_lady_of_guadalupe(day: u32, month: u32) -> bool {
        day == 12 && month == 12
    }

    fn is_christmas_day(day: u32, month: u32) -> bool {
        day == 25 && month == 12
    }

    pub fn is_standard_business_day(&self, date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();

        if Mexico::is_weekend(weekday) {
            return false;
        }

        match self.market {
            MexicoMarket::BMV => {
                if Mexico::is_new_years_day(day, month)
                    || Mexico::is_constitution_day(day, month, year)
                    || Mexico::is_benito_juarez(day, month, year)
                    || Mexico::is_holy_thursday(day, month, year)
                    || Mexico::is_good_friday(day, month, year)
                    || Mexico::is_labour_day(day, month)
                    || Mexico::is_national_day(day, month)
                    || Mexico::is_inauguration_day(day, month, year)
                    || Mexico::is_all_souls_day(day, month)
                    || Mexico::is_revolution_day(day, month, year)
                    || Mexico::is_our_lady_of_guadalupe(day, month)
                    || Mexico::is_christmas_day(day, month)
                {
                    return false;
                }
                true
            }
        }
    }
}

impl ImplCalendar for Mexico {
    fn impl_is_business_day(&self, date: &Date) -> bool {
        self.is_standard_business_day(date.base_date())
    }

    fn impl_name(&self) -> String {
        format!("Mexico({:?})", self.market)
    }

    fn added_holidays(&self) -> HashSet<Date> {
        self.added_holidays.clone()
    }

    fn removed_holidays(&self) -> HashSet<Date> {
        self.removed_holidays.clone()
    }

    fn add_holiday(&mut self, date: Date) {
        self.added_holidays.insert(date);
    }

    fn remove_holiday(&mut self, date: Date) {
        self.removed_holidays.insert(date);
    }

    fn holiday_list(&self, from: Date, to: Date, include_weekends: bool) -> Vec<Date> {
        let mut holidays = vec![];
        let mut d = from;
        while d <= to {
            if self.is_holiday(&d) {
                holidays.push(d);
            }
            d = d + 1;
        }
        if include_weekends {
            holidays
        } else {
            holidays
                .into_iter()
                .filter(|d| !self.is_weekend(&d.weekday()))
                .collect()
        }
    }

    fn business_day_list(&self, from: Date, to: Date) -> Vec<Date> {
        let mut business_days = vec![];
        let mut d = from;
        while d <= to {
            if self.is_business_day(&d) {
                business_days.push(d);
            }
            d = d + 1;
        }
        business_days
    }
}

impl IsCalendar for Mexico {}

impl Default for Mexico {
    fn default() -> Self {
        Mexico::new(MexicoMarket::BMV)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::date::Date;

    #[test]
    fn test_weekend_is_not_business_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 6, 22))); // Saturday
        assert!(!cal.is_business_day(&Date::new(2024, 6, 23))); // Sunday
        assert!(cal.is_business_day(&Date::new(2024, 6, 24)));  // Monday
    }

    #[test]
    fn test_new_years_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 1, 1)));
    }

    #[test]
    fn test_constitution_day_fixed_pre_2006() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2005, 2, 5)));
    }

    #[test]
    fn test_constitution_day_floating_post_2006() {
        let cal = Mexico::new(MexicoMarket::BMV);
        // First Monday of February 2024 = Feb 5
        assert!(!cal.is_business_day(&Date::new(2024, 2, 5)));
        // Feb 6 is Tuesday -> business day
        assert!(cal.is_business_day(&Date::new(2024, 2, 6)));
    }

    #[test]
    fn test_benito_juarez_fixed_pre_2006() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2005, 3, 21)));
    }

    #[test]
    fn test_benito_juarez_floating_post_2006() {
        let cal = Mexico::new(MexicoMarket::BMV);
        // Third Monday of March 2024 = Mar 18
        assert!(!cal.is_business_day(&Date::new(2024, 3, 18)));
        assert!(cal.is_business_day(&Date::new(2024, 3, 19)));
    }

    #[test]
    fn test_holy_thursday_and_good_friday_2024() {
        let cal = Mexico::new(MexicoMarket::BMV);
        // Easter 2024: March 31 -> Holy Thursday Mar 28, Good Friday Mar 29
        assert!(!cal.is_business_day(&Date::new(2024, 3, 28)));
        assert!(!cal.is_business_day(&Date::new(2024, 3, 29)));
    }

    #[test]
    fn test_labour_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 5, 1)));
    }

    #[test]
    fn test_national_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 9, 16)));
    }

    #[test]
    fn test_inauguration_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        // Oct 1 2024: first inauguration day (2024 - 2024) % 6 == 0
        assert!(!cal.is_business_day(&Date::new(2024, 10, 1)));
        // Oct 1 2025: not an inauguration year
        assert!(cal.is_business_day(&Date::new(2025, 10, 1)));
        // Oct 1 2030: next inauguration year
        assert!(!cal.is_business_day(&Date::new(2030, 10, 1)));
    }

    #[test]
    fn test_all_souls_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 11, 2)));
    }

    #[test]
    fn test_revolution_day_fixed_pre_2006() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2005, 11, 20)));
    }

    #[test]
    fn test_revolution_day_floating_post_2006() {
        let cal = Mexico::new(MexicoMarket::BMV);
        // Third Monday of November 2024 = Nov 18
        assert!(!cal.is_business_day(&Date::new(2024, 11, 18)));
        assert!(cal.is_business_day(&Date::new(2024, 11, 19)));
    }

    #[test]
    fn test_our_lady_of_guadalupe() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 12, 12)));
    }

    #[test]
    fn test_christmas_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 12, 25)));
    }

    #[test]
    fn test_add_holiday() {
        let mut cal = Mexico::new(MexicoMarket::BMV);
        assert!(cal.is_business_day(&Date::new(2024, 6, 10)));
        cal.add_holiday(Date::new(2024, 6, 10));
        assert!(!cal.is_business_day(&Date::new(2024, 6, 10)));
    }

    #[test]
    fn test_remove_holiday() {
        let mut cal = Mexico::new(MexicoMarket::BMV);
        assert!(!cal.is_business_day(&Date::new(2024, 12, 25)));
        cal.remove_holiday(Date::new(2024, 12, 25));
        assert!(cal.is_business_day(&Date::new(2024, 12, 25)));
    }

    #[test]
    fn test_bmv_2024_holidays() {
        let cal = Mexico::new(MexicoMarket::BMV);
        let expected_holidays = vec![
            Date::new(2024, 1, 1),  // New Year's Day
            Date::new(2024, 2, 5),  // Constitution Day (first Monday Feb)
            Date::new(2024, 3, 18), // Benito Juarez (third Monday Mar)
            Date::new(2024, 3, 28), // Holy Thursday
            Date::new(2024, 3, 29), // Good Friday
            Date::new(2024, 5, 1),  // Labour Day
            Date::new(2024, 9, 16), // National Day
            Date::new(2024, 10, 1), // Inauguration Day
            Date::new(2024, 11, 2), // All Souls Day
            Date::new(2024, 11, 18),// Revolution Day (third Monday Nov)
            Date::new(2024, 12, 12),// Our Lady of Guadalupe
            Date::new(2024, 12, 25),// Christmas
        ];
        for d in expected_holidays {
            assert!(!cal.is_business_day(&d), "{:?} should be a holiday", d);
        }
    }

    #[test]
    fn test_non_holiday_is_business_day() {
        let cal = Mexico::new(MexicoMarket::BMV);
        assert!(cal.is_business_day(&Date::new(2024, 4, 15))); // Regular Monday
        assert!(cal.is_business_day(&Date::new(2024, 7, 15))); // Regular Monday
    }
}
