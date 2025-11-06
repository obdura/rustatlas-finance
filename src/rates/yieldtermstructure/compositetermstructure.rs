use std::sync::Arc;

use crate::{
    rates::{
        enums::Compounding,
        interestrate::RateDefinition,
        traits::{HasReferenceDate, YieldProvider},
    },
    time::{date::Date, daycounter::DayCounter, enums::Frequency, period::Period},
    utils::errors::Result,
};

use super::traits::{AdvanceTermStructureInTime, YieldTermStructureTrait};

/// # CompositeTermStructure
/// Struct that defines a term structure made with a combination of two curves. It's defined as:
///
/// ## Parameters
/// * `reference_date` - Reference date for the term structure.
/// * `spread_curve` - The spread curve.
/// * `base_curve` - The base curve.
///
/// ## Forward Rate
/// The forward rate is calculated as the sum of the forward rates of the spread curve and the base curve.
///
/// # Example
/// ```
/// use rustatlas::prelude::*;
/// use std::sync::Arc;
/// let ref_date = Date::new(2021, 1, 1);
///
/// let spread_curve = FlatForwardTermStructure::new(
///   ref_date,
///     0.01,
///     RateDefinition::default()
/// );
///
/// let base_curve = FlatForwardTermStructure::new(
///     ref_date,
///     0.02,
///     RateDefinition::default()
/// );
///
/// let spreaded_curve = CompositeTermStructure::new(Arc::new(spread_curve), Arc::new(base_curve));
/// assert_eq!(spreaded_curve.reference_date(), ref_date);
/// ```
#[derive(Clone)]
pub struct CompositeTermStructure {
    date_reference: Date, // reference_date
    spread_curve: Arc<dyn YieldTermStructureTrait>,
    base_curve: Arc<dyn YieldTermStructureTrait>,
}

impl CompositeTermStructure {
    pub fn new(
        spread_curve: Arc<dyn YieldTermStructureTrait>,
        base_curve: Arc<dyn YieldTermStructureTrait>,
    ) -> CompositeTermStructure {
        CompositeTermStructure {
            date_reference: base_curve.reference_date(),
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

// Implement the HasReferenceDate trait for CompositeTermStructure
impl HasReferenceDate for CompositeTermStructure {
    fn reference_date(&self) -> Date {
        return self.date_reference;
    }
}

// Implement the YieldProvider trait for CompositeTermStructure
impl YieldProvider for CompositeTermStructure {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        let spread_discount_factor = self.spread_curve.discount_factor(date)?;
        let base_discount_factor = self.base_curve.discount_factor(date)?;
        let add_df = spread_discount_factor * base_discount_factor;
        return Ok(add_df);
    }

    fn discount_factor_between_dates(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let spread_discount_factor = self
            .spread_curve
            .discount_factor_between_dates(start_date, end_date)?;
        let base_discount_factor = self
            .base_curve
            .discount_factor_between_dates(start_date, end_date)?;
        let add_df = spread_discount_factor * base_discount_factor;
        return Ok(add_df);
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
        day_counter: DayCounter,
    ) -> Result<f64> {
        let comp_factor = 1.0 / self.discount_factor_between_dates(start_date, end_date)?;

        let rate_definition = RateDefinition::new(day_counter, comp, freq);
        let yf = day_counter.year_fraction(start_date, end_date);
        return Ok(rate_definition.implied_rate(comp_factor, yf)?.rate());
    }
}

// Implement the AdvanceTermStructureInTime trait for CompositeTermStructure
impl AdvanceTermStructureInTime for CompositeTermStructure {
    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let base = self.base_curve().advance_to_date(date)?;
        let spread = self.spread_curve().advance_to_date(date)?;
        Ok(Arc::new(CompositeTermStructure::new(spread, base)))
    }

    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let base = self.base_curve().advance_to_period(period)?;
        let spread = self.spread_curve().advance_to_period(period)?;
        Ok(Arc::new(CompositeTermStructure::new(spread, base)))
    }
}

