use serde::Serialize;

use super::calendars::{
    brazil::Brazil,
    chile::Chile,
    colombia::Colombia,
    mexico::Mexico,
    nullcalendar::NullCalendar,
    target::TARGET,
    traits::{ImplCalendar, IsCalendar},
    unitedstates::UnitedStates,
    weekendsonly::WeekendsOnly,
};
use crate::{
    time::date::Date,
    utils::errors::{AtlasError, Result},
};
use std::collections::HashSet;

/// # Calendar
/// A calendar.
///
/// ## Enums
/// * `NullCalendar` - A calendar that considers all days as business days.
/// * `WeekendsOnly` - A calendar that considers only weekends as business days.
/// * `TARGET` - A calendar that considers only TARGET business days as business days.
/// * `UnitedStates` - A calendar for the United States.
/// * `Brazil` - A calendar for Brazil.
/// * `Chile` - A calendar for Chile.
/// * `Composite` - A composite calendar that combines multiple calendars.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Calendar {
    NullCalendar(NullCalendar),
    WeekendsOnly(WeekendsOnly),
    TARGET(TARGET),
    UnitedStates(UnitedStates),
    Brazil(Brazil),
    Chile(Chile),
    Mexico(Mexico),
    Colombia(Colombia),
    Composite(Box<Calendar>, Box<Calendar>),
}

impl Serialize for Calendar {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Calendar::NullCalendar(cal) => cal.impl_name(),
            Calendar::WeekendsOnly(cal) => cal.impl_name(),
            Calendar::TARGET(cal) => cal.impl_name(),
            Calendar::UnitedStates(cal) => cal.impl_name(),
            Calendar::Brazil(cal) => cal.impl_name(),
            Calendar::Chile(cal) => cal.impl_name(),
            Calendar::Mexico(cal) => cal.impl_name(),
            Calendar::Colombia(cal) => cal.impl_name(),
            Calendar::Composite(cal1, cal2) => {
                format!("Composite({}, {})", cal1.impl_name(), cal2.impl_name())
            }
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for Calendar {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Calendar, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "NullCalendar" => Ok(Calendar::NullCalendar(NullCalendar::new())),
            "WeekendsOnly" => Ok(Calendar::WeekendsOnly(WeekendsOnly::new())),
            "TARGET" => Ok(Calendar::TARGET(TARGET::new())),
            "UnitedStates" => Ok(Calendar::UnitedStates(UnitedStates::default())),
            "Brazil" => Ok(Calendar::Brazil(Brazil::default())),
            "Chile" => Ok(Calendar::Chile(Chile::default())),
            "Mexico" => Ok(Calendar::Mexico(Mexico::default())),
            "Colombia" => Ok(Calendar::Colombia(Colombia::default())),
            _ => Err(serde::de::Error::custom(format!("Invalid calendar: {}", s))),
        }
    }
}

impl TryFrom<String> for Calendar {
    type Error = AtlasError;

    fn try_from(s: String) -> Result<Self> {
        match s.as_str() {
            "NullCalendar" => Ok(Calendar::NullCalendar(NullCalendar::new())),
            "WeekendsOnly" => Ok(Calendar::WeekendsOnly(WeekendsOnly::new())),
            "TARGET" => Ok(Calendar::TARGET(TARGET::new())),
            "UnitedStates" => Ok(Calendar::UnitedStates(UnitedStates::default())),
            "Brazil" => Ok(Calendar::Brazil(Brazil::default())),
            "Chile" => Ok(Calendar::Chile(Chile::default())),
            "Mexico" => Ok(Calendar::Mexico(Mexico::default())),
            "Colombia" => Ok(Calendar::Colombia(Colombia::default())),
            _ => Err(AtlasError::InvalidValueErr(format!(
                "Invalid calendar: {}",
                s
            ))),
        }
    }
}

