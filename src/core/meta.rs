
use crate::currencies::enums::Currency;
use crate::currencies::exchangerategeneration::{ExchangeGenerationMethod, SingleDate};
use crate::rates::enums::Compounding;
use crate::time::date::Date;
use crate::time::enums::Frequency;

/// # ExchangeRateRequest
/// Meta data for an exchange rate. Holds the first currency, the second currency and the reference
/// date required to fetch the exchange rate.
///
/// ## Parameters
/// * `first_currency` - The first currency of the exchange rate.
/// * `second_currency` - The second currency of the exchange rate.
/// * `reference_date` - The reference date of the exchange rate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExchangeRateRequest {
    first_currency: Currency,
    second_currency: Option<Currency>,
    generation_method: Option<ExchangeGenerationMethod>,
}

impl ExchangeRateRequest {
    pub fn new_with_method(
        first_currency: Currency,
        second_currency: Option<Currency>,
        generation_method: Option<ExchangeGenerationMethod>,
    ) -> ExchangeRateRequest {
        ExchangeRateRequest {
            first_currency,
            second_currency,
            generation_method,
        }
    }

    pub fn new(
        first_currency: Currency,
        second_currency: Option<Currency>,
        reference_date: Option<Date>,
    ) -> ExchangeRateRequest {
        let method = match reference_date {
            Some(date) => Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(date))),
            None => None,
        };
        ExchangeRateRequest::new_with_method(first_currency, second_currency, method)
    }

    pub fn first_currency(&self) -> Currency {
        self.first_currency
    }

    pub fn second_currency(&self) -> Option<Currency> {
        self.second_currency
    }

    pub fn generation_method(&self) -> &Option<ExchangeGenerationMethod> {
        &self.generation_method
    }
}

/// # DiscountFactorRequest
/// Meta data for a discount factor. Holds the discount curve id and the reference date required to
/// fetch the discount factor.
///
/// ## Parameters
/// * `discount_curve_id` - The discount curve id of the discount factor.
/// * `date` - The reference date of the discount factor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiscountFactorRequest {
    provider_id: usize,
    date: Date,
    discount_currency: Option<Currency>,
}

impl DiscountFactorRequest {
    pub fn new(provider_id: usize, date: Date) -> DiscountFactorRequest {
        DiscountFactorRequest {
            provider_id,
            date,
            discount_currency: None,
        }
    }

    pub fn new_with_discount_currency(
        provider_id: usize,
        date: Date,
        discount_currency: Currency,
    ) -> DiscountFactorRequest {
        DiscountFactorRequest {
            provider_id,
            date,
            discount_currency: Some(discount_currency),
        }
    }

    pub fn set_discount_currency(&mut self, discount_currency: Currency) -> &mut Self {
        self.discount_currency = Some(discount_currency);
        self
    }

    pub fn provider_id(&self) -> usize {
        self.provider_id
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn discount_currency(&self) -> Option<Currency> {
        self.discount_currency
    }
}

/// # ForwardRateRequest
/// Meta data for a forward rate. Holds the forward curve id and the start and end dates required
/// to fetch the forward rate.
///
/// ## Parameters
/// * `provider_id` - The forward curve id of the forward rate.
/// * `start_date` - The start date of the forward rate.
/// * `end_date` - The end date of the forward rate.
/// * `compounding` - The compounding of the forward rate.
/// * `frequency` - The frequency of the forward rate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForwardRateRequest {
    provider_id: usize,
    start_date: Date,
    end_date: Date,
    compounding: Compounding,
    frequency: Frequency,
}

impl ForwardRateRequest {
    pub fn new(
        provider_id: usize,
        start_date: Date,
        end_date: Date,
        compounding: Compounding,
        frequency: Frequency,
    ) -> ForwardRateRequest {
        ForwardRateRequest {
            provider_id,
            start_date,
            end_date,
            compounding,
            frequency,
        }
    }

    pub fn provider_id(&self) -> usize {
        self.provider_id
    }

    pub fn start_date(&self) -> Date {
        self.start_date
    }

    pub fn end_date(&self) -> Date {
        self.end_date
    }

    pub fn compounding(&self) -> Compounding {
        self.compounding
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    fn sample_date() -> Date {
        Date::new(2024, 6, 1)
    }

    #[test]
    fn test_exchange_rate_request_new_and_accessors() {
        let req = ExchangeRateRequest::new(Currency::USD, Some(Currency::EUR), Some(sample_date()));
        assert_eq!(req.first_currency(), Currency::USD);
        assert_eq!(req.second_currency(), Some(Currency::EUR));
        assert_eq!(
            req.generation_method(),
            &Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(
                sample_date()
            )))
        );
    }

    #[test]
    fn test_exchange_rate_request_new_with_method_and_accessors() {
        let method = ExchangeGenerationMethod::SingleDate(SingleDate::new(sample_date()));
        let req =
            ExchangeRateRequest::new_with_method(Currency::USD, Some(Currency::EUR), Some(method));
        assert_eq!(req.first_currency(), Currency::USD);
        assert_eq!(req.second_currency(), Some(Currency::EUR));
        assert_eq!(
            req.generation_method(),
            &Some(ExchangeGenerationMethod::SingleDate(SingleDate::new(
                sample_date()
            )))
        );
    }

    #[test]
    fn test_discount_factor_request_new_and_accessors() {
        let req = DiscountFactorRequest::new(42, sample_date());
        assert_eq!(req.provider_id(), 42);
        assert_eq!(req.date(), sample_date());
        assert_eq!(req.discount_currency(), None);

        let req2 =
            DiscountFactorRequest::new_with_discount_currency(7, sample_date(), Currency::JPY);
        assert_eq!(req2.provider_id(), 7);
        assert_eq!(req2.discount_currency(), Some(Currency::JPY));
    }

    #[test]
    fn test_discount_factor_request_set_discount_currency() {
        let mut req = DiscountFactorRequest::new(1, sample_date());
        req.set_discount_currency(Currency::GBP);
        assert_eq!(req.discount_currency(), Some(Currency::GBP));
    }

    #[test]
    fn test_forward_rate_request_new_and_accessors() {
        let req = ForwardRateRequest::new(
            99,
            sample_date(),
            sample_date(),
            Compounding::Simple,
            Frequency::Annual,
        );
        assert_eq!(req.provider_id(), 99);
        assert_eq!(req.start_date(), sample_date());
        assert_eq!(req.end_date(), sample_date());
        assert_eq!(req.compounding(), Compounding::Simple);
        assert_eq!(req.frequency(), Frequency::Annual);
    }
}
