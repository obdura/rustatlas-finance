use crate::{
    cashflows::{
        cashflow::{Cashflow, CashflowType},
        floatingratecoupon::FloatingRateCoupon,
        side::Side,
    },
    currencies::enums::Currency,
    instruments::{swaps::leg::Leg, traits::RateType, traits::Structure},
    rates::interestrate::RateDefinition,
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

/// # MakeFloatingRateLeg
/// Builder for a floating rate loan.
#[derive(Debug, Clone)]
pub struct MakeFloatingRateLeg {
    negotiation_date: Option<Date>,

    settlement_period: Option<Period>,
    start_date: Option<Date>,

    tenor: Option<Period>,
    end_date: Option<Date>,

    payment_lag: Option<Period>,

    payment_frequency: Option<Frequency>,
    side: Option<Side>,
    currency: Option<Currency>,
    pay_currency: Option<Currency>,
    notional: Option<f64>,

    structure: Option<Structure>,

    rate_definition: Option<RateDefinition>,
    spread: Option<f64>,

    discount_curve_id: Option<usize>,
    forecast_curve_id: Option<usize>,

    calendar: Option<Calendar>,
    business_day_convention: Option<BusinessDayConvention>, // for accrual dates
    termination_business_day_convention: Option<BusinessDayConvention>, // for accrual dates

    business_day_convention_fixing_dates: Option<BusinessDayConvention>, // for fixing dates
    termination_business_day_convention_fixing_dates: Option<BusinessDayConvention>, // for fixing dates

    date_generation_rule: Option<DateGenerationRule>,

    initial_flow: bool,
    final_flow: bool,
}

/// Constructor, setters and getters.
impl MakeFloatingRateLeg {
    pub fn new() -> MakeFloatingRateLeg {
        MakeFloatingRateLeg {
            negotiation_date: None,
            settlement_period: None,
            start_date: None,
            tenor: None,
            end_date: None,
            payment_lag: None,
            payment_frequency: None,
            side: None,
            currency: None,
            pay_currency: None,
            notional: None,
            structure: None,
            rate_definition: None,
            spread: None,
            discount_curve_id: None,
            forecast_curve_id: None,
            calendar: None,
            business_day_convention: None,
            termination_business_day_convention: None,
            business_day_convention_fixing_dates: None,
            termination_business_day_convention_fixing_dates: None,
            date_generation_rule: None,
            initial_flow: false,
            final_flow: true,
        }
    }

    pub fn with_calendar(mut self, calendar: Option<Calendar>) -> MakeFloatingRateLeg {
        self.calendar = calendar;
        self
    }

    pub fn with_business_day_convention(
        mut self,
        business_day_convention: Option<BusinessDayConvention>,
    ) -> MakeFloatingRateLeg {
        self.business_day_convention = business_day_convention;
        self
    }

    pub fn with_termination_business_day_convention(
        mut self,
        termination_business_day_convention: Option<BusinessDayConvention>,
    ) -> MakeFloatingRateLeg {
        self.termination_business_day_convention = termination_business_day_convention;
        self
    }

    pub fn with_business_day_convention_fixing_dates(
        mut self,
        business_day_convention_fixing_dates: Option<BusinessDayConvention>,
    ) -> MakeFloatingRateLeg {
        self.business_day_convention_fixing_dates = business_day_convention_fixing_dates;
        self
    }

    pub fn with_termination_business_day_convention_fixing_dates(
        mut self,
        termination_business_day_convention_fixing_dates: Option<BusinessDayConvention>,
    ) -> MakeFloatingRateLeg {
        self.termination_business_day_convention_fixing_dates =
            termination_business_day_convention_fixing_dates;
        self
    }

    pub fn with_date_generation_rule(
        mut self,
        date_generation_rule: Option<DateGenerationRule>,
    ) -> MakeFloatingRateLeg {
        self.date_generation_rule = date_generation_rule;
        self
    }

    pub fn with_negotiation_date(mut self, negociation_date: Date) -> MakeFloatingRateLeg {
        self.negotiation_date = Some(negociation_date);
        self
    }

    pub fn with_settlement_period(mut self, settlement_period: Period) -> MakeFloatingRateLeg {
        self.settlement_period = Some(settlement_period);
        self
    }

    pub fn with_start_date(mut self, start_date: Date) -> MakeFloatingRateLeg {
        self.start_date = Some(start_date);
        self
    }

    pub fn with_end_date(mut self, end_date: Date) -> MakeFloatingRateLeg {
        self.end_date = Some(end_date);
        self
    }

    pub fn with_payment_lag(mut self, payment_lag: Period) -> MakeFloatingRateLeg {
        self.payment_lag = Some(payment_lag);
        self
    }

    pub fn with_tenor(mut self, tenor: Period) -> MakeFloatingRateLeg {
        self.tenor = Some(tenor);
        return self;
    }

    pub fn with_forecast_curve_id(
        mut self,
        forecast_curve_id: Option<usize>,
    ) -> MakeFloatingRateLeg {
        self.forecast_curve_id = forecast_curve_id;
        return self;
    }

    pub fn with_discount_curve_id(
        mut self,
        discount_curve_id: Option<usize>,
    ) -> MakeFloatingRateLeg {
        self.discount_curve_id = discount_curve_id;
        return self;
    }

    pub fn with_rate_definition(mut self, rate_definition: RateDefinition) -> MakeFloatingRateLeg {
        self.rate_definition = Some(rate_definition);
        return self;
    }

    pub fn with_notional(mut self, notional: f64) -> MakeFloatingRateLeg {
        self.notional = Some(notional);
        return self;
    }

    pub fn with_currency(mut self, currency: Currency) -> MakeFloatingRateLeg {
        self.currency = Some(currency);
        return self;
    }

    pub fn with_pay_currency(mut self, pay_currency: Currency) -> MakeFloatingRateLeg {
        self.pay_currency = Some(pay_currency);
        return self;
    }

    pub fn with_spread(mut self, spread: f64) -> MakeFloatingRateLeg {
        self.spread = Some(spread);
        return self;
    }

    pub fn bullet(mut self) -> MakeFloatingRateLeg {
        self.structure = Some(Structure::Bullet);
        return self;
    }

    pub fn equal_redemptions(mut self) -> MakeFloatingRateLeg {
        self.structure = Some(Structure::EqualRedemptions);
        self
    }

    pub fn zero(mut self) -> MakeFloatingRateLeg {
        self.structure = Some(Structure::Zero);
        self.payment_frequency = Some(Frequency::Once);
        self
    }

    pub fn other(mut self) -> MakeFloatingRateLeg {
        self.structure = Some(Structure::Other);
        self.payment_frequency = Some(Frequency::OtherFrequency);
        self
    }

    pub fn with_side(mut self, side: Side) -> MakeFloatingRateLeg {
        self.side = Some(side);
        return self;
    }

    pub fn with_payment_frequency(mut self, frequency: Frequency) -> MakeFloatingRateLeg {
        self.payment_frequency = Some(frequency);
        return self;
    }

    pub fn with_structure(mut self, structure: Structure) -> MakeFloatingRateLeg {
        self.structure = Some(structure);
        return self;
    }

    /// Sets the initial flow.
    pub fn with_initial_flow(mut self, initial_flow: bool) -> MakeFloatingRateLeg {
        self.initial_flow = initial_flow;
        self
    }

    /// Sets the final flow.
    pub fn with_final_flow(mut self, final_flow: bool) -> MakeFloatingRateLeg {
        self.final_flow = final_flow;
        self
    }
}

/// Build
impl MakeFloatingRateLeg {
    pub fn build(self) -> Result<Leg> {
        let mut cashflows = Vec::new();

        let notional = self
            .notional
            .ok_or(AtlasError::ValueNotSetErr("Notional".into()))?;

        let structure = self
            .structure
            .ok_or(AtlasError::ValueNotSetErr("Structure".into()))?;

        let rate_definition = self
            .rate_definition
            .ok_or(AtlasError::ValueNotSetErr("Rate definition".into()))?;

        // Default spread to 0.0 if not set
        let spread = self.spread.unwrap_or(0.0);

        let payment_frequency = if structure == Structure::Zero {
            Frequency::Once
        } else {
            self.payment_frequency
                .ok_or(AtlasError::ValueNotSetErr("Payment frequency".into()))?
        };

        let side = self.side.ok_or(AtlasError::ValueNotSetErr("Side".into()))?;

        let currency = self
            .currency
            .ok_or(AtlasError::ValueNotSetErr("Currency".into()))?;

        let pay_currency = self
            .pay_currency
            .unwrap_or(currency);

        // Default calendar to NullCalendar if not set
        let calendar = self
            .calendar
            .unwrap_or(Calendar::NullCalendar(NullCalendar::new()));

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

        // Default business day convention to Unadjusted if not set
        let business_day_convention = self
            .business_day_convention
            .unwrap_or(BusinessDayConvention::Unadjusted);

        // Default termination business day convention to the same as business day convention
        let termination_business_day_convention = self
            .termination_business_day_convention
            .unwrap_or(business_day_convention);

        let business_day_convention_fixing_dates = self
            .business_day_convention_fixing_dates
            .unwrap_or(business_day_convention);

        let termination_business_day_convention_fixing_dates = self
            .termination_business_day_convention_fixing_dates
            .unwrap_or(business_day_convention_fixing_dates);

        let date_generation_rule = self
            .date_generation_rule
            .unwrap_or(DateGenerationRule::Backward);

        match structure {
            Structure::Bullet | Structure::Zero => {
                // make schedule for fixing dates
                let mut schedule_builder =
                    MakeSchedule::new(adjusted_start_date, adjusted_end_date)
                        .with_frequency(payment_frequency)?
                        .with_calendar(calendar.clone())
                        .with_convention(business_day_convention_fixing_dates)
                        .with_termination_date_convention(
                            termination_business_day_convention_fixing_dates,
                        )
                        .with_rule(date_generation_rule);

                let fixing_schedule = schedule_builder.build()?;
                let fixings_dates = fixing_schedule.dates();

                // make schedule for accrual dates
                let mut schedule_builder =
                    MakeSchedule::new(adjusted_start_date, adjusted_end_date)
                        .with_frequency(payment_frequency)?
                        .with_calendar(calendar.clone())
                        .with_convention(business_day_convention)
                        .with_termination_date_convention(termination_business_day_convention)
                        .with_rule(date_generation_rule);

                let accrual_schedule = schedule_builder.build()?;
                let accrual_dates = accrual_schedule.dates();

                let payment_dates = match self.payment_lag {
                    Some(lag) => accrual_dates
                        .iter()
                        .map(|d| {
                            calendar.advance(*d, lag, Some(BusinessDayConvention::Following), false)
                        })
                        .collect(),
                    None => accrual_dates.clone(),
                };

                let last_payment_date = payment_dates.last().ok_or(AtlasError::ValueNotSetErr(
                    "Payment dates should have at least one date".into(),
                ))?;

                let maturity_date = accrual_dates.last().ok_or(AtlasError::ValueNotSetErr(
                    "Accrual schedule should have at least one date".into(),
                ))?;

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
                        Some(pay_currency),
                        CashflowType::Disbursement,
                    );
                }

                if self.final_flow {
                    add_cashflows_to_vec(
                        &mut cashflows,
                        &last_date,
                        &vec![notional],
                        side,
                        currency,
                        Some(pay_currency), 
                        CashflowType::Redemption,
                    );
                }

                build_coupons_from_notionals(
                    &mut cashflows,
                    &accrual_dates,
                    &fixings_dates,
                    &payment_dates,
                    &notionals,
                    spread,
                    rate_definition,
                    side,
                    currency,
                );

                match self.discount_curve_id {
                    Some(id) => cashflows.iter_mut().for_each(|cf| {
                        cf.set_discount_curve_id(id);
                    }),
                    None => (),
                }
                match self.forecast_curve_id {
                    Some(id) => cashflows.iter_mut().for_each(|cf| {
                        cf.set_forecast_curve_id(id);
                    }),
                    None => (),
                }

                Ok(Leg::new(
                    self.negotiation_date,
                    adjusted_start_date,
                    *maturity_date,
                    *last_payment_date,
                    notional,
                    payment_frequency,
                    structure,
                    RateType::Floating,
                    spread,
                    rate_definition,
                    currency,
                    pay_currency,
                    side,
                    self.discount_curve_id,
                    self.forecast_curve_id,
                    cashflows,
                ))
            }
            _ => todo!(),
        }
    }
}

