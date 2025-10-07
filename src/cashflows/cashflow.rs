use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use serde::{Deserialize, Serialize};

use crate::{
    cashflows::indexfxcashflow::IndexFxCashflow,
    core::{
        meta::{DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest},
        traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId, Registrable},
    },
    currencies::{enums::Currency, exchangerategeneration::ExchangeGenerationMethod},
    time::date::Date,
    utils::errors::{AtlasError, Result},
};

use super::{
    fixedratecoupon::FixedRateCoupon,
    floatingratecoupon::FloatingRateCoupon,
    side::Side,
    simplecashflow::SimpleCashflow,
    traits::{InterestAccrual, Payable, RequiresFixingRate, Scalable},
};

/// # Cashflow
/// Enum that represents a cashflow.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Cashflow {
    Redemption(SimpleCashflow),
    Disbursement(SimpleCashflow),
    FixedRateCoupon(FixedRateCoupon),
    FloatingRateCoupon(FloatingRateCoupon),
    IndexFxCashflow(IndexFxCashflow),
}

impl Cashflow {
    pub fn set_discount_curve_id(&mut self, id: usize) {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.set_discount_curve_id(id),
            Cashflow::Disbursement(cashflow) => cashflow.set_discount_curve_id(id),
            Cashflow::FixedRateCoupon(coupon) => coupon.set_discount_curve_id(id),
            Cashflow::FloatingRateCoupon(coupon) => coupon.set_discount_curve_id(id),
            Cashflow::IndexFxCashflow(coupon) => coupon.set_discount_curve_id(id),
        }
    }

    pub fn set_forecast_curve_id(&mut self, id: usize) {
        match self {
            Cashflow::FloatingRateCoupon(coupon) => coupon.set_forecast_curve_id(id),
            _ => (),
        }
    }
}

impl From<CashflowType> for String {
    fn from(cashflow_type: CashflowType) -> Self {
        match cashflow_type {
            CashflowType::Redemption => "Redemption".to_string(),
            CashflowType::Disbursement => "Disbursement".to_string(),
            CashflowType::FixedRateCoupon => "FixedRateCoupon".to_string(),
            CashflowType::FloatingRateCoupon => "FloatingRateCoupon".to_string(),
            CashflowType::IndexFxCashflow => "IndexFxCashflow".to_string(),
        }
    }
}

impl Payable for Cashflow {
    fn amount(&self) -> Result<f64> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.amount(),
            Cashflow::Disbursement(cashflow) => cashflow.amount(),
            Cashflow::FixedRateCoupon(coupon) => coupon.amount(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.amount(),
            Cashflow::IndexFxCashflow(coupon) => coupon.amount(),
        }
    }

    fn side(&self) -> Side {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.side(),
            Cashflow::Disbursement(cashflow) => cashflow.side(),
            Cashflow::FixedRateCoupon(coupon) => coupon.side(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.side(),
            Cashflow::IndexFxCashflow(coupon) => coupon.side(),
        }
    }

    fn payment_date(&self) -> Date {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.payment_date(),
            Cashflow::Disbursement(cashflow) => cashflow.payment_date(),
            Cashflow::FixedRateCoupon(coupon) => coupon.payment_date(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.payment_date(),
            Cashflow::IndexFxCashflow(coupon) => coupon.payment_date(),
        }
    }

    fn payment_currency(&self) -> Result<Currency> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.payment_currency(),
            Cashflow::Disbursement(cashflow) => cashflow.payment_currency(),
            Cashflow::FixedRateCoupon(coupon) => coupon.payment_currency(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.payment_currency(),
            Cashflow::IndexFxCashflow(coupon) => coupon.payment_currency(),
        }
    }

    fn exchange_fixing_method(&self) -> Result<&Option<ExchangeGenerationMethod>> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.exchange_fixing_method(),
            Cashflow::Disbursement(cashflow) => cashflow.exchange_fixing_method(),
            Cashflow::FixedRateCoupon(coupon) => coupon.exchange_fixing_method(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.exchange_fixing_method(),
            Cashflow::IndexFxCashflow(coupon) => coupon.exchange_fixing_method(),
        }
    }
}

impl HasCurrency for Cashflow {
    fn currency(&self) -> Result<Currency> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.currency(),
            Cashflow::Disbursement(cashflow) => cashflow.currency(),
            Cashflow::FixedRateCoupon(coupon) => coupon.currency(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.currency(),
            Cashflow::IndexFxCashflow(coupon) => coupon.currency(),
        }
    }
}

