use crate::{
    cashflows::{
        cashflow::{Cashflow, CashflowType},
        fixedratecoupon::FixedRateCoupon,
        side::Side,
    },
    currencies::enums::Currency,
    instruments::{swaps::leg::Leg, traits::RateType, traits::Structure},
    rates::interestrate::{InterestRate, RateDefinition},
    time::{
        calendar::Calendar,
        calendars::{nullcalendar::NullCalendar, traits::IsCalendar},
        date::Date,
        enums::{BusinessDayConvention, DateGenerationRule, Frequency},
        period::Period,
        schedule::MakeSchedule,
    },
    utils::errors::{AtlasError, Result},
};

use super::traits::{add_cashflows_to_vec, notionals_vector};

/// # MakeFixedRateLeg
/// MakeFixedRateLeg is a builder for fixed rate leg. Uses the builder pattern.
// TODO: Handle negative amounts (redemptions, notionals and disbursements)
#[derive(Debug, Clone)]
pub struct MakeFixedRateLeg {
    negotiation_date: Option<Date>,

    settlement_period: Option<Period>,
    start_date: Option<Date>,

    tenor: Option<Period>,
    end_date: Option<Date>,

    payment_lag: Option<Period>,

    payment_frequency: Option<Frequency>,
    side: Option<Side>,
    currency: Option<Currency>,
    notional: Option<f64>,

    structure: Option<Structure>,

    rate_definition: Option<RateDefinition>,
    rate_value: Option<f64>,
    rate: Option<InterestRate>,

    discount_curve_id: Option<usize>,

    calendar: Option<Calendar>,
    business_day_convention: Option<BusinessDayConvention>,
    date_generation_rule: Option<DateGenerationRule>,

    initial_flow: bool,
    final_flow: bool,
}

/// New, setters and getters
impl MakeFixedRateLeg {
    pub fn new() -> MakeFixedRateLeg {
        MakeFixedRateLeg {
            negotiation_date: None,
            start_date: None,
            tenor: None,
            end_date: None,
            settlement_period: None,
            payment_lag: None,
            payment_frequency: None,
            side: None,
            currency: None,
            notional: None,
            structure: None,
            rate_definition: None,
            rate_value: None,
            rate: None,
            discount_curve_id: None,
            calendar: None,
            business_day_convention: None,
            date_generation_rule: None,
            initial_flow: false,
            final_flow: true,
        }
    }

    /// Sets the currency.
    pub fn with_currency(mut self, currency: Currency) -> MakeFixedRateLeg {
        self.currency = Some(currency);
        self
    }

    /// Sets the side.
    pub fn with_side(mut self, side: Side) -> MakeFixedRateLeg {
        self.side = Some(side);
        self
    }

    /// Sets the notional.
    pub fn with_notional(mut self, notional: f64) -> MakeFixedRateLeg {
        self.notional = Some(notional);
        self
    }

    /// Sets the notional.
    pub fn with_calendar(mut self, calendar: Option<Calendar>) -> MakeFixedRateLeg {
        self.calendar = calendar;
        self
    }

    /// Sets the notional.
    pub fn with_business_day_convention(
        mut self,
        business_day_convention: Option<BusinessDayConvention>,
    ) -> MakeFixedRateLeg {
        self.business_day_convention = business_day_convention;
        self
    }

    /// Sets the notional.
    pub fn with_date_generation_rule(
        mut self,
        date_generation_rule: Option<DateGenerationRule>,
    ) -> MakeFixedRateLeg {
        self.date_generation_rule = date_generation_rule;
        self
    }

    /// Sets the rate definition.
    pub fn with_rate_definition(mut self, rate_definition: RateDefinition) -> MakeFixedRateLeg {
        self.rate_definition = Some(rate_definition);
        match self.rate_value {
            Some(rate_value) => {
                self.rate = Some(InterestRate::new(
                    rate_value,
                    rate_definition.compounding(),
                    rate_definition.frequency(),
                    rate_definition.day_counter(),
                ));
            }
            None => match self.rate {
                Some(rate) => {
                    self.rate = Some(InterestRate::new(
                        rate.rate(),
                        rate_definition.compounding(),
                        rate_definition.frequency(),
                        rate_definition.day_counter(),
                    ));
                }
                None => (),
            },
        }
        self
    }

