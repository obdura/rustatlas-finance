use std::iter;

use crate::{
    cashflows::{cashflow::Cashflow, side::Side, simplecashflow::SimpleCashflow},
    core::traits::HasCurrency,
    currencies::enums::Currency,
    utils::errors::Result,
    visitors::traits::HasCashflows,
};

#[derive(Debug, Clone)]
pub struct ForwardLeg {
    currency: Currency,
    pay_currency: Currency,
    side: Side,
    discount_curve_id: Option<usize>,
    cashflow: Cashflow,
}


impl ForwardLeg {
    pub fn new(
        currency: Currency,
        pay_currency: Currency,
        side: Side,
        cashflow: SimpleCashflow,
    ) -> ForwardLeg {
        let cashflow = Cashflow::Disbursement(cashflow);
        ForwardLeg {
            currency,
            pay_currency,
            side,
            discount_curve_id: None,
            cashflow,
        }
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn pay_currency(&self) -> Currency {
        self.pay_currency
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn discount_curve_id(&self) -> Option<usize> {
        self.discount_curve_id
    }
}

impl HasCurrency for ForwardLeg {
    fn currency(&self) -> Result<Currency> {
        Ok(self.currency)
    }
}

impl HasCashflows for ForwardLeg {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(iter::once(&self.cashflow))
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(iter::once(&mut self.cashflow))
    }
}
