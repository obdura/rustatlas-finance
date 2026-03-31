use crate::{
    cashflows::{cashflow::Cashflow, traits::Payable},
    math::solver::{brentopt::BrentOpt, traits::CostFunction},
    rates::
        interestrate::{InterestRate, RateDefinition}
    ,
    time::date::Date,
    utils::errors::Result,
    visitors::traits::{ConstVisit, HasCashflows},
};

pub struct YieldRateConstVisitor {
    rate_definition: RateDefinition,
    reference_date: Date,
    include_today_cashflows: bool,
}

impl YieldRateConstVisitor {
    pub fn new(
        reference_date: Date,
        rate_definition: RateDefinition,
        include_today_cashflows: bool,
    ) -> Self {
        YieldRateConstVisitor {
            rate_definition,
            reference_date,
            include_today_cashflows,
        }
    }

    pub fn set_include_today_cashflows(&mut self, include_today_cashflows: bool) {
        self.include_today_cashflows = include_today_cashflows;
    }

    pub fn set_reference_date(&mut self, reference_date: Date) {
        self.reference_date = reference_date;
    }

    pub fn set_rate_definition(&mut self, rate_definition: RateDefinition) {
        self.rate_definition = rate_definition;
    }
}

struct YieldRateNPV<'a, T> {
    eval: &'a T,
    rate_definition: RateDefinition,
    reference_date: Date,
    include_today_cashflows: bool,
}

impl<'a, T> YieldRateNPV<'a, T>
where
    T: HasCashflows,
{
    fn cashflow_npv(&self, cf: &Cashflow, rate_value: f64) -> Result<f64> {
        if !self.include_today_cashflows && cf.payment_date() == self.reference_date {
            return Ok(0.0);
        }

        let t = self
            .rate_definition
            .day_counter()
            .year_fraction(self.reference_date, cf.payment_date());

        if t < 0.0 {
            return Ok(0.0);
        }

        let rate = InterestRate::from_rate_definition(rate_value, self.rate_definition);
        let compounding = rate.compound_factor(self.reference_date, cf.payment_date());

        let df = 1.0 / compounding;
        let flag = cf.side().sign();
        let cf_npv = cf.amount()? * flag * df;
        Ok(cf_npv)
    }
}

impl<'a, T> CostFunction for YieldRateNPV<'a, T>
where
    T: HasCashflows,
{
    fn cost(&self, param: &f64) -> Result<f64> {
        let npv = self
            .eval
            .cashflows()
            .try_fold(0.0, |acc, cf| -> Result<f64> {
                let cf_npv = self.cashflow_npv(cf, *param)?;
                Ok(acc + cf_npv)
            })?;
        Ok(npv.abs())
    }
}


impl<'a, T> ConstVisit<T> for YieldRateConstVisitor
where
    T: HasCashflows,
{
    type Output = Result<f64>;
    fn visit(&self, visitable: &T) -> Self::Output {
        let npv = YieldRateNPV {
            eval: visitable,
            rate_definition: self.rate_definition,
            reference_date: self.reference_date,
            include_today_cashflows: self.include_today_cashflows,
        };
        let solver = BrentOpt::new(npv, -1.0, 1.0);
        let res = solver.solve()?;

        Ok(res.argmin)
    }
}
