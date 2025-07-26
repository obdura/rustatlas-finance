use crate::time::date::Date;
use std::collections::HashSet;

use super::traits::{ImplCalendar, IsCalendar};

/// # WeekendsOnly
/// A calendar that considers only weekends as business days.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeekendsOnly {
    added_holidays: HashSet<Date>,
    removed_holidays: HashSet<Date>,
}

impl WeekendsOnly {
    pub fn new() -> Self {
        WeekendsOnly {
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        }
    }
}

impl ImplCalendar for WeekendsOnly {
    fn impl_is_business_day(&self, date: &Date) -> bool {
        !self.is_weekend(&date.weekday())
    }

    fn impl_name(&self) -> String {
        "WeekendsOnly".to_string()
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

impl IsCalendar for WeekendsOnly {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::time::{
        calendars::{traits::{ImplCalendar, IsCalendar}, weekendsonly::WeekendsOnly},
        date::Date,
    };

    #[test]
    fn test_weekends_only_add_holiday() {
        let mut cal = WeekendsOnly::new();
        assert_eq!(cal.is_business_day(&Date::new(2025, 7, 25)), true);
        cal.add_holiday(Date::new(2025, 7, 25));
        assert_eq!(cal.is_business_day(&Date::new(2025, 7, 25)), false);
    }

    #[test]
    fn test_weekends_only_remove_holiday() {
        let mut cal = WeekendsOnly::new();
        assert_eq!(cal.is_business_day(&Date::new(2025, 7, 26)), false);
        cal.add_holiday(Date::new(2025, 7,26));
        assert_eq!(cal.is_business_day(&Date::new(2025, 7, 26)), false);
    }

    #[test]
    fn test_weekendsonly() {
        let cal = WeekendsOnly {
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        };
        assert_eq!(cal.name(), "WeekendsOnly");
        assert_eq!(cal.is_business_day(&Date::new(2023, 8, 23)), true);
        assert_eq!(cal.is_business_day(&Date::new(2023, 8, 25)), true);
    }
}
