use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::{
    prelude::NaiveDateExt,
    time::{calendars::traits::easter_monday, date::Date},
};

use super::traits::{ImplCalendar, IsCalendar};

/// # UnitedStatesMarket
/// Defines the relevant market for the United States calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitedStatesMarket {
    Settlement,
    LiborImpact,
    Nyse,
    GovernmentBond,
    Nerc,
    FederalReserve,
    Sofr,
}

/// # UnitedStates
/// A calendar for the United States.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitedStates {
    market: UnitedStatesMarket,
    added_holidays: HashSet<Date>,
    removed_holidays: HashSet<Date>,
}

impl UnitedStates {
    pub fn new(market: UnitedStatesMarket) -> Self {
        UnitedStates {
            market,
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        }
    }

    fn is_weekend(day: Weekday) -> bool {
        day == Weekday::Sat || day == Weekday::Sun
    }

    fn is_washington_birthday(day: u32, month: u32, year: i32, weekday: Weekday) -> bool {
        match year {
            y if y >= 1971 => (day >= 15 && day <= 21) && weekday == Weekday::Mon && month == 2,
            _ => {
                (day == 22
                    || (day == 23 && weekday == Weekday::Mon)
                    || (day == 21 && weekday == Weekday::Fri))
                    && month == 2
            }
        }
    }

    fn is_memorial_day(day: u32, month: u32, year: i32, weekday: Weekday) -> bool {
        match year {
            y if y >= 1971 => day >= 25 && weekday == Weekday::Mon && month == 5,
            _ => {
                (day == 30
                    || (day == 31 && weekday == Weekday::Mon)
                    || (day == 29 && weekday == Weekday::Fri))
                    && month == 5
            }
        }
    }

    fn is_columbus_day(day: u32, month: u32, year: i32, weekday: Weekday) -> bool {
        // Segundo lunes de octubre
        (day >= 8 && day <= 14) && weekday == Weekday::Mon && month == 10 && year >= 1971
    }

    fn is_labor_day(day: u32, month: u32, _year: i32, weekday: Weekday) -> bool {
        day <= 7 && weekday == Weekday::Mon && month == 9
    }

    fn is_veterans_day(day: u32, month: u32, year: i32, weekday: Weekday) -> bool {
        if year <= 1970 || year >= 1978 {
            // 11 de noviembre, ajustado
            (day == 11
                || (day == 12 && weekday == Weekday::Mon)
                || (day == 10 && weekday == Weekday::Fri))
                && month == 11
        } else {
            // Cuarto lunes de octubre
            (day >= 22 && day <= 28) && weekday == Weekday::Mon && month == 10
        }
    }

    fn is_veterans_day_no_saturday(day: u32, month: u32, year: i32, weekday: Weekday) -> bool {
        if year <= 1970 || year >= 1978 {
            // 11 de noviembre, ajustado, sin mover del sábado al viernes
            (day == 11 || (day == 12 && weekday == Weekday::Mon)) && month == 11
        } else {
            // Cuarto lunes de octubre
            (day >= 22 && day <= 28) && weekday == Weekday::Mon && month == 10
        }
    }

    fn is_juneteenth(
        day: u32,
        month: u32,
        year: i32,
        weekday: Weekday,
        move_to_friday: bool,
    ) -> bool {
        // Declarado en 2021, pero observado desde 2022
        (day == 19
            || (day == 20 && weekday == Weekday::Mon)
            || (day == 18 && weekday == Weekday::Fri && move_to_friday))
            && month == 6
            && year >= 2022
    }

