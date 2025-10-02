#[cfg(test)]
mod tests{
    use std::collections::HashMap;

    use crate::{
        cashflows::side::Side,
        currencies::enums::Currency,
        instruments::{
            bonds::fixedratebond::FixedRateBond,
            constructors::{
                makefixedratebond::MakeFixedRateBond,
                makefixedrateinstrument::MakeFixedRateInstrument,
            },
            loandepos::{fixedrateinstrument::FixedRateInstrument, instrument::Instrument},
        },
        rates::{enums::Compounding, interestrate::InterestRate},
        time::{
            date::Date,
            daycounter::DayCounter,
            enums::{Frequency, TimeUnit},
            period::Period,
        },
        utils::errors::Result,
        visitors::{
            flowconsolidationvisitors::{
                accruedinterestconstvisitor::{
                    AccruedAmountMapAtYieldRateConstVisitor, AccruedAmountMapConstVisitor,
                },
                interestpaymentconstvisitor::{InterestPaymentMapAtYieldRateConstVisitor, InterestPaymentMapConstVisitor},
                notpayinterestconstvisitor::{NotPayInterestMapAtYieldRateConstVisitor, NotPayInterestMapConstVisitor},
                outstandingsconstvisitor::{
                    CleanPriceMapConstVisitor, DirtyPriceMapConstVisitor,
                    OutstandingMapConstVisitor,
                },
                placementconstvisitor::{
                    PlacementMapAtYieldRateConstVisitor, PlacementMapConstVisitor,
                },
                redemptionsconstvisitor::{
                    RedemptionMapAtYieldRateConstVisitor, RedemptionMapConstVisitor,
                },
            },
            traits::ConstVisit,
        },
    };

    pub fn make_one_test_fixed_instrument(side: Side) -> Result<FixedRateInstrument> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let instrument = MakeFixedRateInstrument::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(rate)
            .with_notional(1_000_000.0)
            .with_side(side)
            .with_currency(Currency::CLP)
            .bullet()
            .build()?;

