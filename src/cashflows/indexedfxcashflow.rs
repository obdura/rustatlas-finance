use crate::{
    cashflows::{
        side::Side,
        simplecashflow::SimpleCashflow,
        traits::{Expires, Payable, Scalable},
    },
    core::{
        meta::{ExchangeRateRequest, MarketRequest},
        traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId, Registrable},
    },
    currencies::{enums::Currency, exchangerategeneration::ExchangeGenerationMethod},
    time::date::Date,
    utils::errors::{AtlasError, Result},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexedFxCashflow {
    reference_notional: f64,
    reference_currency: Currency,
    currency: Currency,
    fixing_fx: Option<f64>,
    indexed_fx_method: Option<ExchangeGenerationMethod>,
    cashflow: SimpleCashflow,
}

impl IndexedFxCashflow {
    pub fn new(
        reference_notional: f64,
        reference_currency: Currency,
        currency: Currency,
        payment_date: Date,
        side: Side,
    ) -> IndexedFxCashflow {
        IndexedFxCashflow {
            reference_notional,
            reference_currency,
            currency,
            fixing_fx: None,
            indexed_fx_method: None,
            cashflow: SimpleCashflow::new(payment_date, currency, side),
        }
    }

    pub fn reference_notional(&self) -> f64 {
        self.reference_notional
    }

    pub fn reference_currency(&self) -> Currency {
        self.reference_currency
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn fixing_fx(&self) -> Option<f64> {
        self.fixing_fx
    }

    pub fn indexed_fx_method(&self) -> Option<&ExchangeGenerationMethod> {
        self.indexed_fx_method.as_ref()
    }

    pub fn cashflow(&self) -> &SimpleCashflow {
        &self.cashflow
    }

    pub fn cashflow_mut(&mut self) -> &mut SimpleCashflow {
        &mut self.cashflow
    }

    pub fn with_indexed_fixing_method(
        mut self,
        method: ExchangeGenerationMethod,
    ) -> IndexedFxCashflow {
        self.indexed_fx_method = Some(method);
        self
    }

    pub fn set_indexed_fixing_method(&mut self, method: ExchangeGenerationMethod) {
        self.indexed_fx_method = Some(method);
    }

    pub fn with_discount_curve_id(mut self, id: usize) -> IndexedFxCashflow {
        self.cashflow.set_discount_curve_id(id);
        self
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        self.cashflow.set_discount_curve_id(id);
    }

    pub fn with_exchange_fixing_method(
        mut self,
        method: ExchangeGenerationMethod,
    ) -> IndexedFxCashflow {
        self.cashflow_mut().set_exchange_fixing_method(method);
        self
    }

    pub fn set_exchange_fixing_method(&mut self, method: ExchangeGenerationMethod) {
        self.cashflow.set_exchange_fixing_method(method);
    }
}

impl Payable for IndexedFxCashflow {
    fn amount(&self) -> Result<f64> {
        self.cashflow.amount()
    }
    fn side(&self) -> Side {
        self.cashflow.side()
    }
    fn payment_date(&self) -> Date {
        self.cashflow.payment_date()
    }
    fn payment_currency(&self) -> Result<Currency> {
        self.cashflow.payment_currency()
    }
    fn exchange_fixing_method(&self) -> Result<&Option<ExchangeGenerationMethod>> {
        self.cashflow.exchange_fixing_method()
    }
}

impl HasCurrency for IndexedFxCashflow {
    fn currency(&self) -> Result<Currency> {
        self.cashflow.currency()
    }
}

impl HasDiscountCurveId for IndexedFxCashflow {
    fn discount_curve_id(&self) -> Result<usize> {
        self.cashflow.discount_curve_id()
    }
}

impl HasForecastCurveId for IndexedFxCashflow {
    fn forecast_curve_id(&self) -> Result<usize> {
        return Err(AtlasError::InvalidValueErr(
            "No forecast curve id for fixed rate cashflow".to_string(),
        ));
    }
}

impl Registrable for IndexedFxCashflow {
    fn id(&self) -> Result<usize> {
        self.cashflow.id()
    }

    fn set_id(&mut self, id: usize) {
        self.cashflow.set_id(id);
    }

    fn df_request(&self) -> Result<Option<crate::core::meta::DiscountFactorRequest>> {
        self.cashflow.df_request()
    }

    fn fwd_request(&self) -> Result<Option<crate::core::meta::ForwardRateRequest>> {
        self.cashflow.fwd_request()
    }

    fn fx_request(&self) -> Result<Option<ExchangeRateRequest>> {
        self.cashflow.fx_request()
    }

    fn fx_fwd_request(&self) -> Result<Option<ExchangeRateRequest>> {
        self.cashflow.fx_fwd_request()
    }

    fn fx_fixing_request(&self) -> Result<Option<ExchangeRateRequest>> {
        let fx_fixing_request = ExchangeRateRequest::new_with_method(
            self.reference_currency,
            Some(self.currency),
            self.exchange_fixing_method()?.clone(),
        );
        Ok(Some(fx_fixing_request))
    }

    fn market_request(&self) -> Result<MarketRequest> {
        let fx_fixing_request = self.fx_fixing_request()?;
        let mut request = self.cashflow.market_request()?;
        request.set_fx_fixing(fx_fixing_request);
        return Ok(request);
    }
}

impl Expires for IndexedFxCashflow {
    fn is_expired(&self, date: Date) -> bool {
        self.cashflow.payment_date() < date
    }
}

impl Scalable for IndexedFxCashflow {
    fn scale(&mut self, factor: f64) -> Result<()> {
        let notional = self.reference_notional();
        self.reference_notional = notional * factor;
        self.cashflow.scale(factor)?;
        Ok(())
    }
}
