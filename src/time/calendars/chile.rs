use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, Weekday};

use crate::time::date::Date;
use super::traits::{easter_monday, ImplCalendar, IsCalendar};

/// # Chile 
/// A calendar for Chile
/// 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChileMarket {
    SSE, // Santiago Stock Exchange
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chile {
    market: ChileMarket,
    added_holidays: HashSet<Date>,
    removed_holidays: HashSet<Date>,
}

impl Chile {
    pub fn new(market: ChileMarket) -> Self {
        Chile {
            market,
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        }
    }

    fn is_weekend(day: Weekday) -> bool {
        day == Weekday::Sat || day == Weekday::Sun
    }

    fn is_new_years_day(day: u32, month: u32, year: i32) -> bool {
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        (day == 1 && month == 1)    
            || (day == 2 && month == 1 && w == Weekday::Mon && year >= 2016)
    }
    
    fn is_good_friday(day: u32, month: u32, year: i32) -> bool {
        let easter_friday = easter_monday(year) - 3;
        let dd = Date::new(year, month, day).day_of_year();
        dd == easter_friday
    }

    fn is_easter_saturday(day: u32, month: u32, year: i32) -> bool {
        let easter_saturday = easter_monday(year) - 2;
        let dd = Date::new(year, month, day).day_of_year();
        dd == easter_saturday
    }

    fn is_labour_day(day: u32, month: u32) -> bool {
        day == 1 && month == 5
    }

    fn is_navy_day(day: u32, month: u32) -> bool {
        day == 21 && month == 5
    }

    fn is_aboriginal_peoples_day(day: u32, month: u32, year: i32) -> bool {
        if year < 2021 || month != 6 {
            return false;
        }
        let aboriginal_peoples_days = [
            21, 21, 21, 20, 20, 21, 21, 20, 20,   // 2021-2029
            21, 21, 20, 20, 21, 21, 20, 20, 21, 21,   // 2030-2039
            20, 20, 21, 21, 20, 20, 21, 21, 20, 20,   // 2040-2049
            20, 21, 20, 20, 20, 21, 20, 20, 20, 21,   // 2050-2059
            20, 20, 20, 21, 20, 20, 20, 21, 20, 20,   // 2060-2069
            20, 21, 20, 20, 20, 21, 20, 20, 20, 20,   // 2070-2079
            20, 20, 20, 20, 20, 20, 20, 20, 20, 20,   // 2080-2089
            20, 20, 20, 20, 20, 20, 20, 20, 20, 20,   // 2090-2099
            21, 21, 21, 21, 21, 21, 21, 21, 20, 21,   // 2100-2109
            21, 21, 20, 21, 21, 21, 20, 21, 21, 21,   // 2110-2119
            20, 21, 21, 21, 20, 21, 21, 21, 20, 21,   // 2120-2129
            21, 21, 20, 21, 21, 21, 20, 20, 21, 21,   // 2130-2139
            20, 20, 21, 21, 20, 20, 21, 21, 20, 20,   // 2140-2149
            21, 21, 20, 20, 21, 21, 20, 20, 21, 21,   // 2150-2159
            20, 20, 21, 21, 20, 20, 21, 21, 20, 20,   // 2160-2169
            20, 21, 20, 20, 20, 21, 20, 20, 20, 21,   // 2170-2179
            20, 20, 20, 21, 20, 20, 20, 21, 20, 20,   // 2180-2189
            20, 21, 20, 20, 20, 21, 20, 20, 20, 20    // 2190-2199
        ];
        if year > 2199 {
            return false;
        }
        let index = (year - 2021) as usize;
        day == aboriginal_peoples_days[index]
    }

    fn is_saint_peter_and_saint_paul_day(day: u32, month: u32, year: i32) -> bool {
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        day >= 26 && day <= 29 && month == 6 && w == Weekday::Mon
            || day == 2 && month == 7 && w == Weekday::Mon 
    }

    fn is_our_lady_of_mount_carmel_day(day: u32, month: u32) -> bool {
        day == 16 && month == 7
    }

    fn is_assumption_day(day: u32, month: u32) -> bool {
        day == 15 && month == 8
    }

    fn is_independence_day(day: u32, month: u32, year: i32) -> bool {
         let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
         (day == 17 && month == 9 && ((w == Weekday::Mon && year >= 2007) || (w == Weekday::Fri && year >= 2016))) || (day == 16 && month == 9 && year == 2022)
            || (day == 18 && month == 9 )
            || (day == 16 && month == 9 && year == 2022)
    }

    fn is_army_day(day: u32, month: u32, year: i32) -> bool {
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        (day == 19 && month == 9)
            || (day == 20 && month == 9  && w == Weekday::Fri && year >= 2007)
    }

