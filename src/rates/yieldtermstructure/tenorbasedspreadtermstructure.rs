use std::sync::Arc;

use crate::{
    rates::{
        enums::Compounding,
        traits::{HasReferenceDate, YieldProvider},
    },
    time::{
        date::Date,
        enums::{Frequency, TimeUnit},
        period::Period,
    },
    utils::errors::Result,
};

use super::traits::{AdvanceTermStructureInTime, YieldTermStructureTrait};

/// # TenorBasedSpreadRateTermStructure
/// Struct that defines a term structure made with a combination of two curves. It's defined as diference between two curves. 
/// Calculate de implicit premium  between two curves.
/// 
/// Here the forward rate have dependency on the tenor
///
/// ## Forward Rate
/// The forward rate is calculated a the rate between reference date and the (start_date + tenor)
///

/// ## Parameters
/// * `reference_date` - The reference date of the term structure
/// * `spread_curve` - The spread curve
/// * `base_curve` - The base curve
///

#[derive(Clone)]
pub struct TenorBasedSpreadRateTermStructure {
    date_reference: Date, // reference_date
    spread_curve: Arc<dyn YieldTermStructureTrait>,
    base_curve: Arc<dyn YieldTermStructureTrait>,
}

impl TenorBasedSpreadRateTermStructure {
    pub fn new(
        spread_curve: Arc<dyn YieldTermStructureTrait>,
        base_curve: Arc<dyn YieldTermStructureTrait>,
    ) -> TenorBasedSpreadRateTermStructure {
        TenorBasedSpreadRateTermStructure {
            date_reference: base_curve.reference_date(),
            spread_curve,
            base_curve,
        }
    }
    pub fn new_with_date(
        reference_date: Date,
        spread_curve: Arc<dyn YieldTermStructureTrait>,
        base_curve: Arc<dyn YieldTermStructureTrait>,
    ) -> TenorBasedSpreadRateTermStructure {
        TenorBasedSpreadRateTermStructure {
            date_reference: reference_date,
            spread_curve,
            base_curve,
        }
    }

    pub fn spread_curve(&self) -> &dyn YieldTermStructureTrait {
        return self.spread_curve.as_ref();
    }

    pub fn base_curve(&self) -> &dyn YieldTermStructureTrait {
        return self.base_curve.as_ref();
    }
}

impl HasReferenceDate for TenorBasedSpreadRateTermStructure {
    fn reference_date(&self) -> Date {
        return self.date_reference;
    }
}

impl YieldProvider for TenorBasedSpreadRateTermStructure {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        let days = (date - self.date_reference) as i32;
        let eval_date_spread =
            self.spread_curve().reference_date() + Period::new(days, TimeUnit::Days);
        let spread_discount_factor = self.spread_curve.discount_factor(eval_date_spread)?;
        let eval_date_base = self.base_curve().reference_date() + Period::new(days, TimeUnit::Days);
        let base_discount_factor = self.base_curve.discount_factor(eval_date_base)?;
        let add_df = spread_discount_factor / base_discount_factor;
        return Ok(add_df);
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<f64> {
        let days = (end_date - start_date) as i32;
        let eval_date_spread =
            self.spread_curve().reference_date() + Period::new(days, TimeUnit::Days);
        let spread_forward_rate = self.spread_curve.forward_rate(
            self.spread_curve().reference_date(),
            eval_date_spread,
            comp,
            freq,
        )?;

        let eval_date_base = self.base_curve().reference_date() + Period::new(days, TimeUnit::Days);
        let base_forward_rate = self.base_curve.forward_rate(
            self.base_curve().reference_date(),
            eval_date_base,
            comp,
            freq,
        )?;
        return Ok(spread_forward_rate - base_forward_rate);
    }
}

impl AdvanceTermStructureInTime for TenorBasedSpreadRateTermStructure {
    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let new_reference_date = self.reference_date() + period;
        Ok(Arc::new(TenorBasedSpreadRateTermStructure::new_with_date(
            new_reference_date,
            Arc::clone(&self.spread_curve),
            Arc::clone(&self.base_curve),
        )))
    }

    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let days = (date - self.reference_date()) as i32;
        let period = Period::new(days, TimeUnit::Days);
        self.advance_to_period(period)
    }
}