// Implement the YieldTermStructureTrait trait for CompositeTermStructure
impl YieldTermStructureTrait for CompositeTermStructure {}

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
                compositetermstructure::CompositeTermStructure,
                flatforwardtermstructure::FlatForwardTermStructure,
                tenorbasedzeroratetermstructure::TenorBasedZeroRateTermStructure,
                zeroratetermstructure::ZeroRateTermStructure,
            },
        },
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::{errors::Result, marketstoretransformation::TenorBasedValues},
    };

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
        let spreaded_curve = CompositeTermStructure::new(spread_curve, base_curve);
        assert!(spreaded_curve.reference_date() == Date::new(2020, 1, 1));
    }

    #[test]
    fn test_forward_rate() {
        let spread_curve = Arc::new(FlatForwardTermStructure::new(
            Date::new(2020, 1, 1),
            0.01,
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
        let spreaded_curve = CompositeTermStructure::new(spread_curve, base_curve);

        let fr = spreaded_curve.forward_rate(
            Date::new(2020, 1, 1),
            Date::new(2022, 1, 1),
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        assert!((fr.unwrap() - ((1.0 + 0.02) * (1.0 + 0.01) - 1.0) < 0.0001));
    }

    #[test]
    fn test_discount_factor() {
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

        let spreaded_curve = CompositeTermStructure::new(spread_curve, base_curve);

        let df = spreaded_curve.discount_factor(Date::new(2021, 1, 1));
        println!("df: {:?}", df);

        assert!(df.unwrap() - 0.9702040771633191 < 0.00001);
    }

    #[test]
    fn test_negative_rate() {
        let reference_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Continuous,
            Frequency::Annual,
        );

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates.clone(),
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let spread_curve = Arc::new(FlatForwardTermStructure::new(
            reference_date,
            -0.01,
            rate_definition,
        ));

        let spreaded_curve =
            CompositeTermStructure::new(spread_curve, Arc::new(zero_rate_curve.clone()));

        let rate = spreaded_curve
            .forward_rate(
                Date::new(2020, 1, 1),
                Date::new(2021, 1, 1),
                Compounding::Continuous,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();

        assert!((rate - 0.03).abs() < 0.0001);

        let rate = spreaded_curve
            .forward_rate(
                Date::new(2020, 1, 1),
                Date::new(2020, 7, 1),
                Compounding::Continuous,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();

        assert!((rate - 0.01).abs() < 0.000001);

        let spread = vec![-0.001, -0.002, -0.003, -0.004, -0.005];
        let spread_curve = Arc::new(
            ZeroRateTermStructure::new(
                reference_date,
                dates,
                spread,
                rate_definition,
                Interpolator::Linear,
                true,
            )
            .unwrap(),
        );

        let spreaded_curve = CompositeTermStructure::new(spread_curve, Arc::new(zero_rate_curve));

        let rate = spreaded_curve
            .forward_rate(
                Date::new(2020, 1, 1),
                Date::new(2021, 1, 1),
                Compounding::Continuous,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();

        assert!((rate - 0.035).abs() < 0.000001);
    }

    #[test]
    fn test_negative_rate_and_compound_rate() {
        let reference_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates.clone(),
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let spread_curve = Arc::new(FlatForwardTermStructure::new(
            reference_date,
            -0.01,
            rate_definition,
        ));

        let spreaded_curve =
            CompositeTermStructure::new(spread_curve, Arc::new(zero_rate_curve.clone()));

        let rate = spreaded_curve
            .forward_rate(
                Date::new(2020, 1, 1),
                Date::new(2021, 1, 1),
                Compounding::Compounded,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();

        let yf = 366.0 / 360.0;
        let rate_tmp = ((1.0_f64+ 0.04_f64).powf(yf) * (1.0_f64 - 0.01_f64).powf(yf)).powf(1.0 / yf) - 1.0;
        assert!((rate - rate_tmp).abs() < 0.00001);
    }

    #[test]
    fn test_forward_rate_2() -> Result<()> {
        let date = Date::new(2024, 3, 21);
        let fwd_start = date + Period::new(1, TimeUnit::Years);
        let fwd_end = date + Period::new(2, TimeUnit::Years);

        let mut spread_term = Vec::new();
        spread_term.push(TenorBasedValues {
            tenor: Period::new(0, TimeUnit::Years),
            value: 0.001 as f64,
        });
        spread_term.push(TenorBasedValues {
            tenor: Period::new(1, TimeUnit::Years),
            value: 0.01 as f64,
        });
        spread_term.push(TenorBasedValues {
            tenor: Period::new(2, TimeUnit::Years),
            value: 0.02 as f64,
        });
        spread_term.push(TenorBasedValues {
            tenor: Period::new(3, TimeUnit::Years),
            value: 0.03 as f64,
        });
        spread_term.push(TenorBasedValues {
            tenor: Period::new(4, TimeUnit::Years),
            value: 0.04 as f64,
        });
        spread_term.push(TenorBasedValues {
            tenor: Period::new(5, TimeUnit::Years),
            value: 0.05 as f64,
        });

        let (tenors, spread_values): (Vec<_>, Vec<_>) = spread_term
            .iter()
            .map(|v| (v.tenor.clone(), v.value))
            .unzip();

        let spread_term_structure = TenorBasedZeroRateTermStructure::new(
            date,
            tenors,
            spread_values,
            RateDefinition::default(),
            Interpolator::Linear,
            true,
        )?;

        let rate_1 = spread_term_structure
            .forward_rate(
                fwd_start,
                fwd_end,
                Compounding::Simple,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();

        assert!((rate_1 - 0.01).abs() < 0.000001);
        println!("Rate 1: {:?}", rate_1);

        let base_term_curve = FlatForwardTermStructure::new(date, 0.02, RateDefinition::default());

        let rate_2 = base_term_curve
            .forward_rate(
                fwd_start,
                fwd_end,
                Compounding::Simple,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();
        assert!(
            (rate_2
                - (((1.0 + 0.02 * 730.0 / 360.0) / (1.0 + 0.02 * 365.0 / 360.0) - 1.0) * 360.0
                    / 365.0))
                .abs()
                < 0.000001
        );
        println!("Rate 2: {:?}", rate_2);

        let composite_term_structure = Arc::new(CompositeTermStructure::new(
            Arc::new(spread_term_structure),
            Arc::new(base_term_curve),
        ));

        let rate_3 = composite_term_structure
            .forward_rate(
                fwd_start,
                fwd_end,
                Compounding::Simple,
                Frequency::Annual,
                DayCounter::Actual360,
            )
            .unwrap();

        println!("Rate 3: {:?}", rate_3);
        assert!(
            (rate_3
                - ((1.0 + rate_1 * 365.0 / 360.0) * (1.0 + rate_2 * 365.0 / 360.0) - 1.0) * 360.0
                    / 365.0)
                < 0.000001
        );
        Ok(())
    }
}