    fn is_discovery_of_two_worlds(day: u32, month: u32, year: i32) -> bool {
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        (day >= 9 && day <= 12 && month == 10 && w == Weekday::Mon)
            || (day == 15 && month == 10 && w == Weekday::Mon)
    }

    fn is_reformation_day(day: u32, month: u32, year: i32) -> bool {
        let w = NaiveDate::from_ymd_opt(year, month, day).unwrap().weekday();
        ((day == 27 && month == 10 && w == Weekday::Fri)
                 || (day == 31 && month == 10 && w != Weekday::Tue && w != Weekday::Wed)
                 || (day == 2 && month == 11 && w == Weekday::Fri)) 
                 && year >= 2008
    }

    fn is_all_saints_day(day: u32, month: u32) -> bool {
        day == 1 && month == 11
    }

    fn is_immaculate_conception(day: u32, month: u32) -> bool {
        day == 8 && month == 12
    }

    fn is_christmas_day(day: u32, month: u32) -> bool {
        day == 25 && month == 12
    }

    fn is_bank_holiday(day: u32, month: u32) -> bool {
        day == 31 && month == 12
    }

    pub fn is_business_day(&self, date: NaiveDate) -> bool {
        let weekday = date.weekday();
        let day = date.day();
        let month = date.month();
        let year = date.year();
        if Chile::is_weekend(weekday) {
            return false;
        }
        
        match self.market {
            ChileMarket::SSE => {
                    if Chile::is_new_years_day(day, month, year)
                    || Chile::is_good_friday(day, month, year)
                    || Chile::is_easter_saturday(day, month, year)
                    || Chile::is_labour_day(day, month)
                    || Chile::is_navy_day(day, month)
                    || Chile::is_aboriginal_peoples_day(day, month, year)
                    || Chile::is_saint_peter_and_saint_paul_day(day, month, year)
                    || Chile::is_our_lady_of_mount_carmel_day(day, month)
                    || Chile::is_assumption_day(day, month)
                    || Chile::is_independence_day(day, month, year)
                    || Chile::is_army_day(day, month, year)
                    || Chile::is_discovery_of_two_worlds(day, month, year)
                    || Chile::is_reformation_day(day, month, year)
                    || Chile::is_all_saints_day(day, month)
                    || Chile::is_immaculate_conception(day, month)
                    || Chile::is_christmas_day(day, month)
                    || Chile::is_bank_holiday(day, month)
                    {
                        return false;
                    }
                    true
            }
        }
    }
}

impl ImplCalendar for Chile {
    fn impl_is_business_day(&self, date: &Date) -> bool {
        self.is_business_day(date.base_date())
    }

    fn impl_name(&self) -> String {
        format!("Chile({:?})", self.market)
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
            if self.is_business_day(d.base_date()) {
                business_days.push(d);
            }
            d = d + 1;
        }
        business_days
    }
}


impl IsCalendar for Chile {}

