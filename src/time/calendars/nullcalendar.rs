use std::collections::HashSet;

use crate::time::date::Date;

use super::traits::{ImplCalendar, IsCalendar};

/// # NullCalendar
/// A calendar that considers all days as business days.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NullCalendar {
    added_holidays: HashSet<Date>,
    removed_holidays: HashSet<Date>,
}

impl NullCalendar {
    pub fn new() -> Self {
        NullCalendar {
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        }
    }
}

impl ImplCalendar for NullCalendar {
    fn impl_name(&self) -> String {
        "NullCalendar".to_string()
    }

    fn impl_is_business_day(&self, _date: &Date) -> bool {
        true
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

impl IsCalendar for NullCalendar {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::calendars::nullcalendar::NullCalendar;
    use crate::time::calendars::traits::IsCalendar;
    use crate::time::date::Date;


    #[test]
    fn test_nullcalendar_add_holiday() {
        let mut cal = NullCalendar::new();
        assert_eq!(cal.is_business_day(&Date::new(2021, 1, 1)), true);
        cal.add_holiday(Date::new(2021, 1, 1));
        assert_eq!(cal.is_business_day(&Date::new(2021, 1, 1)), false);
    }


    #[test]
    fn test_nullcalendar() {
        let cal = NullCalendar {
            added_holidays: HashSet::new(),
            removed_holidays: HashSet::new(),
        };
        assert_eq!(cal.name(), "NullCalendar");
        assert_eq!(cal.is_business_day(&Date::new(2021, 1, 1)), true);
        assert_eq!(cal.is_business_day(&Date::new(2021, 1, 2)), true);
    }

    #[test]
    fn test_nullcalendar_remove_holiday() {
        let mut cal = NullCalendar::new();
        cal.add_holiday(Date::new(2021, 1, 1));
        assert_eq!(cal.is_business_day(&Date::new(2021, 1, 1)), false);
        cal.remove_holiday(Date::new(2021, 1, 1));
        // Removing a holiday should not affect NullCalendar's business day logic
        assert_eq!(cal.is_business_day(&Date::new(2021, 1, 1)), false);
    }

    #[test]
    fn test_nullcalendar_holiday_list() {
        let mut cal = NullCalendar::new();
        cal.add_holiday(Date::new(2021, 1, 1));
        cal.add_holiday(Date::new(2021, 1, 3));
        let holidays = cal.holiday_list(Date::new(2021, 1, 1), Date::new(2021, 1, 5), true);
        assert!(holidays.contains(&Date::new(2021, 1, 1)));
        assert!(holidays.contains(&Date::new(2021, 1, 3)));
        assert!(!holidays.contains(&Date::new(2021, 1, 2)));
    }

    #[test]
    fn test_nullcalendar_business_day_list() {
        let cal = NullCalendar::new();
        let business_days = cal.business_day_list(Date::new(2021, 1, 1), Date::new(2021, 1, 3));
        assert_eq!(business_days.len(), 3);
        assert_eq!(business_days[0], Date::new(2021, 1, 1));
        assert_eq!(business_days[1], Date::new(2021, 1, 2));
        assert_eq!(business_days[2], Date::new(2021, 1, 3));
    }

}
