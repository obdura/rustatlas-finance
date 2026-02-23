use crate::{
    cashflows::{side::Side, simplecashflow::SimpleCashflow},
    currencies::enums::Currency,
    instruments::forwards::fxforward::FxForward,
    time::{
        calendar::Calendar,
        calendars::{nullcalendar::NullCalendar, traits::IsCalendar},
        date::Date,
        enums::BusinessDayConvention,
        period::Period,
    },
    utils::errors::{AtlasError, Result},
};

/// MakeVanillaFxForward
/// Builder for a vanilla FX Forward contract
/// Currencies are exchanged at maturity without fx fixings.
/// The pay date can be set for both legs or a tenor can be set to calculate the pay dates from the negotiation date.
pub struct MakeVanillaFxForward {
    id: Option<String>,
    negotiation_date: Option<Date>,
    tenor: Option<Period>,
    pay_date_receive_side: Option<Date>,
    pay_date_pay_side: Option<Date>,
    pay_currency: Option<Currency>,
    receive_currency: Option<Currency>,
    pay_nominal: Option<f64>,
    receive_nominal: Option<f64>,
    calendar: Option<Calendar>,
    pay_discount_curve_id: Option<usize>,
    receive_discount_curve_id: Option<usize>,
}

impl MakeVanillaFxForward {
    pub fn new() -> MakeVanillaFxForward {
        MakeVanillaFxForward {
            id: None,
            negotiation_date: None,
            tenor: None,
            pay_date_receive_side: None,
            pay_date_pay_side: None,
            pay_currency: None,
            receive_currency: None,
            pay_nominal: None,
            receive_nominal: None,
            calendar: None,
            pay_discount_curve_id: None,
            receive_discount_curve_id: None,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_negotiation_date(mut self, negotiation_date: Date) -> Self {
        self.negotiation_date = Some(negotiation_date);
        self
    }

    pub fn with_tenor(mut self, tenor: Period) -> Self {
        self.tenor = Some(tenor);
        self
    }

    pub fn with_pay_date_receive_side(mut self, pay_date_receive_side: Date) -> Self {
        self.pay_date_receive_side = Some(pay_date_receive_side);
        self
    }

    pub fn with_pay_date_pay_side(mut self, pay_date_pay_side: Date) -> Self {
        self.pay_date_pay_side = Some(pay_date_pay_side);
        self
    }

    pub fn with_pay_currency(mut self, pay_currency: Currency) -> Self {
        self.pay_currency = Some(pay_currency);
        self
    }

    pub fn with_receive_currency(mut self, receive_currency: Currency) -> Self {
        self.receive_currency = Some(receive_currency);
        self
    }

    pub fn with_pay_nominal(mut self, pay_nominal: f64) -> Self {
        self.pay_nominal = Some(pay_nominal);
        self
    }

    pub fn with_receive_nominal(mut self, receive_nominal: f64) -> Self {
        self.receive_nominal = Some(receive_nominal);
        self
    }

    pub fn with_calendar(mut self, calendar: Calendar) -> Self {
        self.calendar = Some(calendar);
        self
    }

    pub fn with_pay_discount_curve_id(mut self, pay_discount_curve_id: usize) -> Self {
        self.pay_discount_curve_id = Some(pay_discount_curve_id);
        self
    }

    pub fn with_receive_discount_curve_id(mut self, receive_discount_curve_id: usize) -> Self {
        self.receive_discount_curve_id = Some(receive_discount_curve_id);
        self
    }

    pub fn build(self) -> Result<FxForward> {
        // Default calendar to NullCalendar if not set
        let calendar = self
            .calendar
            .unwrap_or(Calendar::NullCalendar(NullCalendar::new()));

        let pay_date_receive_side = match self.pay_date_receive_side {
            Some(date) => date,
            None => match self.pay_date_pay_side {
                Some(date) => date,
                None => {
                    let tenor = self.tenor.ok_or(AtlasError::ValueNotSetErr(
                        "Pay date in pay side could not be set".into(),
                    ))?;
                    let negotiation_date = self.negotiation_date.ok_or(
                        AtlasError::ValueNotSetErr("Pay date in pay side could not be set".into()),
                    )?;
                    // calendar.advance(
                    //     negotiation_date,
                    //     tenor,
                    //     Some(BusinessDayConvention::Following),
                    //     false,
                    // )
                    let negotiation_date = negotiation_date.add_period(tenor);
                    calendar.adjust(negotiation_date, Some(BusinessDayConvention::Following))
                }
            },
        };

        let pay_date_pay_side = match self.pay_date_pay_side {
            Some(date) => date,
            None => match self.pay_date_receive_side {
                Some(date) => date,
                None => {
                    let tenor = self.tenor.ok_or(AtlasError::ValueNotSetErr(
                        "Pay date in pay side could not be set".into(),
                    ))?;
                    let negotiation_date = self.negotiation_date.ok_or(
                        AtlasError::ValueNotSetErr("Pay date in pay side could not be set".into()),
                    )?;
                    // calendar.advance(
                    //     negotiation_date,
                    //     tenor,
                    //     Some(BusinessDayConvention::Following),
                    //     false,
                    // )
                    let negotiation_date = negotiation_date.add_period(tenor);
                    calendar.adjust(negotiation_date, Some(BusinessDayConvention::Following))
                }
            },
        };

        let pay_currency = self.pay_currency.ok_or(AtlasError::ValueNotSetErr(
            "Pay currency could not be set".into(),
        ))?;
        let receive_currency = self.receive_currency.ok_or(AtlasError::ValueNotSetErr(
            "Receive currency could not be set".into(),
        ))?;

        if pay_currency == receive_currency {
            return Err(AtlasError::ValueNotSetErr(
                "Pay and receive currency cannot be the same".into(),
            ));
        }

        let pay_nominal = self.pay_nominal.ok_or(AtlasError::ValueNotSetErr(
            "Pay nominal could not be set".into(),
        ))?;
        let receive_nominal = self.receive_nominal.ok_or(AtlasError::ValueNotSetErr(
            "Receive nominal could not be set".into(),
        ))?;

        let pay_cashflow = SimpleCashflow::new(pay_date_receive_side, pay_currency, Side::Pay)
            .with_amount(pay_nominal);
        let receive_cashflow =
            SimpleCashflow::new(pay_date_pay_side, receive_currency, Side::Receive)
                .with_amount(receive_nominal);

        let mut fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?;

        match self.id {
            Some(id) => fx_forward.set_id(id),
            None => (),
        };

        match self.pay_discount_curve_id {
            Some(id) => fx_forward.set_pay_discount_curve_id(id),
            None => (),
        };

        match self.receive_discount_curve_id {
            Some(id) => fx_forward.set_receive_discount_curve_id(id),
            None => (),
        };

        Ok(fx_forward)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        currencies::enums::Currency,
        time::{
            calendar::Calendar, calendars::nullcalendar::NullCalendar, date::Date, period::Period,
        },
    };

    fn sample_date() -> Date {
        Date::new(2024, 6, 1)
    }

    fn sample_period() -> Period {
        Period::new(1, crate::time::enums::TimeUnit::Months)
    }

    #[test]
    fn test_build_with_all_fields_set() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_id("FXFWD1".to_string())
            .with_negotiation_date(sample_date())
            .with_tenor(sample_period())
            .with_pay_date_receive_side(Date::new(2024, 7, 1))
            .with_pay_date_pay_side(Date::new(2024, 7, 1))
            .with_pay_currency(Currency::USD)
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .with_receive_nominal(900.0)
            .with_calendar(Calendar::NullCalendar(NullCalendar::new()))
            .with_pay_discount_curve_id(1)
            .with_receive_discount_curve_id(2)
            .build();

        assert!(fx_forward.is_ok());
    }

    #[test]
    fn test_build_with_missing_pay_date_receive_side_uses_tenor() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_negotiation_date(sample_date())
            .with_tenor(sample_period())
            .with_pay_currency(Currency::USD)
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .with_receive_nominal(900.0)
            .build();

        assert!(fx_forward.is_ok());
    }