    fn market_nyse_business_days(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();
        let easter_monday = easter_monday(year);
        let day_of_year = date.day_of_year();

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        if
        // New Year's Day (possibly moved to Monday if on Sunday)
        ((day == 1 || (day == 2 && weekday == Weekday::Mon)) && month == 1)
        // Washington's Birthday (third Monday in February)
        || UnitedStates::is_washington_birthday(day, month, year, weekday)
        // Good Friday
        || (day_of_year == easter_monday - 3)
        // Memorial Day (last Monday in May)
        || UnitedStates::is_memorial_day(day, month, year, weekday)
        // Juneteenth (Monday if Sunday or Friday if Saturday)
        || UnitedStates::is_juneteenth(day, month, year, weekday, true)
        // Independence Day (Monday if Sunday or Friday if Saturday)
        || ((day == 4 || (day == 5 && weekday == Weekday::Mon) || (day == 3 && weekday == Weekday::Fri)) && month == 7)
        // Labor Day (first Monday in September)
        || UnitedStates::is_labor_day(day, month, year, weekday)
        // Thanksgiving Day (fourth Thursday in November)
        || ((day >= 22 && day <= 28) && weekday == Weekday::Thu && month == 11)
        // Christmas (Monday if Sunday or Friday if Saturday)
        || ((day == 25 || (day == 26 && weekday == Weekday::Mon) || (day == 24 && weekday == Weekday::Fri)) && month == 12)
        // Martin Luther King's Birthday (third Monday in January, from 1998)
        || (year >= 1998 && (day >= 15 && day <= 21) && weekday == Weekday::Mon && month == 1)
        // Presidential election days
        || ((year <= 1968 || (year <= 1980 && year % 4 == 0)) && month == 11 && day <= 7 && weekday == Weekday::Tue)
        // Special closings
        || (year == 2025 && month == 1 && day == 9)  // President Carter's Funeral
        || (year == 2018 && month == 12 && day == 5) // President Bush's Funeral
        || (year == 2012 && month == 10 && (day == 29 || day == 30)) // Hurricane Sandy
        || (year == 2007 && month == 1 && day == 2)  // President Ford's funeral
        || (year == 2004 && month == 6 && day == 11) // President Reagan's funeral
        || (year == 2001 && month == 9 && (11 <= day && day <= 14)) // September 11-14, 2001
        || (year == 1994 && month == 4 && day == 27) // President Nixon's funeral
        || (year == 1985 && month == 9 && day == 27) // Hurricane Gloria
        || (year == 1977 && month == 7 && day == 14) // 1977 Blackout
        || (year == 1973 && month == 1 && day == 25) // Funeral of former President Lyndon B. Johnson
        || (year == 1972 && month == 12 && day == 28) // Funeral of former President Harry S. Truman
        || (year == 1969 && month == 7 && day == 21) // National Day of Participation for the lunar exploration
        || (year == 1969 && month == 3 && day == 31) // Funeral of former President Eisenhower
        || (year == 1969 && month == 2 && day == 10) // Closed all day - heavy snow
        || (year == 1968 && month == 7 && day == 5) // Day after Independence Day
        || (year == 1968 && day_of_year >= 163 && weekday == Weekday::Wed) // Four day week (Paperwork Crisis)
        || (year == 1968 && month == 4 && day == 9) // Day of mourning for Martin Luther King Jr.
        || (year == 1963 && month == 11 && day == 25) // Funeral of President Kennedy
        || (year == 1961 && month == 5 && day == 29) // Day before Decoration Day
        || (year == 1958 && month == 12 && day == 26) // Day after Christmas
        // Christmas Eve
        || ((year == 1954 || year == 1956 || year == 1965) && month == 12 && day == 24)
        {
            false
        } else {
            true
        }
    }

    fn market_settlement_business_days(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        if
        // New Year's Day (possibly moved to Monday if on Sunday or to Friday if on Saturday)
        ((day == 1 || (day == 2 && weekday == Weekday::Mon)) && month == 1)
        || (day == 31 && weekday == Weekday::Fri && month == 12)
        // Martin Luther King's birthday (third Monday in January, observed since 1983)
        || ((day >= 15 && day <= 21) && weekday == Weekday::Mon && month == 1 && year >= 1983)
        // Washington's birthday (third Monday in February)
        || UnitedStates::is_washington_birthday(day, month, year, weekday)
        // Memorial Day (last Monday in May)
        || UnitedStates::is_memorial_day(day, month, year, weekday)
        // Juneteenth (Monday if Sunday or Friday if Saturday)
        || UnitedStates::is_juneteenth(day, month, year, weekday, true)
        // Independence Day (Monday if Sunday or Friday if Saturday)
        || ((day == 4 || (day == 5 && weekday == Weekday::Mon) || (day == 3 && weekday == Weekday::Fri)) && month == 7)
        // Labor Day (first Monday in September)
        || UnitedStates::is_labor_day(day, month, year, weekday)
        // Columbus Day (second Monday in October)
        || UnitedStates::is_columbus_day(day, month, year, weekday)
        // Veteran's Day (Monday if Sunday or Friday if Saturday)
        || UnitedStates::is_veterans_day(day, month, year, weekday)
        // Thanksgiving Day (fourth Thursday in November)
        || ((day >= 22 && day <= 28) && weekday == Weekday::Thu && month == 11)
        // Christmas (Monday if Sunday or Friday if Saturday)
        || ((day == 25 || (day == 26 && weekday == Weekday::Mon) || (day == 24 && weekday == Weekday::Fri)) && month == 12)
        {
            false
        } else {
            true
        }
    }