impl HasDiscountCurveId for Cashflow {
    fn discount_curve_id(&self) -> Result<usize> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.discount_curve_id(),
            Cashflow::Disbursement(cashflow) => cashflow.discount_curve_id(),
            Cashflow::FixedRateCoupon(coupon) => coupon.discount_curve_id(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.discount_curve_id(),
            Cashflow::IndexFxCashflow(coupon) => coupon.discount_curve_id(),
        }
    }
}

impl HasForecastCurveId for Cashflow {
    fn forecast_curve_id(&self) -> Result<usize> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.forecast_curve_id(),
            Cashflow::Disbursement(cashflow) => cashflow.forecast_curve_id(),
            Cashflow::FixedRateCoupon(coupon) => coupon.forecast_curve_id(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.forecast_curve_id(),
            Cashflow::IndexFxCashflow(coupon) => coupon.forecast_curve_id(),
        }
    }
}

impl Registrable for Cashflow {
    fn set_id(&mut self, id: usize) {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.set_id(id),
            Cashflow::Disbursement(cashflow) => cashflow.set_id(id),
            Cashflow::FixedRateCoupon(coupon) => coupon.set_id(id),
            Cashflow::FloatingRateCoupon(coupon) => coupon.set_id(id),
            Cashflow::IndexFxCashflow(coupon) => coupon.set_id(id),
        }
    }

    fn id(&self) -> Result<usize> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.id(),
            Cashflow::Disbursement(cashflow) => cashflow.id(),
            Cashflow::FixedRateCoupon(coupon) => coupon.id(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.id(),
            Cashflow::IndexFxCashflow(coupon) => coupon.id(),
        }
    }

    fn df_request(&self) -> Result<Option<DiscountFactorRequest>> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.df_request(),
            Cashflow::Disbursement(cashflow) => cashflow.df_request(),
            Cashflow::FixedRateCoupon(coupon) => coupon.df_request(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.df_request(),
            Cashflow::IndexFxCashflow(coupon) => coupon.df_request(),
        }
    }

    fn fwd_request(&self) -> Result<Option<ForwardRateRequest>> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.fwd_request(),
            Cashflow::Disbursement(cashflow) => cashflow.fwd_request(),
            Cashflow::FixedRateCoupon(coupon) => coupon.fwd_request(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.fwd_request(),
            Cashflow::IndexFxCashflow(coupon) => coupon.fwd_request(),
        }
    }

    fn fx_request(&self) -> Result<Option<ExchangeRateRequest>> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.fx_request(),
            Cashflow::Disbursement(cashflow) => cashflow.fx_request(),
            Cashflow::FixedRateCoupon(coupon) => coupon.fx_request(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.fx_request(),
            Cashflow::IndexFxCashflow(coupon) => coupon.fx_request(),
        }
    }

    fn fx_fwd_request(&self) -> Result<Option<ExchangeRateRequest>> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.fx_fwd_request(),
            Cashflow::Disbursement(cashflow) => cashflow.fx_fwd_request(),
            Cashflow::FixedRateCoupon(coupon) => coupon.fx_fwd_request(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.fx_fwd_request(),
            Cashflow::IndexFxCashflow(coupon) => coupon.fx_fwd_request(),
        }
    }

    fn fx_fixing_request(&self) -> Result<Option<ExchangeRateRequest>> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.fx_fixing_request(),
            Cashflow::Disbursement(cashflow) => cashflow.fx_fixing_request(),
            Cashflow::FixedRateCoupon(coupon) => coupon.fx_fixing_request(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.fx_fixing_request(),
            Cashflow::IndexFxCashflow(coupon) => coupon.fx_fixing_request(),
        }
    }
}