        Ok(instrument)
    }

    pub fn make_one_test_fixed_bond_instrument(side: Side) -> Result<FixedRateBond> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(2, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(side)
            .with_currency(Currency::CLP)
            .bullet()
            .build()?;

        Ok(instrument)
    }

    pub fn make_test_instrument() -> Result<Vec<Instrument>> {
        let start_date_0 = Date::new(2020, 1, 1);
        let end_date_0 = start_date_0 + Period::new(5, TimeUnit::Months);
        let rate = InterestRate::new(
            0.05,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual360,
        );

        let mut instruments = Vec::new();

        for i in 0..1 {
            let start_date = start_date_0 + Period::new(i * 2, TimeUnit::Months);
            let end_date = end_date_0 + Period::new(i * 2, TimeUnit::Months);

            let tem = MakeFixedRateInstrument::new()
                .with_start_date(start_date)
                .with_end_date(end_date)
                .with_payment_frequency(Frequency::Monthly)
                .with_rate(rate)
                .with_notional(100.0)
                .with_side(Side::Receive)
                .with_currency(Currency::CLP)
                .zero()
                .build()?;
            instruments.push(Instrument::FixedRateInstrument(tem));
        }
        Ok(instruments)
    }

    #[test]
    fn test_contability_of_one_fixed_instrument() -> Result<()> {
        let instrument = vec![make_one_test_fixed_instrument(Side::Receive)?];

        let eval_date = Date::new(2019, 12, 29);

        let visitor = OutstandingMapConstVisitor::new(eval_date);
        let outstandings = visitor.visit(&instrument.as_slice())?;

        let visitors = RedemptionMapConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instrument.as_slice())?;

        let visitors = PlacementMapConstVisitor::new(eval_date);
        let placements = visitors.visit(&instrument.as_slice())?;

        let visitor = AccruedAmountMapConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instrument.as_slice())?;

        let visistors = NotPayInterestMapConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instrument.as_slice())?;

        let visitors = InterestPaymentMapConstVisitor::new(eval_date);
        let cupon_interest = visitors.visit(&instrument.as_slice())?;

        let mut date = eval_date;
        while date < eval_date + Period::new(25, TimeUnit::Months) {
            let outstandings_at_date = outstandings.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);
            let cupon_interest_at_date = cupon_interest.get(&date).unwrap_or(&0.0);

            println!("date: {}, outstandings: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}, cupon_interest: {:.7}", date, outstandings_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date, cupon_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        assert!((outstandings.get(&Date::new(2022, 1, 1)).unwrap() - 1_000_000.0).abs() < 1e-6);
        assert!((redemptions.get(&Date::new(2022, 1, 1)).unwrap() - 1_000_000.0).abs() < 1e-6);
        assert!((placements.get(&Date::new(2022, 1, 1)).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((accrued_amount.get(&Date::new(2022, 1, 1)).unwrap() - 138.9410049).abs() < 1e-6);
        assert!(
            (not_pay_interest.get(&Date::new(2022, 1, 1)).unwrap() - 25111.7866459).abs() < 1e-6
        );
        assert!((cupon_interest.get(&Date::new(2022, 1, 1)).unwrap() - 25250.7276508).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_contability_of_one_fixed_instrument_inverse() -> Result<()> {
        let instrument = vec![make_one_test_fixed_instrument(Side::Pay)?];

        let eval_date = Date::new(2019, 12, 29);

        let visitor = OutstandingMapConstVisitor::new(eval_date);
        let outstandings = visitor.visit(&instrument.as_slice())?;

        let visitors = RedemptionMapConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instrument.as_slice())?;

        let visitors = PlacementMapConstVisitor::new(eval_date);
        let placements = visitors.visit(&instrument.as_slice())?;

        let visitor = AccruedAmountMapConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instrument.as_slice())?;

        let visistors = NotPayInterestMapConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instrument.as_slice())?;

        let visitors = InterestPaymentMapConstVisitor::new(eval_date);
        let cupon_interest = visitors.visit(&instrument.as_slice())?;

        let mut date = eval_date;
        while date < eval_date + Period::new(25, TimeUnit::Months) {
            let outstandings_at_date = outstandings.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);
            let cupon_interest_at_date = cupon_interest.get(&date).unwrap_or(&0.0);
            println!("date: {}, outstandings: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}, cupon_interest: {:.7}", date, outstandings_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date, cupon_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        assert!((outstandings.get(&Date::new(2022, 1, 1)).unwrap() + 1_000_000.0).abs() < 1e-6);
        assert!((redemptions.get(&Date::new(2022, 1, 1)).unwrap() + 1_000_000.0).abs() < 1e-6);
        assert!((placements.get(&Date::new(2022, 1, 1)).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((accrued_amount.get(&Date::new(2022, 1, 1)).unwrap() + 138.9410049).abs() < 1e-6);
        assert!(
            (not_pay_interest.get(&Date::new(2022, 1, 1)).unwrap() + 25111.7866459).abs() < 1e-6
        );
        assert!((cupon_interest.get(&Date::new(2022, 1, 1)).unwrap() + 25250.7276508).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_contability_of_one_fixed_bond_at_contractual_terms() -> Result<()> {
        let instrument = vec![make_one_test_fixed_bond_instrument(Side::Receive)?];

        let eval_date = Date::new(2019, 12, 29);

        let visitor = OutstandingMapConstVisitor::new(eval_date);
        let outstandings = visitor.visit(&instrument.as_slice())?;

        let visitors = RedemptionMapConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instrument.as_slice())?;

        let visitors = PlacementMapConstVisitor::new(eval_date);
        let placements = visitors.visit(&instrument.as_slice())?;

        let visitor = AccruedAmountMapConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instrument.as_slice())?;

        let visistors = NotPayInterestMapConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instrument.as_slice())?;

        let visitors = InterestPaymentMapConstVisitor::new(eval_date);
        let cupon_interest = visitors.visit(&instrument.as_slice())?;

        let mut date = eval_date;
        while date < eval_date + Period::new(25, TimeUnit::Months) {
            let outstandings_at_date = outstandings.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);
            let cupon_interest_at_date = cupon_interest.get(&date).unwrap_or(&0.0);

            println!("date: {}, outstandings: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}, cupon_interest: {:.7}", date, outstandings_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date, cupon_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        assert!((outstandings.get(&Date::new(2022, 1, 1)).unwrap() - 1_000_000.0).abs() < 1e-6);
        assert!((redemptions.get(&Date::new(2022, 1, 1)).unwrap() - 1_000_000.0).abs() < 1e-6);
        assert!((placements.get(&Date::new(2022, 1, 1)).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((accrued_amount.get(&Date::new(2022, 1, 1)).unwrap() - 138.8888889).abs() < 1e-6);
        assert!(
            (not_pay_interest.get(&Date::new(2022, 1, 1)).unwrap() - 25416.6666667).abs() < 1e-6
        );
        assert!((cupon_interest.get(&Date::new(2022, 1, 1)).unwrap() - 25555.5555556).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_contability_of_one_fixed_bond_at_contractual_terms_inverse() -> Result<()> {
        let instrument = vec![make_one_test_fixed_bond_instrument(Side::Pay)?];

        let eval_date = Date::new(2019, 12, 29);

        let visitor = OutstandingMapConstVisitor::new(eval_date);
        let outstandings = visitor.visit(&instrument.as_slice())?;

        let visitors = RedemptionMapConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instrument.as_slice())?;

        let visitors = PlacementMapConstVisitor::new(eval_date);
        let placements = visitors.visit(&instrument.as_slice())?;

        let visitor = AccruedAmountMapConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instrument.as_slice())?;

        let visistors = NotPayInterestMapConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instrument.as_slice())?;

        let visitors = InterestPaymentMapConstVisitor::new(eval_date);
        let cupon_interest = visitors.visit(&instrument.as_slice())?;

        let mut date = eval_date;
        while date < eval_date + Period::new(25, TimeUnit::Months) {
            let outstandings_at_date = outstandings.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);
            let cupon_interest_at_date = cupon_interest.get(&date).unwrap_or(&0.0);

            println!("date: {}, outstandings: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}, cupon_interest: {:.7}", date, outstandings_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date, cupon_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        assert!((outstandings.get(&Date::new(2022, 1, 1)).unwrap() + 1_000_000.0).abs() < 1e-6);
        assert!((redemptions.get(&Date::new(2022, 1, 1)).unwrap() + 1_000_000.0).abs() < 1e-6);
        assert!((placements.get(&Date::new(2022, 1, 1)).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((accrued_amount.get(&Date::new(2022, 1, 1)).unwrap() + 138.8888889).abs() < 1e-6);
        assert!(
            (not_pay_interest.get(&Date::new(2022, 1, 1)).unwrap() + 25416.6666667).abs() < 1e-6
        );
        assert!((cupon_interest.get(&Date::new(2022, 1, 1)).unwrap() + 25555.5555556).abs() < 1e-6);

        Ok(())
    }

    #[test]
    fn test_contability_of_one_fixed_bond_at_yield_rate() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from([
            (Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0),
        ]);

        let redemptions = HashMap::from([
            (Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0),
        ]);

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Receive)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        let instrument = vec![instrument];

        let eval_date = Date::new(2019, 12, 29);

        let visitor = CleanPriceMapConstVisitor::new(eval_date);
        let clean_price = visitor.visit(&instrument.as_slice())?;

        let visitor = DirtyPriceMapConstVisitor::new(eval_date);
        let dirty_price = visitor.visit(&instrument.as_slice())?;

        let visitors = RedemptionMapAtYieldRateConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instrument.as_slice())?;

        let visitors = PlacementMapAtYieldRateConstVisitor::new(eval_date);
        let placements = visitors.visit(&instrument.as_slice())?;

        let visitor = AccruedAmountMapAtYieldRateConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instrument.as_slice())?;

        let visistors = NotPayInterestMapAtYieldRateConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instrument.as_slice())?;

        let visitors = InterestPaymentMapAtYieldRateConstVisitor::new(eval_date);
        let cupon_interest = visitors.visit(&instrument.as_slice())?;

        let evaluation_date = Date::new(2019, 12, 31);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((cupon_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 1, 1);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((cupon_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) - 1484481.36472548).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 4, 1);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) - 121.102092143148).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) - 10859.1256872891).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) - 1495461.59250491).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) - 1484481.36472548).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) - 503500.775724638).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 4, 2);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) - 160.999316138565).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) - 1988143.13976626).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) - 1987982.14045012).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);

        let evaluation_date = Date::new(2022, 1, 2);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((cupon_interest.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 7, 1);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) - 495268.142823773).abs() < 1e-6);

        let evaluation_date = Date::new(2021, 1, 1);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) - 1492713.99762634).abs() < 1e-6);

        let mut date = eval_date;
        while date < eval_date + Period::new(13, TimeUnit::Months) {
            let clean_price_at_date = clean_price.get(&date).unwrap_or(&0.0);
            let dirty_price_at_date = dirty_price.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);
            let cupon_interest_at_date = cupon_interest.get(&date).unwrap_or(&0.0);
            println!("date: {}, clean_price: {:.2}, dirty_price: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}, cupon_interest: {:.7}", date, clean_price_at_date, dirty_price_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date, cupon_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        Ok(())
    }

    #[test]
    fn test_contability_of_one_fixed_bond_at_yield_rate_inverse() -> Result<()> {
        let start_date = Date::new(2020, 1, 1);
        let end_date = start_date + Period::new(1, TimeUnit::Years);
        let cupo_rate = InterestRate::new(
            0.02,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let yield_rate = InterestRate::new(
            0.03,
            Compounding::Compounded,
            Frequency::Annual,
            DayCounter::Actual365,
        );

        let disbursements = HashMap::from([
            (Date::new(2020, 1, 1), 1_500_000.0),
            (Date::new(2020, 4, 1), 500_000.0),
        ]);

        let redemptions = HashMap::from([
            (Date::new(2020, 7, 1), 500_000.0),
            (Date::new(2021, 1, 1), 1_500_000.0),
        ]);

        let instrument = MakeFixedRateBond::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_payment_frequency(Frequency::Semiannual)
            .with_rate(cupo_rate)
            .with_yield_rate(yield_rate)
            .with_notional(1_000_000.0)
            .with_side(Side::Pay)
            .with_currency(Currency::CLP)
            .with_redemptions(redemptions)
            .with_disbursements(disbursements)
            .other()
            .build()?;

        let instrument = vec![instrument];

        let eval_date = Date::new(2019, 12, 29);

        let visitor = CleanPriceMapConstVisitor::new(eval_date);
        let clean_price = visitor.visit(&instrument.as_slice())?;

        let visitor = DirtyPriceMapConstVisitor::new(eval_date);
        let dirty_price = visitor.visit(&instrument.as_slice())?;

        let visitors = RedemptionMapAtYieldRateConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instrument.as_slice())?;

        let visitors = PlacementMapAtYieldRateConstVisitor::new(eval_date);
        let placements = visitors.visit(&instrument.as_slice())?;

        let visitor = AccruedAmountMapAtYieldRateConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instrument.as_slice())?;

        let visistors = NotPayInterestMapAtYieldRateConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instrument.as_slice())?;

        let visitors = InterestPaymentMapAtYieldRateConstVisitor::new(eval_date);
        let cupon_interest = visitors.visit(&instrument.as_slice())?;

        let evaluation_date = Date::new(2019, 12, 31);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((cupon_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 1, 1);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((cupon_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) + 1484481.36472548).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 4, 1);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) + 121.102092143148).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) + 10859.1256872891).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) + 1495461.59250491).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) + 1484481.36472548).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) + 503500.775724638).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 4, 2);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) + 160.999316138565).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) + 1988143.13976626).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) + 1987982.14045012).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);

        let evaluation_date = Date::new(2022, 1, 2);
        assert!((accrued_amount.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((not_pay_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((cupon_interest.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((dirty_price.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((clean_price.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);
        assert!((placements.get(&evaluation_date).unwrap_or(&0.0) + 0.0).abs() < 1e-6);

        let evaluation_date = Date::new(2020, 7, 1);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) + 495268.142823773).abs() < 1e-6);

        let evaluation_date = Date::new(2021, 1, 1);
        assert!((redemptions.get(&evaluation_date).unwrap_or(&0.0) + 1492713.99762634).abs() < 1e-6);

        let mut date = eval_date;
        while date < eval_date + Period::new(13, TimeUnit::Months) {
            let clean_price_at_date = clean_price.get(&date).unwrap_or(&0.0);
            let dirty_price_at_date = dirty_price.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);
            let cupon_interest_at_date = cupon_interest.get(&date).unwrap_or(&0.0);

            println!("date: {}, clean_price: {:.2}, dirty_price: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}, cupon_interest: {:.7}", date, clean_price_at_date, dirty_price_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date, cupon_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        Ok(())
    }

    #[test]
    fn test_contability_of_instruments() -> Result<()> {
        let instruments = make_test_instrument()?;
        println!("instruments: {:?}", instruments.len());
        let eval_date = Date::new(2019, 12, 28);

        let visitor = AccruedAmountMapConstVisitor::new(eval_date);
        let accrued_amount = visitor.visit(&instruments.as_slice())?;

        let visitor = OutstandingMapConstVisitor::new(eval_date);
        let outstandings = visitor.visit(&instruments.as_slice())?;

        let visitors = RedemptionMapConstVisitor::new(eval_date);
        let redemptions = visitors.visit(&instruments.as_slice())?;

        let visitors = PlacementMapConstVisitor::new(eval_date);
        let placements = visitors.visit(&instruments.as_slice())?;

        let visistors = NotPayInterestMapConstVisitor::new(eval_date);
        let not_pay_interest = visistors.visit(&instruments.as_slice())?;

        let mut date = eval_date;
        while date < eval_date + Period::new(1, TimeUnit::Years) {
            let outstandings_at_date = outstandings.get(&date).unwrap_or(&0.0);
            let redemptions_at_date = redemptions.get(&date).unwrap_or(&0.0);
            let accrued_amount_at_date = accrued_amount.get(&date).unwrap_or(&0.0);
            let placements_at_date = placements.get(&date).unwrap_or(&0.0);
            let not_pay_interest_at_date = not_pay_interest.get(&date).unwrap_or(&0.0);

            println!("date: {:?}, outstandings: {:.2}, redemptions: {:.2}, placements: {:.2}, accrued_amount: {:.7}, notPayInterest: {:.7}", date, outstandings_at_date, redemptions_at_date, placements_at_date, accrued_amount_at_date, not_pay_interest_at_date);
            date = date + Period::new(1, TimeUnit::Days);
        }

        let date = Date::new(2020, 6, 1);
        assert!((outstandings.get(&date).unwrap_or(&0.0) - 100.0).abs() < 1e-6);
        assert!((redemptions.get(&date).unwrap_or(&0.0) - 100.0).abs() < 1e-6);
        assert!((placements.get(&date).unwrap_or(&0.0) - 0.0).abs() < 1e-6);
        assert!((accrued_amount.get(&date).unwrap_or(&0.0) - 0.0138340).abs() < 1e-6);
        Ok(())
    }
}