impl YieldTermStructureTrait for TenorBasedSpreadRateTermStructure {}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use crate::{
        math::interpolation::enums::Interpolator,
        rates::{
            enums::Compounding,
            interestrate::RateDefinition,
            traits::{HasReferenceDate, YieldProvider},
            yieldtermstructure::{
                flatforwardtermstructure::FlatForwardTermStructure, traits::AdvanceTermStructureInTime, zeroratetermstructure::ZeroRateTermStructure
            },
        },
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
    };

    use super::TenorBasedSpreadRateTermStructure;

    #[test]
    fn test_reference_date() {
        let spread_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.1,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let base_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.2,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));
        let spreaded_curve = TenorBasedSpreadRateTermStructure::new(spread_curve, base_curve);
        assert!(spreaded_curve.reference_date() == Date::new(2020, 1, 1));
    }

    #[test]
    fn test_forward_rate() {
        let spread_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.03,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let base_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.02,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));
        let spreaded_curve = TenorBasedSpreadRateTermStructure::new(spread_curve, base_curve);

        let fr = spreaded_curve.forward_rate(
            Date::new(2020, 1, 1),
            Date::new(2022, 1, 1),
            Compounding::Compounded,
            Frequency::Annual,
        );
        assert!((fr.unwrap() - 0.01) < 0.0001);
    }

    #[test]
    fn test_discount_factor() -> Result<()> {
        let spread_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.3,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let df_spread_curve = spread_curve.discount_factor(Date::new(2021, 1, 1))?;

        let base_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.2,
            RateDefinition::new(
                DayCounter::Actual360,
                Compounding::Compounded,
                Frequency::Annual,
            ),
        ));

        let df_base_curve = base_curve.discount_factor(Date::new(2021, 1, 1))?;

        let spreaded_curve = TenorBasedSpreadRateTermStructure::new(spread_curve, base_curve);
        let df = spreaded_curve.discount_factor(Date::new(2021, 1, 1))?;
        assert!((df - 0.9218463178289967).abs() < 0.0001);
        assert!((df_base_curve * df - df_spread_curve).abs() < 0.0001);

        Ok(())
    }

    #[test]
    fn test_advance_time_spreadd() -> Result<()> {
        let reference_date = Date::new(2021, 1, 1);
        let dates = vec![
            Date::new(2021, 1, 1),
            Date::new(2021, 4, 1),
            Date::new(2021, 7, 1),
            Date::new(2021, 10, 1),
            Date::new(2022, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::default();

        let base_curve = Arc::new(
            ZeroRateTermStructure::new(
                reference_date,
                dates.clone(),
                rates,
                rate_definition,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let rates = vec![0.01, 0.02, 0.03, 0.04, 0.05];

        let spread_curve = Arc::new(
            ZeroRateTermStructure::new(
                reference_date,
                dates,
                rates,
                rate_definition,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let spreaded_curve = TenorBasedSpreadRateTermStructure::new(spread_curve, base_curve);

        let rate = spreaded_curve
            .forward_rate(
                Date::new(2021, 1, 1),
                Date::new(2022, 1, 1),
                Compounding::Simple,
                Frequency::Annual,
            )
            .unwrap();

        assert!((rate - 0.01).abs() < 0.000001);

        let rate = spreaded_curve
            .forward_rate(
                Date::new(2021, 1, 1),
                Date::new(2021, 7, 1),
                Compounding::Simple,
                Frequency::Annual,
            )
            .unwrap();

        assert!((rate - 0.01).abs() < 0.000001);

        let df = spreaded_curve.discount_factor(Date::new(2022, 1, 1))?;
        assert!((df - 0.9903502974223397).abs() < 0.00001);

        let new_curve = spreaded_curve.advance_to_period(Period::new(1, TimeUnit::Years))?;

        let rate = new_curve
            .forward_rate(
                Date::new(2022, 1, 1),
                Date::new(2023, 1, 1),
                Compounding::Simple,
                Frequency::Annual,
            )
            .unwrap();

        assert!((rate - 0.01).abs() < 0.000001);

        let rate = new_curve
            .forward_rate(
                Date::new(2022, 1, 1),
                Date::new(2023, 7, 1),
                Compounding::Simple,
                Frequency::Annual,
            )
            .unwrap();

        assert!((rate - 0.01).abs() < 0.000001);

        let df = new_curve.discount_factor(Date::new(2023, 1, 1))?;
        println!("df: {:?}", df);
        assert!((df - 0.9903502974223397).abs() < 0.00001);

        Ok(())
    }


    
}
