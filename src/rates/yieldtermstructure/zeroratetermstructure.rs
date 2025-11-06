use std::sync::Arc;

use crate::{
    math::interpolation::enums::Interpolator,
    rates::{
        enums::Compounding,
        interestrate::{InterestRate, RateDefinition},
        traits::{HasReferenceDate, YieldProvider},
    },
    time::{
        date::Date, daycounter::DayCounter, enums::{Frequency, TimeUnit}, period::Period
    },
    utils::errors::{AtlasError, Result},
};

use super::traits::{AdvanceTermStructureInTime, YieldTermStructureTrait};

/// # ZeroRateTermStructure
/// It is a yield term structure that is based on zero rates.
///
/// ## Parameters
/// * `reference_date` - Reference date for the term structure.
/// * `dates` - Dates for the term structure.
/// * `rates` - Rates for the term structure.
/// * `rate_definition` - Rate definition for the term structure.
/// * `interpolator` - Interpolator for the term structure.
/// * `enable_extrapolation` - Enable extrapolation for the term structure.
///
/// ## Forward Rate
/// The forward rate is calculated a the Forward Rate between two dates (FRA)
///
/// # Example
/// ```
/// use rustatlas::prelude::*;
/// let ref_date = Date::new(2021, 1, 1);
/// let dates = vec![
///    Date::new(2021, 1, 1),
///    Date::new(2021, 4, 1),
///    Date::new(2021, 7, 1),
///    Date::new(2021, 10, 1),
///    Date::new(2022, 1, 1),
/// ];
///
/// let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
/// let rate_definition = RateDefinition::default();
/// let interpolator = Interpolator::Linear;
/// let enable_extrapolation = true;
/// let zero_rate_curve = ZeroRateTermStructure::new(ref_date, dates, rates, rate_definition, interpolator, enable_extrapolation).unwrap();
/// assert_eq!(zero_rate_curve.reference_date(), ref_date);
/// assert_eq!(zero_rate_curve.rate_definition().day_counter(), DayCounter::Actual360);
/// ```
#[derive(Clone)]
pub struct ZeroRateTermStructure {
    reference_date: Date,
    dates: Vec<Date>,
    year_fractions: Vec<f64>,
    rates: Vec<f64>,
    rate_definition: RateDefinition,
    interpolator: Interpolator,
    enable_extrapolation: bool,
}

impl ZeroRateTermStructure {
    pub fn new(
        reference_date: Date,
        dates: Vec<Date>,
        rates: Vec<f64>,
        rate_definition: RateDefinition,
        interpolator: Interpolator,
        enable_extrapolation: bool,
    ) -> Result<ZeroRateTermStructure> {
        // check if dates and rates have the same size
        if dates.len() != rates.len() {
            return Err(AtlasError::InvalidValueErr(
                "Dates and rates need to have the same size".to_string(),
            ));
        }

        ///////// Revisar si esto es necesario
        //// year_fractions[0] needs to be 0.0
        //if dates[0] != reference_date {
        //    return Err(AtlasError::InvalidValueErr(
        //        "First date needs to be equal to reference date".to_string(),
        //    ));
        //}

        let year_fractions: Vec<f64> = dates
            .iter()
            .map(|x| {
                rate_definition
                    .day_counter()
                    .year_fraction(reference_date, *x)
            })
            .collect();

        Ok(ZeroRateTermStructure {
            reference_date,
            dates,
            year_fractions,
            rates,
            rate_definition,
            interpolator,
            enable_extrapolation,
        })
    }

    pub fn dates(&self) -> &Vec<Date> {
        return &self.dates;
    }

    pub fn rates(&self) -> &Vec<f64> {
        return &self.rates;
    }

    pub fn rate_definition(&self) -> RateDefinition {
        return self.rate_definition;
    }

    pub fn enable_extrapolation(&self) -> bool {
        return self.enable_extrapolation;
    }

    pub fn interpolator(&self) -> Interpolator {
        return self.interpolator;
    }
}

impl HasReferenceDate for ZeroRateTermStructure {
    fn reference_date(&self) -> Date {
        return self.reference_date;
    }
}

