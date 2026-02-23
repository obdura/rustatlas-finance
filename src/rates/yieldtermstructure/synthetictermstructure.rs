use std::{any::Any, sync::Arc};

use crate::{
    rates::{
        enums::Compounding,
        interestrate::RateDefinition,
        traits::{HasReferenceDate, YieldProvider},
        yieldtermstructure::traits::{AdvanceTermStructureInTime, YieldTermStructureTrait},
    },
    time::{date::Date, daycounter::DayCounter, enums::Frequency, period::Period},
    utils::errors::Result,
};

#[derive(Clone)]
pub struct SyntheticTermStructure {
    date_reference: Date, // reference_date
    discount_factor_numerator: Vec<Arc<dyn YieldTermStructureTrait>>,
    discount_factor_denominator: Vec<Arc<dyn YieldTermStructureTrait>>,
}

impl SyntheticTermStructure {
    pub fn new(
        date_reference: Date,
        discount_factor_numerator: Vec<Arc<dyn YieldTermStructureTrait>>,
        discount_factor_denominator: Vec<Arc<dyn YieldTermStructureTrait>>
    ) -> Self {
        Self {
            date_reference,
            discount_factor_numerator,
            discount_factor_denominator,
        }
    }

    pub fn discount_factor_denominator(&self) -> &Vec<Arc<dyn YieldTermStructureTrait>> {
        &self.discount_factor_denominator
    }

    pub fn discount_factor_numerator(&self) -> &Vec<Arc<dyn YieldTermStructureTrait>> {
        &self.discount_factor_numerator
    }
}

impl HasReferenceDate for SyntheticTermStructure {
    fn reference_date(&self) -> Date {
        return self.date_reference;
    }
}

impl YieldProvider for SyntheticTermStructure {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        let mut denominator = 1.0;
        let mut numerator = 1.0;

        for i in 0..self.discount_factor_denominator().len() {
            let df = self.discount_factor_denominator()[i].discount_factor(date)?;
            denominator *= df;
        }

        for i in 0..self.discount_factor_numerator().len() {
            let df = self.discount_factor_numerator()[i].discount_factor(date)?;
            numerator *= df;
        }
        return Ok(numerator / denominator);
    }

    fn discount_factor_between_dates(&self, start_date: Date, end_date: Date) -> Result<f64> {
        let mut denominator = 1.0;
        let mut numerator = 1.0;

        for i in 0..self.discount_factor_denominator().len() {
            let df = self.discount_factor_denominator()[i]
                .discount_factor_between_dates(start_date, end_date)?;
            denominator *= df;
        }

        for i in 0..self.discount_factor_numerator().len() {
            let df = self.discount_factor_numerator()[i]
                .discount_factor_between_dates(start_date, end_date)?;
            numerator *= df;
        }
        return Ok(numerator / denominator);
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
        let yf = day_counter.year_fraction(start_date, end_date);
        let rate_definition = RateDefinition::new(day_counter, comp, freq);
        return Ok(rate_definition.implied_rate(comp_factor, yf)?.rate());
    }
}

// Implement the AdvanceTermStructureInTime trait for CompositeTermStructure
impl AdvanceTermStructureInTime for SyntheticTermStructure {
    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let mut discount_factor_denominator = vec![];
        let mut discount_factor_numerator = vec![];

        for i in 0..self.discount_factor_denominator().len() {
            let df = self.discount_factor_denominator()[i].advance_to_date(date)?;
            discount_factor_denominator.push(df);
        }

        for i in 0..self.discount_factor_numerator().len() {
            let df = self.discount_factor_numerator()[i].advance_to_date(date)?;
            discount_factor_numerator.push(df);
        }

        Ok(Arc::new(SyntheticTermStructure::new(
            date,
            discount_factor_denominator,
            discount_factor_numerator,
        )))
    }

    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait>> {
        let new_reference_date = self.reference_date() + period;
        self.advance_to_date(new_reference_date)
    }
}

// Implement the YieldTermStructureTrait trait for CompositeTermStructure
impl YieldTermStructureTrait for SyntheticTermStructure {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        math::interpolation::enums::Interpolator,
        rates::{traits::YieldProvider, yieldtermstructure::{
            discounttermstructure::DiscountTermStructure,
            synthetictermstructure::SyntheticTermStructure,
        }},
        time::{date::Date, daycounter::DayCounter},
        utils::errors::Result,
    };

    #[test]
    fn test_synthetic_term_structure() -> Result<()> {
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure_1 = DiscountTermStructure::new(
            dates.clone(),
            discount_factors,
            day_counter,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let discount_factors_2 = vec![1.0, 0.98, 0.96, 0.94, 0.92];

        let discount_term_structure_2 = DiscountTermStructure::new(
            dates,
            discount_factors_2,
            day_counter,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let synthetic_term_structure = SyntheticTermStructure::new(
            Date::new(2020, 1, 1),
            vec![Arc::new(discount_term_structure_1)],
            vec![Arc::new(discount_term_structure_2)],
        );

        let df = synthetic_term_structure.discount_factor(Date::new(2020, 7, 1))?;

        assert!((df - 0.98/0.96).abs() < 0.00001);
        
        Ok(())
    }

    
    #[test]
    fn test_synthetic_term_structure_2() -> Result<()> {
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure_1 = DiscountTermStructure::new(
            dates.clone(),
            discount_factors,
            day_counter,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let discount_factors_2 = vec![1.0, 0.98, 0.96, 0.94, 0.92];

        let discount_term_structure_2 = DiscountTermStructure::new(
            dates,
            discount_factors_2,
            day_counter,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let synthetic_term_structure = SyntheticTermStructure::new(
            Date::new(2020, 1, 1),
            vec![Arc::new(discount_term_structure_1), Arc::new(discount_term_structure_2)],
            vec![],
        );

        let df = synthetic_term_structure.discount_factor(Date::new(2020, 7, 1))?;

        assert!((df - 0.98*0.96).abs() < 0.00001);
        
        Ok(())
    }

    #[test]
    fn test_synthetic_term_structure_3() -> Result<()> {
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let discount_factors = vec![1.0, 0.99, 0.98, 0.97, 0.96];
        let day_counter = DayCounter::Actual360;

        let discount_term_structure_1 = DiscountTermStructure::new(
            dates.clone(),
            discount_factors,
            day_counter,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let discount_factors_2 = vec![1.0, 0.98, 0.96, 0.94, 0.92];

        let discount_term_structure_2 = DiscountTermStructure::new(
            dates,
            discount_factors_2,
            day_counter,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let synthetic_term_structure = SyntheticTermStructure::new(
            Date::new(2020, 1, 1),
            vec![],
            vec![Arc::new(discount_term_structure_1), Arc::new(discount_term_structure_2)],
        );

        let df = synthetic_term_structure.discount_factor(Date::new(2020, 7, 1))?;

        assert!((df - 1.0/(0.98*0.96)).abs() < 0.00001);
        
        Ok(())
    }
}