impl InterestAccrual for Cashflow {
    fn accrual_end_date(&self) -> Result<Date> {
        match self {
            Cashflow::FixedRateCoupon(coupon) => coupon.accrual_end_date(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.accrual_end_date(),
            Cashflow::Disbursement(_) | Cashflow::Redemption(_) | Cashflow::IndexFxCashflow(_) => {
                Err(AtlasError::InvalidValueErr(
                    "Disbursement, Redemption and IndexFxCashflow cashflows do not have an accrual end date"
                        .to_string(),
                ))
            }

        }
    }

    fn accrual_start_date(&self) -> Result<Date> {
        match self {
            Cashflow::FixedRateCoupon(coupon) => coupon.accrual_start_date(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.accrual_start_date(),
            Cashflow::Disbursement(_) | Cashflow::Redemption(_) | Cashflow::IndexFxCashflow(_) => {
                Err(AtlasError::InvalidValueErr(
                    "Disbursement, Redemption and IndexFxCashflow cashflows do not have an accrual start date"
                        .to_string(),
                ))
            }
        }
    }

    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<f64> {
        match self {
            Cashflow::FixedRateCoupon(coupon) => coupon.accrued_amount(start_date, end_date),
            Cashflow::FloatingRateCoupon(coupon) => coupon.accrued_amount(start_date, end_date),
            _ => Ok(0.0),
        }
    }

    fn accrued_amount_map(&self) -> Result<BTreeMap<Date, f64>> {
        match self {
            Cashflow::FixedRateCoupon(coupon) => coupon.accrued_amount_map(),
            Cashflow::FloatingRateCoupon(coupon) => coupon.accrued_amount_map(),
            _ => Ok(BTreeMap::new()),
        }
    }
}

impl RequiresFixingRate for Cashflow {
    fn set_fixing_rate(&mut self, fixing_rate: f64) {
        match self {
            Cashflow::FloatingRateCoupon(coupon) => coupon.set_fixing_rate(fixing_rate),
            _ => (),
        }
    }

    fn fixing_start_date(&self) -> Result<Option<Date>> {
        match self {
            Cashflow::FloatingRateCoupon(coupon) => coupon.fixing_start_date(),
            _ => Ok(None),
        }
    }

    fn fixing_end_date(&self) -> Result<Option<Date>> {
        match self {
            Cashflow::FloatingRateCoupon(coupon) => coupon.fixing_end_date(),
            _ => Ok(None),
        }
    }
}

impl Scalable for Cashflow {
    fn scale(&mut self, factor: f64) -> Result<()> {
        match self {
            Cashflow::Redemption(cashflow) => cashflow.scale(factor),
            Cashflow::Disbursement(cashflow) => cashflow.scale(factor),
            Cashflow::FixedRateCoupon(coupon) => coupon.scale(factor),
            Cashflow::FloatingRateCoupon(coupon) => coupon.scale(factor),
            Cashflow::IndexFxCashflow(coupon) => coupon.scale(factor),
        }
    }
}

use colored::*;
impl Display for Cashflow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let amount = self.amount().unwrap_or(0.0);
        let currency = self.currency().map_err(|_| std::fmt::Error)?;
        let payment_currency = self.payment_currency().map_err(|_| std::fmt::Error)?;
        match self {
            Cashflow::Redemption(cashflow) => write!(
                f,
                "{} {} {} {} {} {} {} {} {}",
                "Pay date:".bold().yellow(),
                cashflow.payment_date().to_string().cyan(),
                "type:".bold().yellow(),
                "redemp".cyan(),
                " - ".white().bold(),
                "amount: ".bold().yellow(),
                format!("{:.2} {} pay in {}", amount, currency, payment_currency).cyan(),
                "side:".bold().yellow(),
                format!("{:?}", cashflow.side()).cyan()
            ),
            Cashflow::Disbursement(cashflow) => write!(
                f,
                "{} {} {} {} {} {} {} {} {}",
                "Pay date:".bold().yellow(),
                cashflow.payment_date().to_string().cyan(),
                "type:".bold().yellow(),
                "disb".cyan(),
                " - ".white().bold(),
                "amount:".bold().yellow(),
                format!("{:.2} {} pay in {}", amount, currency, payment_currency).cyan(),
                "side:".bold().yellow(),
                format!("{:?}", cashflow.side()).cyan()
            ),
            Cashflow::FixedRateCoupon(coupon) => write!(
                f,
                "{} {} {} {} {} {} {} {} {} {} {} {} {}",
                "Pay date:".bold().yellow(),
                coupon.payment_date().to_string().cyan(),
                "type:".bold().yellow(),
                "fix cpn".cyan(),
                " - ".white().bold(),
                "amount:".bold().yellow(),
                format!("{:.2} {} pay in {}", amount, currency, payment_currency).cyan(),
                "side:".bold().yellow(),
                format!("{}", coupon.side()).cyan(),
                "rate:".bold().yellow(),
                format!("{:.5}", coupon.rate().rate()).cyan(),
                "acc ".bold().yellow(),
                format!(
                    "{} - {}",
                    coupon.accrual_start_date().unwrap_or(Date::new(1970, 1, 1)),
                    coupon.accrual_end_date().unwrap_or(Date::new(1970, 1, 1))
                )
                .cyan()
            ),
            Cashflow::FloatingRateCoupon(coupon) => write!(
                f,
                "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                "Pay date:".bold().yellow(),
                coupon.payment_date().to_string().cyan(),
                "type:".bold().yellow(),
                "float cpn".cyan(),
                " - ".white().bold(),
                "amount:".bold().yellow(),
                format!("{:.2} {} pay in {}", amount, currency, payment_currency).cyan(),
                "side:".bold().yellow(),
                format!("{}", coupon.side()).cyan(),
                "fix rate:".bold().yellow(),
                format!("{:.5}", coupon.fixing_rate().unwrap_or(0.0)).cyan(),
                "acc ".bold().yellow(),
                format!(
                    "{} - {}",
                    coupon.accrual_start_date().unwrap_or(Date::new(1970, 1, 1)),
                    coupon.accrual_end_date().unwrap_or(Date::new(1970, 1, 1))
                )
                .cyan(),
                "fix ".bold().yellow(),
                format!(
                    "{} - {}",
                    coupon
                        .fixing_start_date()
                        .unwrap_or(None)
                        .unwrap_or(Date::new(1970, 1, 1)),
                    coupon
                        .fixing_end_date()
                        .unwrap_or(None)
                        .unwrap_or(Date::new(1970, 1, 1))
                )
                .cyan()
            ),
            Cashflow::IndexFxCashflow(cashflow) => write!(
                f,
                "{} {} {} {} {} {} {} {} {}",
                "Pay date:".bold().yellow(),
                cashflow.payment_date().to_string().cyan(),
                "type:".bold().yellow(),
                "indexfx".cyan(),
                " - ".white().bold(),
                "amount: ".bold().yellow(),
                format!("{:.2} {} pay in {}", amount, currency, payment_currency).cyan(),
                "side:".bold().yellow(),
                format!("{:?}", cashflow.side()).cyan()
            ),
        }
    }
}

