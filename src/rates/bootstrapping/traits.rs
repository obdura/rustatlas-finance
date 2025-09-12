use crate::{time::date::Date, visitors::traits::HasCashflows};

/// Represents a relationship between a rate helper and its maturity date or last relevant date.
pub enum Pillar {
    MaturityDate,
    LastRelevantDate,
}

/// Trait for bootstrapping rate helpers.
pub trait IsRateHelper: HasCashflows {
   fn maturity_date(&self) -> Date;
   fn last_relevant_date(&self) -> Date;
}