impl YieldProvider for ZeroRateTermStructure {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        let year_fraction = self
            .rate_definition()
            .day_counter()
            .year_fraction(self.reference_date(), date);

        let rate = self.interpolator.interpolate(
            year_fraction,
            &self.year_fractions,
            &self.rates,
            self.enable_extrapolation,
        )?;
        let rt = InterestRate::from_rate_definition(rate, self.rate_definition());
        let compound = rt.compound_factor_from_yf(year_fraction);
        return Ok(1.0 / compound);
    }

    fn discount_factor_between_dates(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let df_start = self.discount_factor(start_date)?;
        let df_end = self.discount_factor(end_date)?;
        return Ok(df_end / df_start);
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
        return Ok(rate_definition.implied_rate(comp_factor, yf)?.rate())
    }
}

/// # AdvanceTermStructureInTime for ZeroRateTermStructure
impl AdvanceTermStructureInTime for ZeroRateTermStructure {
    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let new_reference_date = self.reference_date() + period;
        let new_dates: Vec<Date> = self
            .dates()
            .iter()
            .filter_map(|x| {
                if x > &new_reference_date {
                    Some(x.clone())
                } else {
                    None
                }
            })
            .collect();

        let new_rates: Vec<f64> = new_dates
            .iter()
            .map(|x| -> Result<f64> {
                let fwr = self.forward_rate(
                    new_reference_date,
                    *x,
                    self.rate_definition().compounding(),
                    self.rate_definition().frequency(),
                    self.rate_definition().day_counter(),
                )?;
                Ok(fwr)
            })
            .collect::<Result<Vec<f64>>>()?;

        Ok(Arc::new(ZeroRateTermStructure::new(
            new_reference_date,
            new_dates,
            new_rates,
            self.rate_definition(),
            self.interpolator(),
            self.enable_extrapolation(),
        )?))
    }

    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let days = (date - self.reference_date()) as i32;
        if days < 0 {
            return Err(AtlasError::InvalidValueErr(format!(
                "Date {:?} is before reference date {:?}",
                date,
                self.reference_date()
            )));
        }
        let period = Period::new(days, TimeUnit::Days);
        return self.advance_to_period(period);
    }
}

impl YieldTermStructureTrait for ZeroRateTermStructure {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::daycounter::DayCounter;

    #[test]
    fn test_zero_rate_curve() {
        let reference_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::default();

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates,
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        assert_eq!(zero_rate_curve.reference_date(), reference_date);
        assert_eq!(
            zero_rate_curve.dates(),
            &vec![
                Date::new(2020, 1, 1),
                Date::new(2020, 4, 1),
                Date::new(2020, 7, 1),
                Date::new(2020, 10, 1),
                Date::new(2021, 1, 1)
            ]
        );
        assert_eq!(zero_rate_curve.rates(), &vec![0.0, 0.01, 0.02, 0.03, 0.04]);
        assert_eq!(
            zero_rate_curve.rate_definition().day_counter(),
            DayCounter::Actual360
        );
    }