    /// Sets the rate.
    pub fn with_rate(mut self, rate: InterestRate) -> MakeFixedRateLeg {
        self.rate = Some(rate);
        self
    }

    /// Sets the rate value.
    pub fn with_rate_value(mut self, rate_value: f64) -> MakeFixedRateLeg {
        self.rate_value = Some(rate_value);
        match self.rate {
            Some(rate) => {
                self.rate = Some(InterestRate::new(
                    rate_value,
                    rate.compounding(),
                    rate.frequency(),
                    rate.day_counter(),
                ));
            }
            None => match self.rate_definition {
                Some(rate_definition) => {
                    self.rate = Some(InterestRate::new(
                        rate_value,
                        rate_definition.compounding(),
                        rate_definition.frequency(),
                        rate_definition.day_counter(),
                    ));
                }
                None => (),
            },
        }
        self
    }

    /// Sets the negotiation date.
    pub fn with_negotiation_date(mut self, date: Date) -> MakeFixedRateLeg {
        self.negotiation_date = Some(date);
        self
    }

    /// Sets the settlement period.
    pub fn with_settlement_period(mut self, settlement_period: Period) -> MakeFixedRateLeg {
        self.settlement_period = Some(settlement_period);
        self
    }

    /// Sets the start date.
    pub fn with_start_date(mut self, start_date: Date) -> MakeFixedRateLeg {
        self.start_date = Some(start_date);
        self
    }

    /// Sets the payment lag.
    pub fn with_payment_lag(mut self, payment_lag: Period) -> MakeFixedRateLeg {
        self.payment_lag = Some(payment_lag);
        self
    }

    /// Sets the end date.
    pub fn with_end_date(mut self, end_date: Date) -> MakeFixedRateLeg {
        self.end_date = Some(end_date);
        self
    }

    /// Sets the tenor.
    pub fn with_tenor(mut self, tenor: Period) -> MakeFixedRateLeg {
        self.tenor = Some(tenor);
        self
    }

    /// Sets the discount curve id.
    pub fn with_discount_curve_id(mut self, id: Option<usize>) -> MakeFixedRateLeg {
        self.discount_curve_id = id;
        self
    }

    /// Sets the payment frequency.
    pub fn with_payment_frequency(mut self, frequency: Frequency) -> MakeFixedRateLeg {
        self.payment_frequency = Some(frequency);
        self
    }

    /// Sets the structure.
    pub fn with_structure(mut self, structure: Structure) -> MakeFixedRateLeg {
        self.structure = Some(structure);
        self
    }

    /// Sets the structure to bullet.
    pub fn bullet(mut self) -> MakeFixedRateLeg {
        self.structure = Some(Structure::Bullet);
        self
    }

    /// Sets the initial flow.
    pub fn with_initial_flow(mut self, initial_flow: bool) -> MakeFixedRateLeg {
        self.initial_flow = initial_flow;
        self
    }

    /// Sets the final flow.
    pub fn with_final_flow(mut self, final_flow: bool) -> MakeFixedRateLeg {
        self.final_flow = final_flow;
        self
    }
}