/// # CashflowType
/// Enum that represents the type of a cashflow.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CashflowType {
    Redemption,
    Disbursement,
    FixedRateCoupon,
    FloatingRateCoupon,
    IndexFxCashflow,
}

#[cfg(test)]
mod tests {
    use crate::cashflows::side::Side;
    use super::*;

    #[test]
    fn serialization_test() {
        let cashflow = Cashflow::Redemption(SimpleCashflow::new(
            Date::new(2024, 1, 1),
            Currency::USD,
            Side::Receive,
        ));
        let serialized = serde_json::to_string(&cashflow).unwrap();
        println!("{}", serialized);

        let deserialized: Cashflow = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cashflow, deserialized);
    }

    #[test]
    fn display_redemption_test() {
        let cashflow = Cashflow::Redemption(SimpleCashflow::new(
            Date::new(2024, 1, 1),
            Currency::USD,
            Side::Pay,
        ));
        let display = format!("{}", cashflow);
        assert!(display.contains("Pay date:"));
        assert!(display.contains("redemp"));
        assert!(display.contains("USD"));
    }

    #[test]
    fn scale_redemption_test() {
        let mut cashflow = Cashflow::Redemption(SimpleCashflow::new(
            Date::new(2024, 1, 1),
            Currency::USD,
            Side::Receive,
        ));
        assert!(cashflow.scale(2.0).is_ok());
    }

    #[test]
    fn set_and_get_discount_curve_id() {
        let mut cashflow = Cashflow::Redemption(SimpleCashflow::new(
            Date::new(2024, 1, 1),
            Currency::USD,
            Side::Receive,
        ));
        cashflow.set_discount_curve_id(42);
        assert_eq!(cashflow.discount_curve_id().unwrap(), 42);
    }

    #[test]
    fn cashflow_type_to_string() {
        assert_eq!(String::from(CashflowType::Redemption), "Redemption");
        assert_eq!(String::from(CashflowType::Disbursement), "Disbursement");
        assert_eq!(
            String::from(CashflowType::FixedRateCoupon),
            "FixedRateCoupon"
        );
        assert_eq!(
            String::from(CashflowType::FloatingRateCoupon),
            "FloatingRateCoupon"
        );
    }

    #[test]
    fn interest_accrual_error_on_redemption() {
        let cashflow = Cashflow::Redemption(SimpleCashflow::new(
            Date::new(2024, 1, 1),
            Currency::USD,
            Side::Receive,
        ));
        assert!(cashflow.accrual_start_date().is_err());
        assert!(cashflow.accrual_end_date().is_err());
    }

    #[test]
    fn requires_fixing_rate_none_on_redemption() {
        let cashflow = Cashflow::Redemption(SimpleCashflow::new(
            Date::new(2024, 1, 1),
            Currency::USD,
            Side::Receive,
        ));
        assert!(matches!(cashflow.fixing_start_date(), Ok(None)));
        assert!(matches!(cashflow.fixing_end_date(), Ok(None)));
    }
}