impl From<Calendar> for String {
    fn from(calendar: Calendar) -> Self {
        match calendar {
            Calendar::NullCalendar(_) => "NullCalendar".to_string(),
            Calendar::WeekendsOnly(_) => "WeekendsOnly".to_string(),
            Calendar::TARGET(_) => "TARGET".to_string(),
            Calendar::UnitedStates(_) => "UnitedStates".to_string(),
            Calendar::Brazil(_) => "Brazil".to_string(),
            Calendar::Chile(_) => "Chile".to_string(),
            Calendar::Mexico(_) => "Mexico".to_string(),
            Calendar::Colombia(_) => "Colombia".to_string(),
            Calendar::Composite(cal1, cal2) => {
                format!("Composite({}, {})", cal1.impl_name(), cal2.impl_name())
            }
        }
    }
}

impl ImplCalendar for Calendar {
    fn impl_name(&self) -> String {
        match self {
            Calendar::NullCalendar(cal) => cal.impl_name(),
            Calendar::WeekendsOnly(cal) => cal.impl_name(),
            Calendar::TARGET(cal) => cal.impl_name(),
            Calendar::UnitedStates(cal) => cal.impl_name(),
            Calendar::Brazil(cal) => cal.impl_name(),
            Calendar::Chile(cal) => cal.impl_name(),
            Calendar::Mexico(cal) => cal.impl_name(),
            Calendar::Colombia(cal) => cal.impl_name(),
            Calendar::Composite(cal1, cal2) => {
                format!("Composite({}, {})", cal1.impl_name(), cal2.impl_name())
            }
        }
    }

    fn impl_is_business_day(&self, date: &Date) -> bool {
        match self {
            Calendar::NullCalendar(cal) => cal.impl_is_business_day(date),
            Calendar::WeekendsOnly(cal) => cal.impl_is_business_day(date),
            Calendar::TARGET(cal) => cal.impl_is_business_day(date),
            Calendar::UnitedStates(cal) => cal.impl_is_business_day(date),
            Calendar::Brazil(cal) => cal.impl_is_business_day(date),
            Calendar::Chile(cal) => cal.impl_is_business_day(date),
            Calendar::Mexico(cal) => cal.impl_is_business_day(date),
            Calendar::Colombia(cal) => cal.impl_is_business_day(date),
            Calendar::Composite(cal1, cal2) => {
                cal1.impl_is_business_day(date) && cal2.impl_is_business_day(date)
            }
        }
    }

    fn added_holidays(&self) -> HashSet<Date> {
        match self {
            Calendar::NullCalendar(cal) => cal.added_holidays(),
            Calendar::WeekendsOnly(cal) => cal.added_holidays(),
            Calendar::TARGET(cal) => cal.added_holidays(),
            Calendar::UnitedStates(cal) => cal.added_holidays(),
            Calendar::Brazil(cal) => cal.added_holidays(),
            Calendar::Chile(cal) => cal.added_holidays(),
            Calendar::Mexico(cal) => cal.added_holidays(),
            Calendar::Colombia(cal) => cal.added_holidays(),
            Calendar::Composite(cal1, cal2) => {
                let mut holidays = cal1.added_holidays();
                holidays.extend(cal2.added_holidays());
                holidays
            }
        }
    }

    fn removed_holidays(&self) -> HashSet<Date> {
        match self {
            Calendar::NullCalendar(cal) => cal.removed_holidays(),
            Calendar::WeekendsOnly(cal) => cal.removed_holidays(),
            Calendar::TARGET(cal) => cal.removed_holidays(),
            Calendar::UnitedStates(cal) => cal.removed_holidays(),
            Calendar::Brazil(cal) => cal.removed_holidays(),
            Calendar::Chile(cal) => cal.removed_holidays(),
            Calendar::Colombia(cal) => cal.removed_holidays(),
            Calendar::Mexico(cal) => cal.removed_holidays(),
            Calendar::Composite(cal1, cal2) => {
                let mut holidays = cal1.removed_holidays();
                holidays.extend(cal2.removed_holidays());
                holidays
            }
        }
    }