    #[test]
    fn test_zero_rate_curve_df_simple() {
        let reference_date = Date::new(2021, 1, 1);
        let dates = vec![
            Date::new(2021, 1, 1),
            Date::new(2021, 4, 1),
            Date::new(2021, 7, 1),
            Date::new(2021, 10, 1),
            Date::new(2022, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];

        // actual/365 
        let rate_definition = RateDefinition::new(DayCounter::Actual365, Compounding::Simple, Frequency::Annual);

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates.clone(),
            rates.clone(),
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        assert_eq!(zero_rate_curve.discount_factor(Date::new(2022, 1, 1)).unwrap(), 1.0/(1.0 + 0.04 * 365.0/365.0));

        // actual/365 
        let rate_definition = RateDefinition::new(DayCounter::Actual360, Compounding::Simple, Frequency::Annual);

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates.clone(),
            rates.clone(),
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        assert_eq!(zero_rate_curve.discount_factor(Date::new(2022, 1, 1)).unwrap(), 1.0/(1.0 + 0.04 * 365.0/360.0));
    }

    #[test]
    fn test_zero_rate_curve_df_compounded() {
        let reference_date = Date::new(2021, 1, 1);
        let dates = vec![
            Date::new(2021, 1, 1),
            Date::new(2021, 4, 1),
            Date::new(2021, 7, 1),
            Date::new(2021, 10, 1),
            Date::new(2022, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];

        // actual/365 
        let rate_definition = RateDefinition::new(DayCounter::Actual365, Compounding::Compounded, Frequency::Annual);

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates.clone(),
            rates.clone(),
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        assert_eq!(zero_rate_curve.discount_factor(Date::new(2022, 1, 1)).unwrap(), 1.0/(1.0 + 0.04_f64).powf(365.0/365.0));

        // actual/365 
        let rate_definition = RateDefinition::new(DayCounter::Actual360, Compounding::Compounded, Frequency::Annual);

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates.clone(),
            rates.clone(),
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        assert_eq!(zero_rate_curve.discount_factor(Date::new(2022, 1, 1)).unwrap(), 1.0/(1.0 + 0.04_f64).powf(365.0/360.0));
    }

    #[test]
    fn test_forward_rate() {
        let reference_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2021, 1, 1),
            Date::new(2022, 1, 1),
            Date::new(2023, 1, 1),
            Date::new(2024, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::default();

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates,
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let fr = zero_rate_curve.forward_rate(
            Date::new(2021, 1, 1),
            Date::new(2022, 1, 1),
            rate_definition.compounding(),
            rate_definition.frequency(),
            rate_definition.day_counter(),
        );

        println!("fr: {:?}", fr);
        assert!(fr.unwrap() - 0.02972519115024655 < 0.000000001);
    }

    #[test]
    fn test_zero_rate_advance() -> Result<()> {
        let reference_date = Date::new(2021, 1, 1);
        let dates = vec![
            Date::new(2021, 1, 1),
            Date::new(2022, 1, 1),
            Date::new(2023, 1, 1),
            Date::new(2024, 1, 1),
            Date::new(2025, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::new(
            DayCounter::Actual365,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates,
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let fr1 = zero_rate_curve.forward_rate(
            Date::new(2023, 1, 1),
            Date::new(2024, 1, 1),
            rate_definition.compounding(),
            rate_definition.frequency(),
            rate_definition.day_counter(),
        )?;

        let advance_zero_rate_curve = zero_rate_curve.advance_to_date(Date::new(2022, 1, 1))?;

        let fr2 = advance_zero_rate_curve.forward_rate(
            Date::new(2023, 1, 1),
            Date::new(2024, 1, 1),
            rate_definition.compounding(),
            rate_definition.frequency(),
            rate_definition.day_counter(),
        )?;

        assert!((fr1 - fr2).abs() < 0.000001);

        Ok(())
    }

    #[test]
    fn test_zero_rate_advance_2() -> Result<()> {
        let reference_date = Date::new(2021, 1, 1);
        let dates = vec![
            Date::new(2021, 1, 1),
            Date::new(2022, 1, 1),
            Date::new(2023, 1, 1),
            Date::new(2024, 1, 1),
            Date::new(2025, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::new(
            DayCounter::Actual365,
            Compounding::Compounded,
            Frequency::Annual,
        );

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates,
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let fr1 = zero_rate_curve.forward_rate(
            Date::new(2023, 2, 1),
            Date::new(2024, 2, 1),
            rate_definition.compounding(),
            rate_definition.frequency(),
            rate_definition.day_counter(),
        )?;

        let advance_zero_rate_curve = zero_rate_curve.advance_to_date(Date::new(2022, 1, 1))?;

        let fr2 = advance_zero_rate_curve.forward_rate(
            Date::new(2023, 2, 1),
            Date::new(2024, 2, 1),
            rate_definition.compounding(),
            rate_definition.frequency(),
            rate_definition.day_counter(),
        )?;

        assert!((fr1 - fr2).abs() < 0.000001);

        Ok(())
    }
}