    fn market_government_bond_business_days(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();
        let day_of_year = date.day_of_year();
        let easter_monday = easter_monday(year);

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        if
        // New Year's Day (possibly moved to Monday if on Sunday)
        ((day == 1 || (day == 2 && weekday == Weekday::Mon)) && month == 1)
        // Martin Luther King's birthday (third Monday in January)
        || ((day >= 15 && day <= 21) && weekday == Weekday::Mon && month == 1 && year >= 1983)
        // Washington's birthday (third Monday in February)
        || UnitedStates::is_washington_birthday(day, month, year, weekday)
        // Good Friday. Specific conditions since 1996
        || (day_of_year == easter_monday - 3 && (year < 1996 || day > 7))
        // Memorial Day (last Monday in May)
        || UnitedStates::is_memorial_day(day, month, year, weekday)
        // Juneteenth (Monday if Sunday or Friday if Saturday)
        || UnitedStates::is_juneteenth(day, month, year, weekday, true)
        // Independence Day (Monday if Sunday or Friday if Saturday)
        || ((day == 4 || (day == 5 && weekday == Weekday::Mon) || (day == 3 && weekday == Weekday::Fri)) && month == 7)
        // Labor Day (first Monday in September)
        || UnitedStates::is_labor_day(day, month, year, weekday)
        // Columbus Day (second Monday in October)
        || UnitedStates::is_columbus_day(day, month, year, weekday)
        // Veteran's Day (Monday if Sunday)
        || UnitedStates::is_veterans_day_no_saturday(day, month, year, weekday)
        // Thanksgiving Day (fourth Thursday in November)
        || ((day >= 22 && day <= 28) && weekday == Weekday::Thu && month == 11)
        // Christmas (Monday if Sunday or Friday if Saturday)
        || ((day == 25 || (day == 26 && weekday == Weekday::Mon) || (day == 24 && weekday == Weekday::Fri)) && month == 12)
        {
            false
        } else if
        // President Bush's Funeral
        (year == 2018 && month == 12 && day == 5)
        // Hurricane Sandy
        || (year == 2012 && month == 10 && day == 30)
        // President Reagan's funeral
        || (year == 2004 && month == 6 && day == 11)
        {
            false
        } else {
            true
        }
    }

    fn market_sofr_business_days(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();
        let day_of_year = date.day_of_year();
        let easter_monday = easter_monday(year);

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        if
        // New Year's Day (possibly moved to Monday if on Sunday)
        ((day == 1 || (day == 2 && weekday == Weekday::Mon)) && month == 1)
        // Martin Luther King's birthday (third Monday in January)
        || ((day >= 15 && day <= 21) && weekday == Weekday::Mon && month == 1 && year >= 1983)
        // Washington's birthday (third Monday in February)
        || UnitedStates::is_washington_birthday(day, month, year, weekday)
        // Good Friday. Specific conditions since 1996
        || (day_of_year == easter_monday - 3 && (year < 1996 || day > 7))
        // Memorial Day (last Monday in May)
        || UnitedStates::is_memorial_day(day, month, year, weekday)
        // Juneteenth (Monday if Sunday or Friday if Saturday)
        || UnitedStates::is_juneteenth(day, month, year, weekday, true)
        // Independence Day (Monday if Sunday or Friday if Saturday)
        || ((day == 4 || (day == 5 && weekday == Weekday::Mon) || (day == 3 && weekday == Weekday::Fri)) && month == 7)
        // Labor Day (first Monday in September)
        || UnitedStates::is_labor_day(day, month, year, weekday)
        // Columbus Day (second Monday in October)
        || UnitedStates::is_columbus_day(day, month, year, weekday)
        // Veteran's Day (Monday if Sunday)
        || UnitedStates::is_veterans_day_no_saturday(day, month, year, weekday)
        // Thanksgiving Day (fourth Thursday in November)
        || ((day >= 22 && day <= 28) && weekday == Weekday::Thu && month == 11)
        // Christmas (Monday if Sunday or Friday if Saturday)
        || ((day == 25 || (day == 26 && weekday == Weekday::Mon) || (day == 24 && weekday == Weekday::Fri)) && month == 12)
        {
            false
        } else if
        // President Bush's Funeral
        (year == 2018 && month == 12 && day == 5)
        // Hurricane Sandy
        || (year == 2012 && month == 10 && day == 30)
        // President Reagan's funeral
        || (year == 2004 && month == 6 && day == 11)
        {
            false
        } else if
        // so far (that is, up to 2023 at the time of this change) SOFR never fixed
        // on Good Friday.  We're extrapolating that pattern.  This might change if
        // a fixing on Good Friday occurs in future years.
        day_of_year == easter_monday - 3 {
            false
        } else {
            true
        }
    }