    fn add_holiday(&mut self, date: Date) {
        match self {
            Calendar::NullCalendar(cal) => cal.add_holiday(date),
            Calendar::WeekendsOnly(cal) => cal.add_holiday(date),
            Calendar::TARGET(cal) => cal.add_holiday(date),
            Calendar::UnitedStates(cal) => cal.add_holiday(date),
            Calendar::Brazil(cal) => cal.add_holiday(date),
            Calendar::Chile(cal) => cal.add_holiday(date),
            Calendar::Mexico(cal) => cal.add_holiday(date),
            Calendar::Colombia(cal) => cal.add_holiday(date),
            Calendar::Composite(cal1, cal2) => {
                cal1.add_holiday(date);
                cal2.add_holiday(date);
            }
        }
    }

    fn remove_holiday(&mut self, date: Date) {
        match self {
            Calendar::NullCalendar(cal) => cal.remove_holiday(date),
            Calendar::WeekendsOnly(cal) => cal.remove_holiday(date),
            Calendar::TARGET(cal) => cal.remove_holiday(date),
            Calendar::UnitedStates(cal) => cal.remove_holiday(date),
            Calendar::Brazil(cal) => cal.remove_holiday(date),
            Calendar::Chile(cal) => cal.remove_holiday(date),
            Calendar::Mexico(cal) => cal.remove_holiday(date),
            Calendar::Colombia(cal) => cal.remove_holiday(date),
            Calendar::Composite(cal1, cal2) => {
                cal1.remove_holiday(date);
                cal2.remove_holiday(date);
            }
        }
    }

