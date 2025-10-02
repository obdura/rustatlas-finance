use crate::{
    cashflows::{
        cashflow::{Cashflow, CashflowType},
        traits::Payable,
    },
    time::date::Date,
};

/// Trait for objects that can be visited by a visitor that can mutate the object.
pub trait Visit<T> {
    type Output;
    fn visit(&self, visitable: &mut T) -> Self::Output;
}

/// Trait for objects that can be visited by a const visitor.
pub trait ConstVisit<T> {
    type Output;
    fn visit(&self, visitable: &T) -> Self::Output;
}

/// Trait for objects that have cashflows.
///
/// # methods
///
/// - `cashflows` returns an iterator over the cashflows.
/// - `mut_cashflows` returns an iterator over the cashflows that can be mutated.
/// - `cashflows_as_vec` returns a vector of references to the cashflows.
/// - `mut_cashflows_as_vec` returns a vector of mutable references to the cashflows.
/// - `set_discount_curve_id` sets the discount curve id for all cashflows.
/// - `set_forecast_curve_id` sets the forecast curve id for all floating rate cashflows.
/// - `next_cashflow` returns the next cashflow of a given type after a reference date.
/// - `first_cashflow` returns the first cashflow of a given type.
/// - `last_cashflow` returns the last cashflow of a given type.
///
///
pub trait HasCashflows {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_>;
    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_>;

    fn cashflows_as_vec(&self) -> Vec<&Cashflow> {
        self.cashflows().collect()
    }

    fn mut_cashflows_as_vec(&mut self) -> Vec<&mut Cashflow> {
        self.mut_cashflows().collect()
    }

    fn set_discount_curve_id(&mut self, id: usize) {
        self.mut_cashflows()
            .for_each(|cf| cf.set_discount_curve_id(id));
    }

    fn set_forecast_curve_id(&mut self, id: usize) {
        self.mut_cashflows().for_each(|cf| match cf {
            Cashflow::FloatingRateCoupon(frcf) => frcf.set_forecast_curve_id(id),
            _ => (),
        });
    }

    fn next_cashflow(&self, reference_date: Date, cashflow_type: CashflowType) -> Option<Cashflow> {
        match cashflow_type {
            CashflowType::Disbursement => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::Disbursement(_)))
                .filter(|cf| cf.payment_date() > reference_date)
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::Redemption => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::Redemption(_)))
                .filter(|cf| cf.payment_date() > reference_date)
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::FixedRateCoupon => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::FixedRateCoupon(_)))
                .filter(|cf| cf.payment_date() > reference_date)
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::FloatingRateCoupon => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::FloatingRateCoupon(_)))
                .filter(|cf| cf.payment_date() > reference_date)
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::IndexFxCashflow => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::IndexFxCashflow(_)))
                .filter(|cf| cf.payment_date() > reference_date)
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
        }
    }

    fn first_cashflow(&self, cashflow_type: CashflowType) -> Option<Cashflow> {
        match cashflow_type {
            CashflowType::Disbursement => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::Disbursement(_)))
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::Redemption => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::Redemption(_)))
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::FixedRateCoupon => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::FixedRateCoupon(_)))
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::FloatingRateCoupon => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::FloatingRateCoupon(_)))
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::IndexFxCashflow => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::IndexFxCashflow(_)))
                .min_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
        }
    }

    fn last_cashflow(&self, cashflow_type: CashflowType) -> Option<Cashflow> {
        match cashflow_type {
            CashflowType::Disbursement => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::Disbursement(_)))
                .max_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::Redemption => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::Redemption(_)))
                .max_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::FixedRateCoupon => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::FixedRateCoupon(_)))
                .max_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::FloatingRateCoupon => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::FloatingRateCoupon(_)))
                .max_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
            CashflowType::IndexFxCashflow => self
                .cashflows()
                .filter(|cf| matches!(cf, Cashflow::IndexFxCashflow(_)))
                .max_by(|cf1, cf2| cf1.payment_date().cmp(&cf2.payment_date()))
                .cloned(),
        }
    }
}

// Base implementation for HasCashflows for &vec<Cashflow> and &[Cashflow]
// Implement HasCashflows for &Vec<Cashflow>
impl HasCashflows for Vec<Cashflow> {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(self.iter())
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(self.iter_mut())
    }
}