    #[test]
    fn test_build_missing_pay_currency_should_fail() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_negotiation_date(sample_date())
            .with_tenor(sample_period())
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .with_receive_nominal(900.0)
            .build();

        assert!(fx_forward.is_err());
    }

    #[test]
    fn test_build_missing_receive_nominal_should_fail() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_negotiation_date(sample_date())
            .with_tenor(sample_period())
            .with_pay_currency(Currency::USD)
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .build();

        assert!(fx_forward.is_err());
    }

    #[test]
    fn test_build_with_only_pay_date_pay_side_set() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_negotiation_date(sample_date())
            .with_tenor(sample_period())
            .with_pay_date_pay_side(Date::new(2024, 7, 1))
            .with_pay_currency(Currency::USD)
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .with_receive_nominal(900.0)
            .build();

        assert!(fx_forward.is_ok());
    }

    #[test]
    fn test_build_with_only_pay_date_receive_side_set() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_negotiation_date(sample_date())
            .with_tenor(sample_period())
            .with_pay_date_receive_side(Date::new(2024, 7, 1))
            .with_pay_currency(Currency::USD)
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .with_receive_nominal(900.0)
            .build();

        assert!(fx_forward.is_ok());
    }

    #[test]
    fn test_build_missing_negotiation_date_should_fail() {
        let fx_forward = MakeVanillaFxForward::new()
            .with_tenor(sample_period())
            .with_pay_currency(Currency::USD)
            .with_receive_currency(Currency::EUR)
            .with_pay_nominal(1000.0)
            .with_receive_nominal(900.0)
            .build();

        assert!(fx_forward.is_err());
    }
}