    fn market_federal_reserve_business_days(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        if
        // New Year's Day (possibly moved to Monday if on Sunday)
        ((day == 1 || (day == 2 && weekday == Weekday::Mon)) && month == 1)
        // Martin Luther King's birthday (third Monday in January)
        || ((day >= 15 && day <= 21) && weekday == Weekday::Mon && month == 1 && year >= 1983)
        // Washington's birthday (third Monday in February)
        || UnitedStates::is_washington_birthday(day, month, year, weekday)
        // Memorial Day (last Monday in May)
        || UnitedStates::is_memorial_day(day, month, year, weekday)
        // Juneteenth (Monday if Sunday)
        || UnitedStates::is_juneteenth(day, month, year, weekday, false)
        // Independence Day (Monday if Sunday)
        || ((day == 4 || (day == 5 && weekday == Weekday::Mon)) && month == 7)
        // Labor Day (first Monday in September)
        || UnitedStates::is_labor_day(day, month, year, weekday)
        // Columbus Day (second Monday in October)
        || UnitedStates::is_columbus_day(day, month, year, weekday)
        // Veteran's Day (Monday if Sunday)
        || UnitedStates::is_veterans_day_no_saturday(day, month, year, weekday)
        // Thanksgiving Day (fourth Thursday in November)
        || ((day >= 22 && day <= 28) && weekday == Weekday::Thu && month == 11)
        // Christmas (Monday if Sunday)
        || ((day == 25 || (day == 26 && weekday == Weekday::Mon)) && month == 12)
        {
            false
        } else {
            true
        }
    }

    fn market_nerc_business_days(date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        if
        // New Year's Day (possibly moved to Monday if on Sunday)
        ((day == 1 || (day == 2 && weekday == Weekday::Mon)) && month == 1)
        // Memorial Day (last Monday in May)
        || UnitedStates::is_memorial_day(day, month, year, weekday)
        // Independence Day (Monday if Sunday)
        || ((day == 4 || (day == 5 && weekday == Weekday::Mon)) && month == 7)
        // Labor Day (first Monday in September)
        || UnitedStates::is_labor_day(day, month, year, weekday)
        // Thanksgiving Day (fourth Thursday in November)
        || ((day >= 22 && day <= 28) && weekday == Weekday::Thu && month == 11)
        // Christmas (Monday if Sunday)
        || ((day == 25 || (day == 26 && weekday == Weekday::Mon)) && month == 12)
        {
            false
        } else {
            true
        }
    }

    pub fn is_standard_business_day(&self, date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();

        if UnitedStates::is_weekend(weekday) {
            return false;
        }

        match self.market {
            UnitedStatesMarket::Nyse => UnitedStates::market_nyse_business_days(date),
            UnitedStatesMarket::Settlement => UnitedStates::market_settlement_business_days(date),
            UnitedStatesMarket::LiborImpact => {
                if ((day == 5 && weekday == Weekday::Mon) || (day == 3 && weekday == Weekday::Fri))
                    && month == 7
                    && year >= 2015
                {
                    return true;
                } else {
                    UnitedStates::market_settlement_business_days(date)
                }
            }
            UnitedStatesMarket::GovernmentBond => UnitedStates::market_government_bond_business_days(date), 
            UnitedStatesMarket::Sofr => UnitedStates::market_sofr_business_days(date),
            UnitedStatesMarket::Nerc => UnitedStates::market_nerc_business_days(date),
            UnitedStatesMarket::FederalReserve => UnitedStates::market_federal_reserve_business_days(date),
        }
    }
}

impl ImplCalendar for UnitedStates {
    fn impl_is_business_day(&self, date: &Date) -> bool {
        self.is_standard_business_day(date.base_date())
    }
    fn impl_name(&self) -> String {
        format!("UnitedStates({:?})", self.market)
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

impl IsCalendar for UnitedStates {}

impl Default for UnitedStates {
    fn default() -> Self {
        UnitedStates::new(UnitedStatesMarket::Sofr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_united_states_add_holiday() {
        let mut cal = UnitedStates::new(UnitedStatesMarket::Sofr);
        cal.add_holiday(Date::new(2028, 7, 22));
        assert_eq!(cal.is_business_day(&Date::new(2028, 7, 22)), false);
    }

    #[test]
    fn test_united_states_remove_holiday() {
        let mut cal = UnitedStates::new(UnitedStatesMarket::Sofr);
        cal.remove_holiday(Date::new(2025, 1, 1));
        assert_eq!(cal.is_business_day(&Date::new(2025, 1, 1)), true);
    }

    #[test]
    fn test_united_states_sofr() {
        let cal = UnitedStates::new(UnitedStatesMarket::Sofr);
        let expected_hol = vec![Date::new(2028, 7, 22)];
        for d in expected_hol {
            assert_eq!(cal.is_business_day(&d), false);
        }
    }
}
