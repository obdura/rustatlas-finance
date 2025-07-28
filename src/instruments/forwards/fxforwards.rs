use crate::{
    cashflows::{cashflow::Cashflow, simplecashflow::SimpleCashflow},
    core::traits::{HasCurrency, HasDiscountCurveId},
    currencies::enums::Currency,
    visitors::traits::HasCashflows,
    utils::errors::Result,
};

/// # forwards
/// A financial forward. Contains a stream of cashflows. Instruments have one or more legs.
#[derive(Debug, Clone)]
pub struct FxForward {
    id: Option<String>,
    pay_currency: Currency,
    receive_currency: Currency,
    pay_cashflows: Vec<Cashflow>,
    receive_cashflows: Vec<Cashflow>,
    pay_discount_curve_id: Option<usize>,
    receive_discount_curve_id: Option<usize>,
    mtm: Option<f64>,
}

impl FxForward {
    pub fn new(
        pay_cashflow: SimpleCashflow,
        receive_cashflow: SimpleCashflow,
    ) -> Result<FxForward> {
        let pay_cashflows = vec![Cashflow::Disbursement(pay_cashflow)];
        let pay_discount_curve_id = pay_cashflow.discount_curve_id().ok();
        let pay_currency = pay_cashflow.currency()?;
        let receive_cashflows = vec![Cashflow::Redemption(receive_cashflow)];
        let receive_discount_curve_id = receive_cashflow.discount_curve_id().ok();
        let receive_currency = receive_cashflow.currency()?;
        Ok(FxForward {
            id: None,
            pay_currency,
            receive_currency,
            pay_cashflows,
            receive_cashflows,
            pay_discount_curve_id,
            receive_discount_curve_id,
            mtm: None,
        })
    }

    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn pay_currency(&self) -> Currency {
        self.pay_currency
    }

    pub fn receive_currency(&self) -> Currency {
        self.receive_currency
    }

    pub fn pay_cashflows(&self) -> &Vec<Cashflow> {
        &self.pay_cashflows
    }

    pub fn receive_cashflows(&self) -> &Vec<Cashflow> {
        &self.receive_cashflows
    }

    pub fn pay_discount_curve_id(&self) -> Option<usize> {
        self.pay_discount_curve_id
    }

    pub fn receive_discount_curve_id(&self) -> Option<usize> {
        self.receive_discount_curve_id
    }

    pub fn mtm(&self) -> Option<f64> {
        self.mtm
    }

    pub fn mut_pay_cashflows(&mut self) -> &mut Vec<Cashflow> {
        &mut self.pay_cashflows
    }

    pub fn mut_receive_cashflows(&mut self) -> &mut Vec<Cashflow> {
        &mut self.receive_cashflows
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn with_pay_discount_curve_id(mut self, id: usize) -> Self {
        self.pay_discount_curve_id = Some(id);
        self.mut_pay_cashflows()
            .iter_mut()
            .for_each(|cf| cf.set_discount_curve_id(id));
        self
    }

    pub fn with_receive_discount_curve_id(mut self, id: usize) -> Self {
        self.receive_discount_curve_id = Some(id);
        self.mut_receive_cashflows()
            .iter_mut()
            .for_each(|cf| cf.set_discount_curve_id(id));
        self
    }

    pub fn set_pay_discount_curve_id(&mut self, id: usize) {
        self.pay_discount_curve_id = Some(id);
        self.mut_pay_cashflows()
            .iter_mut()
            .for_each(|cf| cf.set_discount_curve_id(id));
    }

    pub fn set_receive_discount_curve_id(&mut self, id: usize) {
        self.receive_discount_curve_id = Some(id);
        self.mut_receive_cashflows()
            .iter_mut()
            .for_each(|cf| cf.set_discount_curve_id(id));
    }

    pub fn with_mtm(mut self, mtm: f64) -> Self {
        self.mtm = Some(mtm);
        self
    }
}

impl HasCashflows for FxForward {
    fn cashflows(&self) -> Box<dyn Iterator<Item = &Cashflow> + '_> {
        Box::new(
            self.pay_cashflows
                .iter()
                .chain(self.receive_cashflows.iter()),
        )
    }

    fn mut_cashflows(&mut self) -> Box<dyn Iterator<Item = &mut Cashflow> + '_> {
        Box::new(
            self.pay_cashflows
                .iter_mut()
                .chain(self.receive_cashflows.iter_mut()),
        )
    }
}

