use crate::{
    currencies::enums::Currency, math::interpolation::enums::Interpolator, rates::{enums::Compounding, interestrate::InterestRate, traits::{HasReferenceDate, YieldProvider}}, time::{date::Date, daycounter::DayCounter, enums::Frequency}, utils::errors::{AtlasError, Result}
};

pub struct BootstrappingCurve {
    reference_date: Date,
    id: usize,
    currency: Currency,
    dates: Vec<Date>,
    year_fractions: Vec<f64>, 
    discount_factors: Vec<f64>,
    day_counter: DayCounter,
    interpolator: Interpolator,
    enable_extrapolation: bool,
}

impl BootstrappingCurve {
    pub fn new(reference_date: Date, id: usize, currency: Currency) -> Self {
        BootstrappingCurve {
            reference_date,
            id,
            currency,
            dates: vec![reference_date], 
            year_fractions: vec![0.0], // Start with 0.0 for the reference date
            discount_factors: vec![1.0], // Start with 1.0 for the reference date
            day_counter: DayCounter::Actual360, // Default day counter
            interpolator: Interpolator::LogLinear,// Default interpolator
            enable_extrapolation: true, // Default to true
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
    
    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn dates(&self) -> &Vec<Date> {
        &self.dates
    }

    pub fn year_fractions(&self) -> &Vec<f64> {
        &self.year_fractions
    }

    pub fn discount_factors(&self) -> &Vec<f64> {
        &self.discount_factors
    }

    pub fn day_counter(&self) -> &DayCounter {
        &self.day_counter
    }

    pub fn interpolator(&self) -> &Interpolator {
        &self.interpolator
    }

    pub fn enable_extrapolation(&self) -> bool {
        self.enable_extrapolation
    }

    pub fn set_interpolator(&mut self, interpolator: Interpolator) {
        self.interpolator = interpolator;
    }

    pub fn set_day_counter(&mut self, day_counter: DayCounter) {
        self.day_counter = day_counter;
    }

    pub fn set_enable_extrapolation(&mut self, enable: bool) {
        self.enable_extrapolation = enable;
    }

    pub fn add_date(&mut self, date: Date) -> Result<()> {
        if date < self.reference_date {
            return Err(AtlasError::BootstrappingErr(
                "Date in bootstrapping curve must be after the reference date".to_string(),
            ));
        }

        let year_fraction = self.day_counter.year_fraction(self.reference_date, date);

        match self.dates.binary_search(&date) {
            Ok(pos) | Err(pos) => {
                self.dates.insert(pos, date);
                self.discount_factors.insert(pos, 1.0); // Initialize with a default value 1.0
                self.year_fractions.insert(pos, year_fraction);
            }
        }
        Ok(())
    }
}

impl HasReferenceDate for BootstrappingCurve {
    fn reference_date(&self) -> Date {
        self.reference_date
    }
}

impl YieldProvider for BootstrappingCurve {
    fn discount_factor(&self, date: Date) -> Result<f64> {
        if date < self.reference_date() {
            return Err(AtlasError::BootstrappingErr(
                format!("Date {} needs to be greater than reference date {}", date, self.reference_date())
            ));
        }
        if date == self.reference_date() {
            return Ok(1.0);
        }

        let year_fraction = self
            .day_counter()
            .year_fraction(self.reference_date(), date);

        let discount_factor = self.interpolator.interpolate(
            year_fraction,
            &self.year_fractions,
            &self.discount_factors,
            true,
        );
        return Ok(discount_factor);
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<f64> {
        let discount_factor_to_star = self.discount_factor(start_date)?;
        let discount_factor_to_end = self.discount_factor(end_date)?;

        let comp_factor = discount_factor_to_star / discount_factor_to_end;
        let t = self.day_counter().year_fraction(start_date, end_date);

        return Ok(
            InterestRate::implied_rate(comp_factor, *self.day_counter(), comp, freq, t)?.rate(),
        );
    }

}

#[cfg(test)]
mod test {
    use crate::{
        currencies::enums::Currency, rates::bootstrapping::bootstrappingcurve::BootstrappingCurve,
        time::date::Date, utils::errors::Result,
    };

    #[test]
    fn test_bootstrapping_curve_add_date() -> Result<()> {
        let ref_date = Date::new(2022, 1, 1);
        let mut curve = BootstrappingCurve::new(ref_date, 1, Currency::USD);
        let date1 = Date::new(2023, 1, 1);
        let date2 = Date::new(2023, 6, 1);
        let date3 = Date::new(2023, 2, 1);

        curve.add_date(date1)?;
        curve.add_date(date2)?;
        curve.add_date(date3)?;

        assert_eq!(curve.dates, vec![ref_date, date1, date3, date2]);
        assert_eq!(curve.discount_factors, vec![1.0, 1.0, 1.0, 1.0]);

        Ok(())
    }
}
