use serde::{Deserialize, Serialize};

use crate::{currencies::enums::Currency, time::date::Date};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize, Hash)]
pub enum ExchangeGenerationMethod {
    SingleDate(SingleDate),
    DateWindow(DateWindow),
    Triangulation(Triangulation),
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize, Hash)]
pub struct SingleDate {
    date: Date,
}

impl SingleDate {
    pub fn new(date: Date) -> SingleDate {
        SingleDate { date }
    }

    pub fn date(&self) -> Date {
        self.date
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize, Hash)]
pub struct DateWindow {
    date_window: Vec<Date>,
}

impl DateWindow {
    pub fn new(date_window: Vec<Date>) -> DateWindow {
        DateWindow { date_window }
    }

    pub fn date_window(&self) -> &Vec<Date> {
        &self.date_window
    }

    pub fn number_of_dates(&self) -> usize {
        self.date_window.len()
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize, Hash)]
pub struct Triangulation {
    triangulation_curency: Currency, // The currency to triangulate
    first_date: Date, // The first date to triangulate
    second_date: Date, // The second date to triangulate 
}

impl Triangulation {
    pub fn new(triangulation_curency: Currency, first_date: Date, second_date: Date) -> Triangulation {
        Triangulation {
            triangulation_curency,
            first_date,
            second_date,
        }
    }

    pub fn triangulation_curency(&self) -> Currency {
        self.triangulation_curency
    }

    pub fn first_date(&self) -> Date {
        self.first_date
    }

    pub fn second_date(&self) -> Date {
        self.second_date
    }
}