use colored::*;
impl std::fmt::Display for FxForward {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}\n",
            "FxForward  id: ".white().bold(),
            self.id().unwrap_or("no set!".to_string()).cyan()
        )?;
        write!(
            f,
            "\t{} {}\n",
            "-> Pay Currency: ".magenta().bold(),
            self.pay_currency().to_string().cyan()
        )?;
        write!(
            f,
            "\t{} {}\n",
            "-> Recive Currency: ".magenta().bold(),
            self.receive_currency().to_string().cyan()
        )?;
        write!(f, "\t{}\n", "-> Cashflows: ".magenta().bold())?;
        self.cashflows()
            .for_each(|cf| write!(f, "\t\t{} {}\n", "-> ".white().bold(), cf).unwrap());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cashflows::side::Side, time::date::Date, utils::errors::Result};

    #[test]
    fn test_fxforward_creation() -> Result<()> {
        let pay_date = Date::new(2021, 1, 1);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(100.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(100.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?;
        assert_eq!(fx_forward.pay_currency(), pay_currency);
        assert_eq!(fx_forward.receive_currency(), receive_currency);
        Ok(())
    }

    #[test]
    fn test_fxforward_discount_factor() -> Result<()> {
        let pay_date = Date::new(2021, 1, 1);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(100.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(100.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?
            .with_pay_discount_curve_id(0)
            .with_receive_discount_curve_id(1);

        let _ = fx_forward
            .pay_cashflows()
            .iter()
            .try_for_each(|cf| -> Result<()> {
                assert!(cf.discount_curve_id().unwrap() == 0);
                Ok(())
            });

        let _ = fx_forward
            .receive_cashflows()
            .iter()
            .try_for_each(|cf| -> Result<()> {
                assert!(cf.discount_curve_id().unwrap() == 1);
                Ok(())
            });

        Ok(())
    }

    #[test]
    fn test_fxforward_mtm() -> Result<()> {
        let pay_date = Date::new(2021, 1, 1);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(100.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(100.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?.with_mtm(0.5);
        assert_eq!(fx_forward.mtm().unwrap(), 0.5);
        Ok(())
    }

    #[test]
    fn test_fxforward_set_id() -> Result<()> {
        let pay_date = Date::new(2021, 1, 1);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(100.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(100.0);
        let mut fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?;
        fx_forward.set_id("test".to_string());
        assert_eq!(fx_forward.id().unwrap(), "test".to_string());
        Ok(())
    }

    #[test]
    fn test_fxforward_with_id() -> Result<()> {
        let pay_date = Date::new(2021, 1, 1);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(50.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(50.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?
            .with_id("forward123".to_string());
        assert_eq!(fx_forward.id().unwrap(), "forward123".to_string());
        Ok(())
    }

    #[test]
    fn test_fxforward_set_pay_discount_curve_id() -> Result<()> {
        let pay_date = Date::new(2022, 2, 2);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(200.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(200.0);
        let mut fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?;
        fx_forward.set_pay_discount_curve_id(5);
        assert_eq!(fx_forward.pay_discount_curve_id().unwrap(), 5);
        for cf in fx_forward.pay_cashflows() {
            assert_eq!(cf.discount_curve_id().unwrap(), 5);
        }
        Ok(())
    }

    #[test]
    fn test_fxforward_set_receive_discount_curve_id() -> Result<()> {
        let pay_date = Date::new(2023, 3, 3);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(300.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(300.0);
        let mut fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?;
        fx_forward.set_receive_discount_curve_id(7);
        assert_eq!(fx_forward.receive_discount_curve_id().unwrap(), 7);
        for cf in fx_forward.receive_cashflows() {
            assert_eq!(cf.discount_curve_id().unwrap(), 7);
        }
        Ok(())
    }

    #[test]
    fn test_fxforward_display_trait() -> Result<()> {
        let pay_date = Date::new(2024, 4, 4);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(400.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(400.0);
        let fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?
            .with_id("display_test".to_string())
            .with_mtm(123.45);
        let display_str = format!("{}", fx_forward);
        assert!(display_str.contains("FxForward  id:"));
        assert!(display_str.contains("display_test"));
        assert!(display_str.contains("Pay Currency"));
        assert!(display_str.contains("Recive Currency"));
        assert!(display_str.contains("Cashflows"));
        Ok(())
    }

    #[test]
    fn test_fxforward_mut_cashflows() -> Result<()> {
        let pay_date = Date::new(2025, 5, 5);
        let pay_currency = Currency::USD;
        let receive_currency = Currency::CLP;
        let pay_cashflow =
            SimpleCashflow::new(pay_date, pay_currency, Side::Pay).with_amount(500.0);
        let receive_cashflow =
            SimpleCashflow::new(pay_date, receive_currency, Side::Receive).with_amount(500.0);
        let mut fx_forward = FxForward::new(pay_cashflow, receive_cashflow)?;
        fx_forward
            .mut_cashflows()
            .for_each(|cf| cf.set_discount_curve_id(99));
        for cf in fx_forward.pay_cashflows().iter().chain(fx_forward.receive_cashflows().iter()) {
            assert_eq!(cf.discount_curve_id().unwrap(), 99);
        }
        Ok(())
    }
}
