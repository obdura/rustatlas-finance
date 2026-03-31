extern crate rustatlas;

use rustatlas::{
    cashflows::side::Side,
    currencies::enums::Currency,
    instruments::{
        constructors::{
            makefixedrateleg::MakeFixedRateLeg, makefloatingrateleg::MakeFloatingRateLeg,
        },
        swaps::vanillairsswap::VanillaIRSSwap,
    },
    models::simplemodel::SimpleModel,
    rates::{
        enums::Compounding,
        interestrate::{InterestRate, RateDefinition},
        traits::HasReferenceDate,
    },
    time::{
        daycounter::DayCounter,
        enums::{Frequency, TimeUnit},
        period::Period,
    },
    visitors::{
        fixingvisitor::fixingvisitor::FixingVisitor,
        npvvisitors::npvconstvisitor::NPVConstVisitor,
        parvaluevisitors::traits::ParValueConstVisitor,
        traits::{ConstVisit, Visit},
    },
};

mod common;
use crate::common::common::*;

// Vanilla IRS: receive fixed 5%, pay floating (index 0) + 0 spread.
// The par rate is the fixed rate that makes the swap NPV = 0.
fn vanilla_irs_pricing() {
    print_title("Vanilla IRS — receive fixed / pay floating");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date;
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 1_000_000.0;

    let rate_definition = RateDefinition::new(
        DayCounter::Actual360,
        Compounding::Simple,
        Frequency::Annual,
    );

    // Fixed leg: receive 5% annually, discounted with curve 2
    let fixed_leg = MakeFixedRateLeg::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_notional(notional)
        .with_payment_frequency(Frequency::Semiannual)
        .with_rate(InterestRate::from_rate_definition(0.05, rate_definition))
        .with_side(Side::Receive)
        .with_currency(Currency::USD)
        .with_discount_curve_id(Some(2))
        .bullet()
        .build()
        .unwrap();

    // Floating leg: pay index 0 (ibor) quarterly, discounted with curve 2
    let floating_leg = MakeFloatingRateLeg::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_payment_frequency(Frequency::Quarterly)
        .with_spread(0.0)
        .with_rate_definition(rate_definition)
        .with_side(Side::Pay)
        .with_currency(Currency::USD)
        .with_discount_curve_id(Some(2))
        .with_forecast_curve_id(Some(0))
        .with_notional(notional)
        .bullet()
        .build()
        .unwrap();

    let mut swap = VanillaIRSSwap::new(fixed_leg, floating_leg).unwrap();

    let model = SimpleModel::new(&market_store);

    // Resolve floating coupon rates before pricing
    let fixing_visitor = FixingVisitor::new(&model);
    fixing_visitor.visit(&mut swap).unwrap();

    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&swap).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);

    // Par rate: the fixed rate that makes the swap NPV = 0 against the current floating leg
    let par_visitor = ParValueConstVisitor::new(&model);
    let par_rate = par_visitor.visit(&swap).unwrap();
    println!("Par Rate (fixed leg): {:.6}", par_rate);
}

// Same swap but with a non-zero spread on the floating leg.
// The par rate will shift to compensate for the spread.
fn irs_with_spread() {
    print_title("Vanilla IRS — floating leg with spread");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date;
    let end_date = start_date + Period::new(3, TimeUnit::Years);
    let notional = 500_000.0;

    let rate_definition = RateDefinition::new(
        DayCounter::Actual360,
        Compounding::Simple,
        Frequency::Annual,
    );

    let fixed_leg = MakeFixedRateLeg::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_notional(notional)
        .with_payment_frequency(Frequency::Annual)
        .with_rate(InterestRate::from_rate_definition(0.04, rate_definition))
        .with_side(Side::Receive)
        .with_currency(Currency::USD)
        .with_discount_curve_id(Some(2))
        .bullet()
        .build()
        .unwrap();

    // 100bps spread over the index
    let floating_leg = MakeFloatingRateLeg::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_payment_frequency(Frequency::Semiannual)
        .with_spread(0.01)
        .with_rate_definition(rate_definition)
        .with_side(Side::Pay)
        .with_currency(Currency::USD)
        .with_discount_curve_id(Some(2))
        .with_forecast_curve_id(Some(0))
        .with_notional(notional)
        .bullet()
        .build()
        .unwrap();

    let mut swap = VanillaIRSSwap::new(fixed_leg, floating_leg).unwrap();

    let model = SimpleModel::new(&market_store);
    let fixing_visitor = FixingVisitor::new(&model);
    fixing_visitor.visit(&mut swap).unwrap();

    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&swap).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);

    let par_visitor = ParValueConstVisitor::new(&model);
    let par_rate = par_visitor.visit(&swap).unwrap();
    println!("Par Rate (fixed leg): {:.6}", par_rate);
}

fn main() {
    vanilla_irs_pricing();
    println!();
    irs_with_spread();
}