fn build_coupons_from_notionals(
    cashflows: &mut Vec<Cashflow>,
    accrual_dates: &Vec<Date>,
    fixing_dates: &Vec<Date>,
    payment_dates: &Vec<Date>,
    notionals: &Vec<f64>,
    spread: f64,
    rate_definition: RateDefinition,
    side: Side,
    currency: Currency,
) {
    for (((accrual_date_pair, fixing_date_pair), payment_date_pair), notional) in accrual_dates
        .windows(2)
        .zip(fixing_dates.windows(2))
        .zip(payment_dates.windows(2))
        .zip(notionals)
    {
        let d1 = accrual_date_pair[0];
        let d2 = accrual_date_pair[1];
        let payment_date = payment_date_pair[1];
        let mut coupon = FloatingRateCoupon::new(
            *notional,
            spread,
            d1,
            d2,
            payment_date,
            rate_definition,
            currency,
            side,
        );
        coupon.with_fixing_dates(fixing_date_pair[0], fixing_date_pair[1]);
        cashflows.push(Cashflow::FloatingRateCoupon(coupon));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cashflows::{
            cashflow::Cashflow,
            side::Side,
            traits::{InterestAccrual, Payable, RequiresFixingRate},
        },
        currencies::enums::Currency,
        instruments::{constructors::makefloatingrateleg::MakeFloatingRateLeg, traits::Structure},
        rates::{enums::Compounding, interestrate::RateDefinition},
        time::{
            calendar::Calendar,
            calendars::{
                chile::Chile,
                unitedstates::{UnitedStates, UnitedStatesMarket},
            },
            date::Date,
            daycounter::DayCounter,
            enums::{BusinessDayConvention, Frequency, TimeUnit},
            period::Period,
        },
        visitors::traits::HasCashflows,
    };

    #[test]
    fn test_make_floating_rate_leg_pay() {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate_defintion = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let spread = 0.0;

        let notional = 1_000_000.0;
        let mut instrument = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate_definition(rate_defintion)
            .with_spread(spread)
            .with_notional(notional)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .bullet()
            .build()
            .unwrap();

        let fixing_rate = 0.05;
        for cf in instrument.mut_cashflows() {
            cf.set_fixing_rate(fixing_rate);
            assert!(cf.amount().unwrap() > 0.0);
        }

        let fixing_rate = -0.05;
        for cf in instrument.mut_cashflows() {
            cf.set_fixing_rate(fixing_rate);
            match cf {
                Cashflow::Redemption(_) => {
                    assert!(cf.amount().unwrap() > 0.0);
                }
                Cashflow::FloatingRateCoupon(_) => {
                    assert!(cf.amount().unwrap() < 0.0);
                }
                _ => (),
            };
        }
    }

    #[test]
    fn test_make_floating_rate_leg_receive() {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate_defintion = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let spread = 0.0;

        let notional = 1_000_000.0;
        let mut instrument = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate_definition(rate_defintion)
            .with_spread(spread)
            .with_notional(notional)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .bullet()
            .build()
            .unwrap();

        let fixing_rate = 0.05;
        for cf in instrument.mut_cashflows() {
            cf.set_fixing_rate(fixing_rate);
            assert!(cf.amount().unwrap() > 0.0);
        }

        let fixing_rate = -0.05;
        for cf in instrument.mut_cashflows() {
            cf.set_fixing_rate(fixing_rate);
            match cf {
                Cashflow::Redemption(_) => {
                    assert!(cf.amount().unwrap() > 0.0);
                }
                Cashflow::FloatingRateCoupon(_) => {
                    assert!(cf.amount().unwrap() < 0.0);
                }
                _ => (),
            };
        }
    }

    #[test]
    fn test_payment_date_in_make_floating_rate_leg_with_sofr_calendar() {
        let cal = Calendar::UnitedStates(UnitedStates::new(UnitedStatesMarket::Sofr));
        let start_date = Date::new(2025, 7, 25);
        let end_date = start_date + Period::new(10, TimeUnit::Years);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let notional = 1_000_000.0;

        let fix_leg = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_notional(notional)
            .with_side(Side::Receive)
            .with_rate_definition(rate_definition)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Annual)
            .with_calendar(Some(cal))
            .with_business_day_convention(Some(BusinessDayConvention::ModifiedFollowing))
            .bullet()
            .with_final_flow(false)
            .build()
            .unwrap();

        let expected_payment_day = vec![
            Date::new(2025, 7, 25),
            Date::new(2026, 7, 27),
            Date::new(2027, 7, 26),
            Date::new(2028, 7, 25),
            Date::new(2029, 7, 25),
            Date::new(2030, 7, 25),
            Date::new(2031, 7, 25),
            Date::new(2032, 7, 26),
            Date::new(2033, 7, 25),
            Date::new(2034, 7, 25),
            Date::new(2035, 7, 25),
        ];

        fix_leg.cashflows().for_each(|cf| {
            assert!(expected_payment_day.contains(&cf.payment_date()));
        });
    }

    #[test]
    fn test_payment_date_in_make_floating_rate_leg_with_sofr_calendar_and_settlement_period() {
        let cal = Calendar::UnitedStates(UnitedStates::new(UnitedStatesMarket::Sofr));
        let start_date = Date::new(2025, 7, 23);
        let tenor = Period::new(10, TimeUnit::Years);
        let set_period = Period::new(2, TimeUnit::Days);
        let rate_definition = RateDefinition::new(
            DayCounter::Thirty360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let notional = 1_000_000.0;

        let fix_leg = MakeFloatingRateLeg::new()
            .with_negotiation_date(start_date)
            .with_settlement_period(set_period)
            .with_tenor(tenor)
            .with_notional(notional)
            .with_rate_definition(rate_definition)
            .with_side(Side::Receive)
            .with_currency(Currency::USD)
            .with_discount_curve_id(Some(0))
            .with_payment_frequency(Frequency::Annual)
            .with_calendar(Some(cal))
            .with_business_day_convention(Some(BusinessDayConvention::ModifiedFollowing))
            .bullet()
            .with_final_flow(false)
            .build()
            .unwrap();

        let expected_payment_day = vec![
            Date::new(2025, 7, 25),
            Date::new(2026, 7, 27),
            Date::new(2027, 7, 26),
            Date::new(2028, 7, 25),
            Date::new(2029, 7, 25),
            Date::new(2030, 7, 25),
            Date::new(2031, 7, 25),
            Date::new(2032, 7, 26),
            Date::new(2033, 7, 25),
            Date::new(2034, 7, 25),
            Date::new(2035, 7, 25),
        ];

        fix_leg.cashflows().for_each(|cf| {
            assert!(expected_payment_day.contains(&cf.payment_date()));
        });
    }

    #[test]
    fn test_make_floating_rate_leg_pay_zero_structure() {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(5, TimeUnit::Years);
        let rate_defintion = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let spread = 0.0;

        let notional = 1_000_000.0;
        let mut instrument = MakeFloatingRateLeg::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_rate_definition(rate_defintion)
            .with_spread(spread)
            .with_notional(notional)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .zero()
            .build()
            .unwrap();

        assert!(instrument.payment_frequency() == Frequency::Once);
        assert!(instrument.structure() == Structure::Zero);
        assert!(instrument.cashflows_as_vec().len() == 2);

        let fixing_rate = 0.05;
        for cf in instrument.mut_cashflows() {
            cf.set_fixing_rate(fixing_rate);
            assert!(cf.amount().unwrap() > 0.0);
        }

        let fixing_rate = -0.05;
        for cf in instrument.mut_cashflows() {
            cf.set_fixing_rate(fixing_rate);
            match cf {
                Cashflow::Redemption(_) => {
                    assert!(cf.amount().unwrap() > 0.0);
                }
                Cashflow::FloatingRateCoupon(_) => {
                    assert!(cf.amount().unwrap() < 0.0);
                }
                _ => (),
            };
        }
    }

    #[test]
    fn test_make_floating_rate_leg_pay_zero_structure_with_settlement() {
        let start_date = Date::new(2025, 8, 11);
        let rate_defintion = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate_definition = RateDefinition::default();

        let calendar = Calendar::Chile(Chile::default());
        let notional = 1_000_000.0;
        let instrument = MakeFloatingRateLeg::new()
            .with_calendar(Some(calendar))
            .with_negotiation_date(start_date)
            .with_tenor(Period::new(1, TimeUnit::Months))
            .with_rate_definition(rate_defintion)
            .with_settlement_period(Period::new(2, TimeUnit::Days))
            .with_payment_lag(Period::new(0, TimeUnit::Days))
            .with_rate_definition(rate_definition)
            .with_notional(notional)
            .with_side(Side::Pay)
            .with_currency(Currency::USD)
            .zero()
            .build()
            .unwrap();

        assert!(instrument.last_payment_date() == Date::new(2025, 9, 15));
        instrument.cashflows().for_each(|cf| {
            assert!(cf.payment_date() == Date::new(2025, 9, 15));
            match cf {
                Cashflow::FixedRateCoupon(coupon) => {
                    assert!(coupon.accrual_start_date().unwrap() == Date::new(2025, 8, 13));
                    assert!(coupon.accrual_end_date().unwrap() == Date::new(2025, 9, 13));
                }
                _ => (),
            };
        });
    }

    #[test]
    fn test_make_floating_rate_leg_pay_bullet_structure_with_settlement() {
        let start_date = Date::new(2025, 8, 11);
        let rate_defintion = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Compounded,
            Frequency::Annual,
        );
        let rate_definition = RateDefinition::default();

        let calendar = Calendar::Chile(Chile::default());
        let notional = 1_000_000.0;
        let instrument = MakeFloatingRateLeg::new()
            .with_calendar(Some(calendar))
            .with_negotiation_date(start_date)
            .with_tenor(Period::new(8, TimeUnit::Years))
            .with_rate_definition(rate_defintion)
            .with_settlement_period(Period::new(2, TimeUnit::Days))
            .with_business_day_convention(Some(BusinessDayConvention::ModifiedFollowing))
            .with_business_day_convention_fixing_dates(Some(BusinessDayConvention::Unadjusted))
            .with_payment_lag(Period::new(0, TimeUnit::Days))
            .with_rate_definition(rate_definition)
            .with_notional(notional)
            .with_side(Side::Pay)
            .with_payment_frequency(Frequency::Annual)
            .with_currency(Currency::USD)
            .bullet()
            .build()
            .unwrap();
        assert!(instrument.payment_frequency() == Frequency::Annual);
        assert!(instrument.structure() == Structure::Bullet);
        instrument
            .cashflows()
            .filter(|cf| cf.payment_date() == Date::new(2025, 8, 13))
            .for_each(|cf| {
                assert!(cf.accrual_start_date().unwrap() != cf.fixing_start_date().unwrap().unwrap());
            });
    }
}