impl MakeFixedRateLeg {
    pub fn build(self) -> Result<Leg> {
        let mut cashflows = Vec::new();

        let notional = self
            .notional
            .ok_or(AtlasError::ValueNotSetErr("Notional".into()))?;

        let rate = self.rate.ok_or(AtlasError::ValueNotSetErr("Rate".into()))?;

        let side = self.side.ok_or(AtlasError::ValueNotSetErr("Side".into()))?;

        let currency = self
            .currency
            .ok_or(AtlasError::ValueNotSetErr("Currency".into()))?;

        let calendar = self
            .calendar
            .unwrap_or(Calendar::NullCalendar(NullCalendar::new()));

        let payment_frequency = self
            .payment_frequency
            .ok_or(AtlasError::ValueNotSetErr("Payment frequency".into()))?;

        let structure = self
            .structure
            .ok_or(AtlasError::ValueNotSetErr("Structure".into()))?;

        let adjusted_start_date = match self.start_date {
            Some(date) => date,
            None => match self.negotiation_date {
                Some(date) => match self.settlement_period {
                    Some(settlement_period) => calendar.advance(
                        date,
                        settlement_period,
                        Some(BusinessDayConvention::Following),
                        false,
                    ),
                    None => date,
                },
                None => Err(AtlasError::ValueNotSetErr("Start date".into()))?,
            },
        };

        let adjusted_end_date = match self.end_date {
            Some(date) => date,
            None => {
                let tenor = self
                    .tenor
                    .ok_or(AtlasError::ValueNotSetErr("end date".into()))?;
                adjusted_start_date + tenor
            }
        };

        let business_day_convention = self
            .business_day_convention
            .unwrap_or(BusinessDayConvention::Unadjusted);
        let date_generation_rule = self
            .date_generation_rule
            .unwrap_or(DateGenerationRule::Backward);

        match structure {
            Structure::Bullet => {
                // make schedule
                let mut schedule_builder =
                    MakeSchedule::new(adjusted_start_date, adjusted_end_date)
                        .with_frequency(payment_frequency)
                        .with_calendar(calendar.clone())
                        .with_convention(business_day_convention)
                        .with_termination_date_convention(business_day_convention)
                        .with_rule(date_generation_rule);

                let fixing_schedule = schedule_builder.build()?;

                let fixings_dates = fixing_schedule.dates();

                let payment_dates = match self.payment_lag {
                    Some(lag) => fixings_dates
                        .iter()
                        .map(|d| {
                            calendar.advance(*d, lag, Some(BusinessDayConvention::Following), false)
                        })
                        .collect(),
                    None => fixings_dates.clone(),
                };

                let first_date: Vec<Date> = vec![*payment_dates.first().unwrap()];
                let last_date: Vec<Date> = vec![*payment_dates.last().unwrap()];

                let notionals =
                    notionals_vector(fixings_dates.len() - 1, notional, Structure::Bullet);

                if self.initial_flow {
                    add_cashflows_to_vec(
                        &mut cashflows,
                        &first_date,
                        &vec![notional],
                        side.inverse(),
                        currency,
                        CashflowType::Disbursement,
                    );
                }

                if self.final_flow {
                    add_cashflows_to_vec(
                        &mut cashflows,                      &last_date,
                        &vec![notional],
                        side,
                        currency,
                        CashflowType::Redemption,
                    );
                }

                build_coupons_from_notionals(
                    &mut cashflows,
                    &payment_dates,
                    &fixings_dates,
                    &notionals,
                    rate,
                    side,
                    currency,
                )?;

                match self.discount_curve_id {
                    Some(id) => cashflows
                        .iter_mut()
                        .for_each(|cf| cf.set_discount_curve_id(id)),
                    None => (),
                }

                let leg = Leg::new(
                    self.negotiation_date,
                    adjusted_start_date,
                    adjusted_end_date,
                    notional,
                    payment_frequency,
                    structure,
                    RateType::Fixed,
                    rate.rate(),
                    rate.rate_definition(),
                    currency,
                    side,
                    self.discount_curve_id,
                    None,
                    cashflows,
                );

                Ok(leg)
            }
            _ => todo!(),
        }
    }
}

