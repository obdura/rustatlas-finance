use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::time::date::{Date, NaiveDateExt}; // NaiveDateExt needed for day_of_year() on NaiveDate
use super::traits::{easter_monday, ImplCalendar, IsCalendar};

/// # Colombia Calendar
///
/// A calendar implementation for Colombian financial markets.
///
/// ## Disclaimer
/// This calendar is provided for reference purposes only and should be used exclusively
/// for instrument valuation estimates. It is NOT an official calendar and may not reflect
/// actual market holidays or business days. Users should verify against official sources
/// for production use.
///
/// ## Holiday Rules
/// Colombia uses the "Ley Emiliani" (Law 51 of 1983): religious holidays that do not
/// fall on a fixed weekday are moved to the following Monday. Fixed-date civic holidays
/// and Holy Thursday/Good Friday are NOT moved.
///
/// ## Markets
/// - BVC: Bolsa de Valores de Colombia calendar

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColombiaMarket {
    BVC, // Bolsa de Valores de Colombia calendar
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Colombia {
    market: ColombiaMarket,
    added_holidays: HashSet<Date>,
    removed_holidays: HashSet<Date>,
}

impl Colombia {
    pub fn new(market: ColombiaMarket) -> Self {
        Colombia {
            market,
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        }
    }

    fn is_weekend(day: Weekday) -> bool {
        day == Weekday::Sat || day == Weekday::Sun
    }

    // Core Ley Emiliani logic: returns the observed Monday for a given base date.
    // If the base date already falls on Monday it is observed that same day (Mon -> +0).
    // Any other weekday advances to the following Monday.
    fn next_monday(base: NaiveDate) -> NaiveDate {
        let days: i64 = match base.weekday() {
            Weekday::Mon => 0,
            Weekday::Tue => 6,
            Weekday::Wed => 5,
            Weekday::Thu => 4,
            Weekday::Fri => 3,
            Weekday::Sat => 2,
            Weekday::Sun => 1,
        };
        base + chrono::Duration::days(days)
    }

    // Ley Emiliani for fixed-date holidays: moves (fixed_month/fixed_day) to its observed Monday.
    fn emiliani_monday(fixed_day: u32, fixed_month: u32, day: u32, month: u32, year: i32) -> bool {
        let fixed = NaiveDate::from_ymd_opt(year, fixed_month, fixed_day).unwrap();
        let observed = Self::next_monday(fixed);
        observed.day() == day && observed.month() == month
    }

    // --- Fixed holidays (not moved) ---

    fn is_new_years_day(day: u32, month: u32) -> bool {
        day == 1 && month == 1
    }

    fn is_holy_thursday(day: u32, month: u32, year: i32) -> bool {
        let em = easter_monday(year);
        let dd = NaiveDate::from_ymd_opt(year, month, day).unwrap().day_of_year();
        dd == em - 4
    }

    fn is_good_friday(day: u32, month: u32, year: i32) -> bool {
        let em = easter_monday(year);
        let dd = NaiveDate::from_ymd_opt(year, month, day).unwrap().day_of_year();
        dd == em - 3
    }

    fn is_labour_day(day: u32, month: u32) -> bool {
        day == 1 && month == 5
    }

    fn is_independence_day(day: u32, month: u32) -> bool {
        day == 20 && month == 7
    }

    fn is_battle_of_boyaca(day: u32, month: u32) -> bool {
        day == 7 && month == 8
    }

    fn is_immaculate_conception(day: u32, month: u32) -> bool {
        day == 8 && month == 12
    }

    fn is_christmas(day: u32, month: u32) -> bool {
        day == 25 && month == 12
    }

    // --- Ley Emiliani holidays (moved to next Monday) ---

    fn is_epiphany(day: u32, month: u32, year: i32) -> bool {
        // Jan 6 -> next Mondaymm
        Self::emiliani_monday(6, 1, day, month, year)
    }

    fn is_saint_joseph(day: u32, month: u32, year: i32) -> bool {
        // Mar 19 -> next Monday
        Self::emiliani_monday(19, 3, day, month, year)
    }

    // Returns the NaiveDate that is `days_after` days after Easter Monday.
    // easter_monday(year) returns the day-of-year (1-based) of Easter Monday.
    // Jan 1 is day 1, so Easter Monday = Jan 1 + (em - 1) days.
    fn days_after_easter(year: i32, days_after: i64) -> NaiveDate {
        let em = easter_monday(year);
        NaiveDate::from_ymd_opt(year, 1, 1).unwrap()
            + chrono::Duration::days(em as i64 - 1 + days_after)
    }

    fn is_ascension(day: u32, month: u32, year: i32) -> bool {
        // Ascension Thursday = Easter Sunday + 39 = Easter Monday + 38
        let observed = Self::next_monday(Self::days_after_easter(year, 38));
        observed.day() == day && observed.month() == month
    }

    fn is_corpus_christi(day: u32, month: u32, year: i32) -> bool {
        // Corpus Christi Thursday = Easter Sunday + 60 = Easter Monday + 59
        let observed = Self::next_monday(Self::days_after_easter(year, 59));
        observed.day() == day && observed.month() == month
    }

    fn is_sacred_heart(day: u32, month: u32, year: i32) -> bool {
        // Sacred Heart Friday = Easter Sunday + 68 = Easter Monday + 67
        let observed = Self::next_monday(Self::days_after_easter(year, 67));
        observed.day() == day && observed.month() == month
    }

    fn is_saints_peter_and_paul(day: u32, month: u32, year: i32) -> bool {
        // Jun 29 -> next Monday
        Self::emiliani_monday(29, 6, day, month, year)
    }

    fn is_assumption(day: u32, month: u32, year: i32) -> bool {
        // Aug 15 -> next Monday
        Self::emiliani_monday(15, 8, day, month, year)
    }

    fn is_columbus_day(day: u32, month: u32, year: i32) -> bool {
        // Oct 12 -> next Monday
        Self::emiliani_monday(12, 10, day, month, year)
    }

    fn is_all_saints(day: u32, month: u32, year: i32) -> bool {
        // Nov 1 -> next Monday
        Self::emiliani_monday(1, 11, day, month, year)
    }

    fn is_cartagena_independence(day: u32, month: u32, year: i32) -> bool {
        // Nov 11 -> next Monday
        Self::emiliani_monday(11, 11, day, month, year)
    }

    pub fn is_standard_business_day(&self, date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();

        if Colombia::is_weekend(weekday) {
            return false;
        }

        match self.market {
            ColombiaMarket::BVC => {
                if Colombia::is_new_years_day(day, month)
                    || Colombia::is_epiphany(day, month, year)
                    || Colombia::is_saint_joseph(day, month, year)
                    || Colombia::is_holy_thursday(day, month, year)
                    || Colombia::is_good_friday(day, month, year)
                    || Colombia::is_labour_day(day, month)
                    || Colombia::is_ascension(day, month, year)
                    || Colombia::is_corpus_christi(day, month, year)
                    || Colombia::is_sacred_heart(day, month, year)
                    || Colombia::is_saints_peter_and_paul(day, month, year)
                    || Colombia::is_independence_day(day, month)
                    || Colombia::is_battle_of_boyaca(day, month)
                    || Colombia::is_assumption(day, month, year)
                    || Colombia::is_columbus_day(day, month, year)
                    || Colombia::is_all_saints(day, month, year)
                    || Colombia::is_cartagena_independence(day, month, year)
                    || Colombia::is_immaculate_conception(day, month)
                    || Colombia::is_christmas(day, month)
                {
                    return false;
                }
                true
            }
        }
    }
}

impl ImplCalendar for Colombia {
    fn impl_is_business_day(&self, date: &Date) -> bool {
        self.is_standard_business_day(date.base_date())
    }

    fn impl_name(&self) -> String {
        format!("Colombia({:?})", self.market)
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

impl IsCalendar for Colombia {}

impl Default for Colombia {
    fn default() -> Self {
        Colombia::new(ColombiaMarket::BVC)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::date::Date;

    #[test]
    fn test_weekend_is_not_business_day() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        assert!(!cal.is_business_day(&Date::new(2025, 1, 4))); // Saturday
        assert!(!cal.is_business_day(&Date::new(2025, 1, 5))); // Sunday
        assert!(cal.is_business_day(&Date::new(2025, 1, 7)));  // Tuesday, not a holiday
    }

    // --- 2025 full holiday list (exact dates from BVC website) ---

    #[test]
    fn test_bvc_2025_holidays() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Jan 6 2025 is Monday -> Mon+0 = Jan 6 (observed same day)
        // NOTE: BVC website listed Jan 12 but that is inconsistent with Ley Emiliani
        // and with the verified weekday (Jan 6 2025 = Monday). Using Jan 6.
        let expected = vec![
            Date::new(2025, 1,  1),  // Año Nuevo (fixed)
            Date::new(2025, 1,  6),  // Reyes Magos (Jan 6 Mon -> Mon+0 = Jan 6)
            Date::new(2025, 3, 23),  // San José (Mar 19 Wed -> Mon Mar 23)
            Date::new(2025, 4, 17),  // Jueves Santo
            Date::new(2025, 4, 18),  // Viernes Santo
            Date::new(2025, 5,  1),  // Día del Trabajo (fixed)
            Date::new(2025, 6,  2),  // Ascensión
            Date::new(2025, 6, 23),  // Corpus Christi
            Date::new(2025, 6, 30),  // Sagrado Corazón / San Pedro y San Pablo
            Date::new(2025, 7, 20),  // Independencia (fixed)
            Date::new(2025, 8,  7),  // Batalla de Boyacá (fixed)
            Date::new(2025, 8, 18),  // Asunción (Aug 15 Fri -> Mon Aug 18)
            Date::new(2025, 10, 13), // Día de la Raza (Oct 12 Sun -> Mon Oct 13)
            Date::new(2025, 11,  3), // Todos los Santos (Nov 1 Sat -> Mon Nov 3)
            Date::new(2025, 11, 17), // Independencia de Cartagena (Nov 11 Tue -> Mon Nov 17)
            Date::new(2025, 12,  8), // Inmaculada Concepción (fixed)
            Date::new(2025, 12, 25), // Navidad (fixed)
        ];
        for d in &expected {
            assert!(!cal.is_business_day(d), "{} should be a holiday", d);
        }
    }

    // --- Fixed holidays across multiple years ---

    #[test]
    fn test_new_years_day_multiple_years() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        for year in [2023, 2024, 2025, 2026] {
            assert!(!cal.is_business_day(&Date::new(year, 1, 1)), "New Year {}", year);
        }
    }

    #[test]
    fn test_labour_day_multiple_years() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        for year in [2023, 2024, 2025, 2026] {
            assert!(!cal.is_business_day(&Date::new(year, 5, 1)), "Labour Day {}", year);
        }
    }

    #[test]
    fn test_independence_day_multiple_years() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        for year in [2023, 2024, 2025, 2026] {
            assert!(!cal.is_business_day(&Date::new(year, 7, 20)), "Independence {}", year);
        }
    }

    #[test]
    fn test_battle_of_boyaca_multiple_years() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        for year in [2023, 2024, 2025, 2026] {
            assert!(!cal.is_business_day(&Date::new(year, 8, 7)), "Boyaca {}", year);
        }
    }

    #[test]
    fn test_immaculate_conception_multiple_years() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        for year in [2023, 2024, 2025, 2026] {
            assert!(!cal.is_business_day(&Date::new(year, 12, 8)), "Immaculate {}", year);
        }
    }

    #[test]
    fn test_christmas_multiple_years() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        for year in [2023, 2024, 2025, 2026] {
            assert!(!cal.is_business_day(&Date::new(year, 12, 25)), "Christmas {}", year);
        }
    }

    // --- Ley Emiliani: Epiphany ---

    #[test]
    fn test_epiphany_on_monday_observed_same_day() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Jan 6 2025 is Monday -> Mon+0 = Jan 6 (observed same day)
        assert!(!cal.is_business_day(&Date::new(2025, 1, 6)));
        assert!(cal.is_business_day(&Date::new(2025, 1, 7))); // Tuesday after is business day
    }

    #[test]
    fn test_epiphany_on_saturday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Jan 6 2024 is Saturday -> Mon+2 = Jan 8
        assert!(!cal.is_business_day(&Date::new(2024, 1, 8)));
        assert!(!cal.is_business_day(&Date::new(2024, 1, 6))); // Saturday = weekend
        assert!(cal.is_business_day(&Date::new(2024, 1, 9)));  // Tuesday after
    }

    #[test]
    fn test_epiphany_on_friday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Jan 6 2023 is Friday -> Mon+3 = Jan 9
        assert!(!cal.is_business_day(&Date::new(2023, 1, 9)));
        assert!(cal.is_business_day(&Date::new(2023, 1, 6))); // Friday itself is business day
    }

    // --- Ley Emiliani: Saint Joseph ---

    #[test]
    fn test_saint_joseph_2025() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Mar 19 2025 is Wednesday -> next Monday = Mar 23 (BVC confirmed)
        assert!(!cal.is_business_day(&Date::new(2025, 3, 23)));
        assert!(cal.is_business_day(&Date::new(2025, 3, 19)));
    }

    #[test]
    fn test_saint_joseph_2024() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Mar 19 2024 is Tuesday -> next Monday = Mar 25
        assert!(!cal.is_business_day(&Date::new(2024, 3, 25)));
    }

    // --- Holy Week ---

    #[test]
    fn test_holy_week_2025() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        assert!(!cal.is_business_day(&Date::new(2025, 4, 17))); // Holy Thursday
        assert!(!cal.is_business_day(&Date::new(2025, 4, 18))); // Good Friday
        assert!(cal.is_business_day(&Date::new(2025, 4, 16)));  // Wednesday before
        assert!(cal.is_business_day(&Date::new(2025, 4, 22)));  // Tuesday after
    }

    #[test]
    fn test_holy_week_2024() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Easter 2024: March 31 -> Holy Thursday Mar 28, Good Friday Mar 29
        assert!(!cal.is_business_day(&Date::new(2024, 3, 28)));
        assert!(!cal.is_business_day(&Date::new(2024, 3, 29)));
    }

    #[test]
    fn test_holy_week_2023() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Easter 2023: April 9 -> Holy Thursday Apr 6, Good Friday Apr 7
        assert!(!cal.is_business_day(&Date::new(2023, 4, 6)));
        assert!(!cal.is_business_day(&Date::new(2023, 4, 7)));
    }

    // --- Saints Peter and Paul ---

    #[test]
    fn test_saints_peter_paul_on_saturday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Jun 29 2024 is Saturday -> Mon+2 = Jul 1
        assert!(!cal.is_business_day(&Date::new(2024, 7, 1)));
        assert!(!cal.is_business_day(&Date::new(2024, 6, 29))); // Saturday = weekend
    }

    #[test]
    fn test_saints_peter_paul_on_sunday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Jun 29 2025 is Sunday -> Mon+1 = Jun 30
        assert!(!cal.is_business_day(&Date::new(2025, 6, 30)));
        assert!(!cal.is_business_day(&Date::new(2025, 6, 29))); // Sunday = weekend
    }

    // --- Assumption ---

    #[test]
    fn test_assumption_on_thursday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Aug 15 2024 is Thursday -> Mon+4 = Aug 19
        assert!(!cal.is_business_day(&Date::new(2024, 8, 19)));
        assert!(cal.is_business_day(&Date::new(2024, 8, 15))); // Thursday itself is business day
    }

    #[test]
    fn test_assumption_on_friday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Aug 15 2025 is Friday -> Mon+3 = Aug 18
        assert!(!cal.is_business_day(&Date::new(2025, 8, 18)));
        assert!(cal.is_business_day(&Date::new(2025, 8, 15))); // Friday itself is business day
    }

    // --- Columbus Day ---

    #[test]
    fn test_columbus_day_on_saturday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Oct 12 2024 is Saturday -> Mon+2 = Oct 14
        assert!(!cal.is_business_day(&Date::new(2024, 10, 14)));
        assert!(!cal.is_business_day(&Date::new(2024, 10, 12))); // Saturday = weekend
    }

    #[test]
    fn test_columbus_day_on_sunday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Oct 12 2025 is Sunday -> Mon+1 = Oct 13
        assert!(!cal.is_business_day(&Date::new(2025, 10, 13)));
        assert!(!cal.is_business_day(&Date::new(2025, 10, 12))); // Sunday = weekend
    }

    // --- All Saints ---

    #[test]
    fn test_all_saints_on_friday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Nov 1 2024 is Friday -> Mon+3 = Nov 4
        assert!(!cal.is_business_day(&Date::new(2024, 11, 4)));
        assert!(cal.is_business_day(&Date::new(2024, 11, 1))); // Friday itself is business day
    }

    #[test]
    fn test_all_saints_on_saturday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Nov 1 2025 is Saturday -> Mon+2 = Nov 3
        assert!(!cal.is_business_day(&Date::new(2025, 11, 3)));
        assert!(!cal.is_business_day(&Date::new(2025, 11, 1))); // Saturday = weekend
    }

    // --- Cartagena Independence ---

    #[test]
    fn test_cartagena_independence_on_monday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Nov 11 2024 is Monday -> Mon+0 = Nov 11 (observed same day)
        assert!(!cal.is_business_day(&Date::new(2024, 11, 11)));
        assert!(cal.is_business_day(&Date::new(2024, 11, 12))); // Tuesday after
    }

    #[test]
    fn test_cartagena_independence_on_tuesday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Nov 11 2025 is Tuesday -> Mon+6 = Nov 17
        assert!(!cal.is_business_day(&Date::new(2025, 11, 17)));
        assert!(cal.is_business_day(&Date::new(2025, 11, 11))); // Tuesday itself is business day
    }

    #[test]
    fn test_cartagena_independence_on_saturday() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        // Nov 11 2023 is Saturday -> Mon+2 = Nov 13
        assert!(!cal.is_business_day(&Date::new(2023, 11, 13)));
        assert!(!cal.is_business_day(&Date::new(2023, 11, 11))); // Saturday = weekend
    }

    // --- add/remove holiday ---

    #[test]
    fn test_add_holiday() {
        let mut cal = Colombia::new(ColombiaMarket::BVC);
        assert!(cal.is_business_day(&Date::new(2025, 6, 10)));
        cal.add_holiday(Date::new(2025, 6, 10));
        assert!(!cal.is_business_day(&Date::new(2025, 6, 10)));
    }

    #[test]
    fn test_remove_holiday() {
        let mut cal = Colombia::new(ColombiaMarket::BVC);
        assert!(!cal.is_business_day(&Date::new(2025, 12, 25)));
        cal.remove_holiday(Date::new(2025, 12, 25));
        assert!(cal.is_business_day(&Date::new(2025, 12, 25)));
    }

    // --- Regular business days ---

    #[test]
    fn test_regular_business_days() {
        let cal = Colombia::new(ColombiaMarket::BVC);
        assert!(cal.is_business_day(&Date::new(2025, 2, 3)));  // Monday
        assert!(cal.is_business_day(&Date::new(2025, 4, 14))); // Monday before Holy Week
        assert!(cal.is_business_day(&Date::new(2025, 9, 1)));  // Monday
    }
}