impl Default for Chile {
    fn default() -> Self {
        Chile::new(ChileMarket::SSE)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::{date::Date, enums::{BusinessDayConvention, TimeUnit}, period::Period};

    #[test]
    fn test_chile_settlement() {
        let cal = Chile::new(ChileMarket::SSE);
        let expected_hol = vec![
            Date::new(2024, 1, 1),
            Date::new(2024, 3, 29),
            Date::new(2024, 5, 1),
            Date::new(2024, 5, 21),
            Date::new(2024, 6, 21),
            Date::new(2024, 7, 16),
            Date::new(2024, 8, 15),
            Date::new(2024, 9, 18),
            Date::new(2024, 9, 19),
            Date::new(2024, 10, 31),
            Date::new(2024, 11, 1),
            Date::new(2024, 12, 25),
            Date::new(2024, 12, 31),
        ];
        for d in expected_hol {
            assert_eq!(cal.is_business_day(d.base_date()), false);
        }
            
    }

    #[test]
    fn test_chile_settlement_extended() {
        let cal = Chile::new(ChileMarket::SSE);
        let expected_hol =         
        vec![
            Date::new(2010, 1, 1),
            Date::new(2010, 4, 2),
            Date::new(2010, 5, 21),
            Date::new(2010, 6, 28),
            Date::new(2010, 7, 16),
            Date::new(2010, 10, 11),
            Date::new(2010, 11, 1),
            Date::new(2010, 12, 8),
            Date::new(2011, 4, 22),
            Date::new(2011, 6, 27),
            Date::new(2011, 8, 15),
            Date::new(2011, 9, 19),
            Date::new(2011, 10, 10),
            Date::new(2011, 10, 31),
            Date::new(2011, 11, 1),
            Date::new(2011, 12, 8),
            Date::new(2012, 4, 6),
            Date::new(2012, 5, 1),
            Date::new(2012, 5, 21),
            Date::new(2012, 7, 2),
            Date::new(2012, 7, 16),
            Date::new(2012, 8, 15),
            Date::new(2012, 9, 17),
            Date::new(2012, 9, 18),
            Date::new(2012, 9, 19),
            Date::new(2012, 10, 15),
            Date::new(2012, 11, 1),
            Date::new(2012, 11, 2),
            Date::new(2012, 12, 25),
            Date::new(2013, 1, 1),
            Date::new(2013, 3, 29),
            Date::new(2013, 5, 1),
            Date::new(2013, 5, 21),
            Date::new(2013, 7, 16),
            Date::new(2013, 8, 15),
            Date::new(2013, 9, 18),
            Date::new(2013, 9, 19),
            Date::new(2013, 9, 20),
            Date::new(2013, 10, 31),
            Date::new(2013, 11, 1),
            Date::new(2013, 12, 25),
            Date::new(2014, 1, 1),
            Date::new(2014, 4, 18),
            Date::new(2014, 5, 1),
            Date::new(2014, 5, 21),
            Date::new(2014, 7, 16),
            Date::new(2014, 8, 15),
            Date::new(2014, 9, 18),
            Date::new(2014, 9, 19),
            Date::new(2014, 10, 31),
            Date::new(2014, 12, 8),
            Date::new(2014, 12, 25),
            Date::new(2015, 1, 1),
            Date::new(2015, 4, 3),
            Date::new(2015, 5, 1),
            Date::new(2015, 5, 21),
            Date::new(2015, 6, 29),
            Date::new(2015, 7, 16),
            Date::new(2015, 9, 18),
            Date::new(2015, 10, 12),
            Date::new(2015, 12, 8),
            Date::new(2015, 12, 25),
            Date::new(2016, 1, 1),
            Date::new(2016, 3, 25),
            Date::new(2016, 6, 27),
            Date::new(2016, 8, 15),
            Date::new(2016, 9, 19),
            Date::new(2016, 10, 10),
            Date::new(2016, 10, 31),
            Date::new(2016, 11, 1),
            Date::new(2016, 12, 8),
            Date::new(2017, 1, 2),
            Date::new(2017, 4, 14),
            Date::new(2017, 5, 1),
            Date::new(2017, 6, 26),
            Date::new(2017, 8, 15),
            Date::new(2017, 9, 18),
            Date::new(2017, 9, 19),
            Date::new(2017, 10, 9),
            Date::new(2017, 10, 27),
            Date::new(2017, 11, 1),
            Date::new(2017, 12, 8),
            Date::new(2017, 12, 25),
            Date::new(2018, 1, 1),
            Date::new(2018, 3, 30),
            Date::new(2018, 5, 1),
            Date::new(2018, 5, 21),
            Date::new(2018, 7, 2),
            Date::new(2018, 7, 16),
            Date::new(2018, 8, 15),
            Date::new(2018, 9, 17),
            Date::new(2018, 9, 18),
            Date::new(2018, 9, 19),
            Date::new(2018, 10, 15),
            Date::new(2018, 11, 1),
            Date::new(2018, 11, 2),
            Date::new(2018, 12, 25),
            Date::new(2019, 1, 1),
            Date::new(2019, 4, 19),
            Date::new(2019, 5, 1),
            Date::new(2019, 5, 21),
            Date::new(2019, 7, 16),
            Date::new(2019, 8, 15),
            Date::new(2019, 9, 18),
            Date::new(2019, 9, 19),
            Date::new(2019, 9, 20),
            Date::new(2019, 10, 31),
            Date::new(2019, 11, 1),
            Date::new(2019, 12, 25),
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 10),
            Date::new(2020, 5, 1),
            Date::new(2020, 5, 21),
            Date::new(2020, 6, 29),
            Date::new(2020, 7, 16),
            Date::new(2020, 9, 18),
            Date::new(2020, 10, 12),
            Date::new(2020, 12, 8),
            Date::new(2020, 12, 25),
            Date::new(2021, 1, 1),
            Date::new(2021, 4, 2),
            Date::new(2021, 5, 21),
            Date::new(2021, 6, 21),
            Date::new(2021, 6, 28),
            Date::new(2021, 7, 16),
            Date::new(2021, 9, 17),
            Date::new(2021, 10, 11),
            Date::new(2021, 11, 1),
            Date::new(2021, 12, 8),
            Date::new(2022, 4, 15),
            Date::new(2022, 6, 21),
            Date::new(2022, 6, 27),
            Date::new(2022, 8, 15),
            Date::new(2022, 9, 16),
            Date::new(2022, 9, 19),
            Date::new(2022, 10, 10),
            Date::new(2022, 10, 31),
            Date::new(2022, 11, 1),
            Date::new(2022, 12, 8),
            Date::new(2023, 1, 2),
            Date::new(2023, 4, 7),
            Date::new(2023, 5, 1),
            Date::new(2023, 6, 21),
            Date::new(2023, 6, 26),
            Date::new(2023, 8, 15),
            Date::new(2023, 9, 18),
            Date::new(2023, 9, 19),
            Date::new(2023, 10, 9),
            Date::new(2023, 10, 27),
            Date::new(2023, 11, 1),
            Date::new(2023, 12, 8),
            Date::new(2023, 12, 25),
            Date::new(2024, 1, 1),
            Date::new(2024, 3, 29),
            Date::new(2024, 5, 1),
            Date::new(2024, 5, 21),
            Date::new(2024, 6, 20),
            Date::new(2024, 7, 16),
            Date::new(2024, 8, 15),
            Date::new(2024, 9, 18),
            Date::new(2024, 9, 19),
            Date::new(2024, 9, 20),
            Date::new(2024, 10, 31),
            Date::new(2024, 11, 1),
            Date::new(2024, 12, 25),
            Date::new(2025, 1, 1),
            Date::new(2025, 4, 18),
            Date::new(2025, 5, 1),
            Date::new(2025, 5, 21),
            Date::new(2025, 6, 20),
            Date::new(2025, 7, 16),
            Date::new(2025, 8, 15),
            Date::new(2025, 9, 18),
            Date::new(2025, 9, 19),
            Date::new(2025, 10, 31),
            Date::new(2025, 12, 8),
            Date::new(2025, 12, 25),
            Date::new(2026, 1, 1),
            Date::new(2026, 4, 3),
            Date::new(2026, 5, 1),
            Date::new(2026, 5, 21),
            Date::new(2026, 6, 29),
            Date::new(2026, 7, 16),
            Date::new(2026, 9, 18),
            Date::new(2026, 10, 12),
            Date::new(2026, 12, 8),
            Date::new(2026, 12, 25),
            Date::new(2027, 1, 1),
            Date::new(2027, 3, 26),
            Date::new(2027, 5, 21),
            Date::new(2027, 6, 21),
            Date::new(2027, 6, 28),
            Date::new(2027, 7, 16),
            Date::new(2027, 9, 17),
            Date::new(2027, 10, 11),
            Date::new(2027, 11, 1),
            Date::new(2027, 12, 8),
            Date::new(2028, 4, 14),
            Date::new(2028, 5, 1),
            Date::new(2028, 6, 20),
            Date::new(2028, 6, 26),
            Date::new(2028, 8, 15),
            Date::new(2028, 9, 18),
            Date::new(2028, 9, 19),
            Date::new(2028, 10, 9),
            Date::new(2028, 10, 27),
            Date::new(2028, 11, 1),
            Date::new(2028, 12, 8),
            Date::new(2028, 12, 25),
            Date::new(2029, 1, 1),
            Date::new(2029, 3, 30),
            Date::new(2029, 5, 1),
            Date::new(2029, 5, 21),
            Date::new(2029, 6, 20),
            Date::new(2029, 7, 2),
            Date::new(2029, 7, 16),
            Date::new(2029, 8, 15),
            Date::new(2029, 9, 17),
            Date::new(2029, 9, 18),
            Date::new(2029, 9, 19),
            Date::new(2029, 10, 15),
            Date::new(2029, 11, 1),
            Date::new(2029, 11, 2),
            Date::new(2029, 12, 25),
            Date::new(2030, 1, 1),
            Date::new(2030, 4, 19),
            Date::new(2030, 5, 1),
            Date::new(2030, 5, 21),
            Date::new(2030, 6, 21),
            Date::new(2030, 7, 16),
            Date::new(2030, 8, 15),
            Date::new(2030, 9, 18),
            Date::new(2030, 9, 19),
            Date::new(2030, 9, 20),
            Date::new(2030, 10, 31),
            Date::new(2030, 11, 1),
            Date::new(2030, 12, 25),
        ];
        for d in expected_hol {
            println!("{:?}", d);
            assert_eq!(cal.is_business_day(d.base_date()), false);
        }
            
    }

    #[test]
    fn test_ajusted_date(){
        let cal = Chile::new(ChileMarket::SSE);
        let date = Date::new(2040, 1, 28);

        let new_date = cal.adjust(date, Some(BusinessDayConvention::ModifiedFollowing));

        assert!(new_date == Date::new(2040, 1, 30));

        let date = Date::new(2040, 1, 29);
        let new_date = cal.adjust(date, Some(BusinessDayConvention::ModifiedFollowing));
        assert!(new_date == Date::new(2040, 1, 30));
    }

    #[test]
    fn test_advace_date (){
        let cal = Chile::new(ChileMarket::SSE);
        let date = Date::new(2040, 1, 28);
        let new_date = cal.advance(date, Period::new(2,  TimeUnit::Days), Some(BusinessDayConvention::ModifiedFollowing), false);
        
        assert!(new_date == Date::new(2040, 1, 31));
    }

}