fn build_coupons_from_notionals(
    cashflows: &mut Vec<Cashflow>,
    payment_dates: &Vec<Date>,
    fixing_dates: &Vec<Date>,
    notionals: &Vec<f64>,
    rate: InterestRate,
    side: Side,
    currency: Currency,
) -> Result<()> {
    if fixing_dates.len() - 1 != notionals.len() {
        Err(AtlasError::InvalidValueErr(
            "Dates and notionals must have the same length".to_string(),
        ))?;
    }
    if fixing_dates.len() < 2 {
        Err(AtlasError::InvalidValueErr(
            "Dates must have at least two elements".to_string(),
        ))?;
    }
    for ((fixing_date_pair, payment_date_pair), notional) in fixing_dates
        .windows(2)
        .zip(payment_dates.windows(2))
        .zip(notionals)
    {
        let d1 = fixing_date_pair[0];
        let d2 = fixing_date_pair[1];
        let payment_date = payment_date_pair[1];
        let coupon = FixedRateCoupon::new(*notional, rate, d1, d2, payment_date, currency, side);
        cashflows.push(Cashflow::FixedRateCoupon(coupon));
    }
    Ok(())
}

#[cfg(test)]
mod tests{

    use super::MakeFixedRateLeg;
    use crate::{
        cashflows::{side::Side, traits::Payable},
        currencies::enums::Currency,
        rates::{enums::Compounding, interestrate::{InterestRate, RateDefinition}},
        time::{
            calendar::Calendar,
            calendars::chile::Chile,
            date::Date,
            daycounter::DayCounter,
            enums::{BusinessDayConvention, DateGenerationRule, Frequency, TimeUnit},
            period::Period,
        },
        visitors::traits::HasCashflows,
    };

    #[test]
    fn test_make_leg_bullet() {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let notional = 1_000_000.0;
        let instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(notional)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()
            .unwrap();

        assert_eq!(instrument.cashflows_as_vec().len(), 3);
    }

    #[test]
    fn test_make_schedule() {
        let start_date = Date::new(2025, 1, 27);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let cal = Calendar::Chile(Chile::default());

        let notional = 1_000_000.0;
        let instrument = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_settlement_period(Period::new(2, TimeUnit::Days))
            .with_tenor(Period::new(5, TimeUnit::Years))
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(notional)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .with_calendar(Some(cal))
            .with_business_day_convention(Some(BusinessDayConvention::ModifiedFollowing))
            .with_date_generation_rule(Some(DateGenerationRule::Backward))
            .bullet()
            .build()
            .unwrap();

        let cashflows = instrument.cashflows_as_vec();
 
        let dates = vec![
            Date::new(2030, 1, 28),
            Date::new(2025, 7, 28),
            Date::new(2026, 1, 27),
            Date::new(2026, 7, 27),
            Date::new(2027, 1, 27),
            Date::new(2027, 7, 27),
            Date::new(2028, 1, 27),
            Date::new(2028, 7, 27),
            Date::new(2029, 1, 29),
            Date::new(2029, 7, 27),
            Date::new(2030, 1, 28),
        ];

        for cf in cashflows {
            assert!(dates.contains(&cf.payment_date()));
        }

    }

    #[test]
    fn test_make_fixed_rate_leg_with_initial_flow() {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(0.5, rate_definition);
        let notional = 1_000_000.0;

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Annual)
            .bullet()
            .build()
            .unwrap();

        assert!(fix_leg.cashflows_as_vec().len() == 2);

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Annual)
            .with_initial_flow(true)
            .bullet()
            .build()
            .unwrap();

        assert!(fix_leg.cashflows_as_vec().len() == 3);
    }

    #[test]
    fn test_make_fixed_rate_leg_with_final_flow() {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate = InterestRate::from_rate_definition(0.5, rate_definition);
        let notional = 1_000_000.0;

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Annual)
            .bullet()
            .with_final_flow(false)
            .build()
            .unwrap();

        assert!(fix_leg.cashflows_as_vec().len() == 1);

        let fix_leg = MakeFixedRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_rate(rate)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Annual)
            .with_final_flow(false)
            .with_initial_flow(true)
            .bullet()
            .build()
            .unwrap();

        assert!(fix_leg.cashflows_as_vec().len() == 2);
    }   
}