    fn holiday_list(&self, from: Date, to: Date, include_weekends: bool) -> Vec<Date> {
        match self {
            Calendar::NullCalendar(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::WeekendsOnly(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::TARGET(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::UnitedStates(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::Brazil(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::Chile(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::Mexico(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::Colombia(cal) => cal.holiday_list(from, to, include_weekends),
            Calendar::Composite(cal1, cal2) => {
                let mut holidays = cal1.holiday_list(from, to, include_weekends);
                holidays.extend(cal2.holiday_list(from, to, include_weekends));
                holidays
            }
        }
    }

    fn business_day_list(&self, from: Date, to: Date) -> Vec<Date> {
        match self {
            Calendar::NullCalendar(cal) => cal.business_day_list(from, to),
            Calendar::WeekendsOnly(cal) => cal.business_day_list(from, to),
            Calendar::TARGET(cal) => cal.business_day_list(from, to),
            Calendar::UnitedStates(cal) => cal.business_day_list(from, to),
            Calendar::Brazil(cal) => cal.business_day_list(from, to),
            Calendar::Chile(cal) => cal.business_day_list(from, to),
            Calendar::Mexico(cal) => cal.business_day_list(from, to),
            Calendar::Colombia(cal) => cal.business_day_list(from, to),
            Calendar::Composite(cal1, cal2) => {
                let mut business_days = cal1.business_day_list(from, to);
                business_days.extend(cal2.business_day_list(from, to));
                business_days
            }
        }
    }
}

impl IsCalendar for Calendar {}

#[cfg(test)]
mod tests {
    use crate::time::calendars::traits::IsCalendar;
    use crate::time::date::Date;
    use crate::time::enums::{BusinessDayConvention, TimeUnit};
    use crate::time::period::Period;
    use crate::time::{
        calendar::Calendar,
        calendars::{
            brazil::Brazil, chile::Chile, colombia::Colombia, mexico::Mexico,
            nullcalendar::NullCalendar, target::TARGET, traits::ImplCalendar,
            unitedstates::UnitedStates, weekendsonly::WeekendsOnly,
        },
    };
    use serde_json;

    #[test]
    fn test_create_calendar() {
        let calendar = Calendar::NullCalendar(NullCalendar::new());
        assert_eq!(calendar.impl_name(), "NullCalendar");
        let calendar = Calendar::WeekendsOnly(WeekendsOnly::new());
        assert_eq!(calendar.impl_name(), "WeekendsOnly");
        let calendar = Calendar::TARGET(TARGET::new());
        assert_eq!(calendar.impl_name(), "TARGET");
        let calendar = Calendar::UnitedStates(UnitedStates::default());
        assert_eq!(calendar.impl_name(), "UnitedStates(Sofr)");
        let calendar = Calendar::Brazil(Brazil::default());
        assert_eq!(calendar.impl_name(), "Brazil(Settlement)");
        let calendar = Calendar::Chile(Chile::default());
        assert_eq!(calendar.impl_name(), "Chile(SSE)");
        let calendar = Calendar::Mexico(Mexico::default());
        assert_eq!(calendar.impl_name(), "Mexico(BMV)");
        let calendar = Calendar::Colombia(Colombia::default());
        assert_eq!(calendar.impl_name(), "Colombia(BVC)");
        let calendar = Calendar::Composite(
            Box::new(Calendar::TARGET(TARGET::new())),
            Box::new(Calendar::UnitedStates(UnitedStates::default())),
        );
        assert_eq!(
            calendar.impl_name(),
            "Composite(TARGET, UnitedStates(Sofr))"
        );
    }

    #[test]
    fn test_calendar_try_from_string() {
        let calendar = Calendar::try_from("TARGET".to_string()).unwrap();
        assert_eq!(calendar.impl_name(), "TARGET");
        let invalid = Calendar::try_from("InvalidCalendar".to_string());
        assert!(invalid.is_err());
    }

    #[test]
    fn test_calendar_from_enum_to_string() {
        let calendar = Calendar::Chile(Chile::default());
        let name: String = calendar.clone().into();
        assert_eq!(name, "Chile");
    }

    #[test]
    fn test_add_and_remove_holiday() {
        let mut calendar = Calendar::UnitedStates(UnitedStates::default());
        let date = Date::new(2024, 7, 4);
        calendar.add_holiday(date);
        assert!(calendar.added_holidays().contains(&date));
        calendar.remove_holiday(date);
        assert!(calendar.removed_holidays().contains(&date));
    }

    #[test]
    fn test_business_day_and_holiday_list() {
        let calendar = Calendar::WeekendsOnly(WeekendsOnly::new());
        let from = Date::new(2024, 6, 1);
        let to = Date::new(2024, 6, 10);
        let holidays = calendar.holiday_list(from, to, true);
        let business_days = calendar.business_day_list(from, to);
        assert!(!holidays.is_empty());
        assert!(!business_days.is_empty());
    }

    #[test]
    fn test_impl_is_business_day() {
        let calendar = Calendar::NullCalendar(NullCalendar::new());
        let date = Date::new(2024, 1, 1);
        assert!(calendar.impl_is_business_day(&date));
    }

    #[test]
    fn test_calendar_composite() {
        let cal1 = Calendar::Chile(Chile::default());
        let cal2 = Calendar::UnitedStates(UnitedStates::default());
        let composite = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        assert_eq!(
            composite.impl_name(),
            "Composite(Chile(SSE), UnitedStates(Sofr))"
        );

        let date = Date::new(2024, 7, 4);
        assert!(!composite.is_business_day(&date));

        let date = Date::new(2024, 9, 18);
        assert!(!composite.is_business_day(&date));
    }

    #[test]
    fn test_calendar_composite_with_three_calendars() {
        let cal1 = Calendar::Chile(Chile::default());
        let cal2 = Calendar::UnitedStates(UnitedStates::default());
        let composite1 = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        let cal3 = Calendar::Brazil(Brazil::default());
        let composite2 = Calendar::Composite(Box::new(composite1), Box::new(cal3));
        assert_eq!(
            composite2.impl_name(),
            "Composite(Composite(Chile(SSE), UnitedStates(Sofr)), Brazil(Settlement))"
        );

        let date = Date::new(2024, 7, 4);
        assert!(!composite2.is_business_day(&date));

        let date = Date::new(2024, 9, 18);
        assert!(!composite2.is_business_day(&date));

        let date = Date::new(2024, 10, 12);
        assert!(!composite2.is_business_day(&date));
    }

    #[test]
    fn test_calendar_serialize_deserialize() {
        let calendar = Calendar::Brazil(Brazil::default());
        let serialized = serde_json::to_string(&calendar).unwrap();
        assert_eq!(serialized, "\"Brazil(Settlement)\"");
        let deserialized: Calendar = serde_json::from_str("\"Brazil\"").unwrap();
        assert_eq!(deserialized.impl_name(), "Brazil(Settlement)");
        let invalid: Result<Calendar, _> = serde_json::from_str("\"Invalid\"");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_calendar_added_and_removed_holidays_composite() {
        let mut cal1 = Calendar::Chile(Chile::default());
        let mut cal2 = Calendar::UnitedStates(UnitedStates::default());
        let date1 = Date::new(2024, 9, 18);
        let date2 = Date::new(2024, 7, 4);
        cal1.add_holiday(date1);
        cal2.add_holiday(date2);
        let composite = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        let holidays = composite.added_holidays();
        assert!(holidays.contains(&date1));
        assert!(holidays.contains(&date2));
    }

    #[test]
    fn test_calendar_remove_holiday_composite() {
        let mut cal1 = Calendar::Chile(Chile::default());
        let mut cal2 = Calendar::UnitedStates(UnitedStates::default());
        let date1 = Date::new(2024, 9, 18);
        let date2 = Date::new(2024, 7, 4);
        cal1.remove_holiday(date1);
        cal2.remove_holiday(date2);
        let composite = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        let removed = composite.removed_holidays();
        assert!(removed.contains(&date1));
        assert!(removed.contains(&date2));
    }

    #[test]
    fn test_calendar_holiday_and_business_day_list_composite() {
        let cal1 = Calendar::WeekendsOnly(WeekendsOnly::new());
        let cal2 = Calendar::NullCalendar(NullCalendar::new());
        let composite = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        let from = Date::new(2024, 6, 1);
        let to = Date::new(2024, 6, 7);
        let holidays = composite.holiday_list(from, to, true);
        let business_days = composite.business_day_list(from, to);
        assert!(!holidays.is_empty());
        assert!(!business_days.is_empty());
    }

    #[test]
    fn test_calendar_impl_name_for_nested_composite() {
        let cal1 = Calendar::Brazil(Brazil::default());
        let cal2 = Calendar::Chile(Chile::default());
        let composite = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        let cal3 = Calendar::TARGET(TARGET::new());
        let nested = Calendar::Composite(Box::new(composite), Box::new(cal3));
        assert_eq!(
            nested.impl_name(),
            "Composite(Composite(Brazil(Settlement), Chile(SSE)), TARGET)"
        );
    }

    #[test]
    fn test_calendar_is_business_day_for_weekends_only() {
        let calendar = Calendar::WeekendsOnly(WeekendsOnly::new());
        let weekday = Date::new(2024, 6, 3); // Monday
        let weekend = Date::new(2024, 6, 2); // Sunday
        assert!(calendar.is_business_day(&weekday));
        assert!(!calendar.is_business_day(&weekend));
    }

    #[test]
    fn test_pub_date_icap() {
        let cal1 = Calendar::Chile(Chile::default());
        let cal2 = Calendar::UnitedStates(UnitedStates::default());
        let composite = Calendar::Composite(Box::new(cal1), Box::new(cal2));
        let today = Date::new(2025, 8, 19);
        let _today_0_bd = composite.advance(today, Period::new(0, TimeUnit::Days), None, false);
        let today_1_bd = composite.advance(today, Period::new(1, TimeUnit::Days), None, false);
        let today_2_bd = composite.advance(today, Period::new(2, TimeUnit::Days), None, false);
        let tenors = vec![
            "1W", "2W", "1M", "2M", "3M", "4M", "5M", "6M", "7M", "8M", "9M", "10M", "11M", "12M",
            "18M", "24M",
        ];

        for tenor in tenors {
            let t = Period::from_str(tenor).unwrap();
            let settlement_date = match t.units() {
                TimeUnit::Weeks => composite.adjust(
                    today_1_bd + t,
                    Some(BusinessDayConvention::ModifiedFollowing),
                ),
                TimeUnit::Months => composite.adjust(
                    today_2_bd + t,
                    Some(BusinessDayConvention::ModifiedFollowing),
                ),
                _ => todo!(),
            };
            let pub_date = composite.advance(
                settlement_date,
                Period::new(-1, TimeUnit::Days),
                None,
                false,
            );

            println!("{:?} - {:?}", settlement_date, pub_date);
        }
    }
}
