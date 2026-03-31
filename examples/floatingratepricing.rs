extern crate rustatlas;

use rustatlas::{
    cashflows::side::Side,
    currencies::enums::Currency,
    instruments::constructors::makefloatingrateinstrument::MakeFloatingRateInstrument,
    models::simplemodel::SimpleModel,
    rates::{interestrate::RateDefinition, traits::HasReferenceDate},
    time::{
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

// Floating rate loan starting today.
// The FixingVisitor resolves the index rates for each coupon before pricing.
// Par spread: the spread over the index that makes NPV = 0.
fn starting_today_pricing() {
    print_title("Floating Rate Loan — starting today");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date;
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;

    // RateDefinition::default() uses Actual360 / Simple / Annual
    let rate_definition = RateDefinition::default();

    // forecast_curve_id(1) provides the index fixings; discount_curve_id(2) discounts cashflows.
    let mut instrument = MakeFloatingRateInstrument::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_payment_frequency(Frequency::Semiannual)
        .with_rate_definition(rate_definition)
        .bullet()
        .with_notional(notional)
        .with_spread(0.01)
        .with_currency(Currency::USD)
        .with_side(Side::Pay)
        .with_forecast_curve_id(Some(1))
        .with_discount_curve_id(Some(2))
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);

    // FixingVisitor must run before NPV to populate floating coupon rates
    let fixing_visitor = FixingVisitor::new(&model);
    fixing_visitor.visit(&mut instrument).unwrap();

    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&instrument).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);

    let par_visitor = ParValueConstVisitor::new(&model);
    let par_spread = par_visitor.visit(&instrument).unwrap();
    println!("Par Spread: {:.6}", par_spread);
}

// Already-started floating loan: began 3 months ago.
// The first coupon is partially accrued; its fixing rate comes from historical data.
fn already_started_pricing() {
    print_title("Floating Rate Loan — already started (-3M)");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date - Period::new(3, TimeUnit::Months);
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;

    let rate_definition = RateDefinition::default();

    let mut instrument = MakeFloatingRateInstrument::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_payment_frequency(Frequency::Semiannual)
        .bullet()
        .with_rate_definition(rate_definition)
        .with_notional(notional)
        .with_spread(0.01)
        .with_currency(Currency::USD)
        .with_side(Side::Pay)
        .with_forecast_curve_id(Some(1))
        .with_discount_curve_id(Some(2))
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);

    let fixing_visitor = FixingVisitor::new(&model);
    fixing_visitor.visit(&mut instrument).unwrap();

    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&instrument).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);
}

fn main() {
    starting_today_pricing();
    println!();
    already_started_pricing();
}
