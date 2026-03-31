extern crate rustatlas;

use rustatlas::{
    cashflows::{
        side::Side,
        traits::{InterestAccrual, Payable},
    },
    currencies::enums::Currency,
    instruments::constructors::makefixedrateinstrument::MakeFixedRateInstrument,
    models::simplemodel::SimpleModel,
    rates::{enums::Compounding, interestrate::InterestRate, traits::HasReferenceDate},
    time::{
        date::Date,
        daycounter::DayCounter,
        enums::{Frequency, TimeUnit},
        period::Period,
    },
    visitors::{
        npvvisitors::npvconstvisitor::NPVConstVisitor,
        parvaluevisitors::traits::ParValueConstVisitor,
        traits::{ConstVisit, HasCashflows},
    },
};

mod common;
use crate::common::common::*;

// Loan starting today: NPV should be ~0 when the coupon rate equals the discount rate.
// Also shows accrual calculation and par rate (the rate that makes NPV = 0).
fn starting_today_pricing() {
    print_title("Fixed Rate Loan — starting today");
    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date;
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;
    let rate = InterestRate::new(0.05, Compounding::Simple, Frequency::Annual, DayCounter::Actual360);

    // Bullet structure: interest paid semiannually, principal returned at maturity.
    // discount_curve_id(2) refers to the discount index registered in create_store().
    let instrument = MakeFixedRateInstrument::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_rate(rate)
        .with_payment_frequency(Frequency::Semiannual)
        .with_side(Side::Receive)
        .with_currency(Currency::USD)
        .bullet()
        .with_discount_curve_id(Some(2))
        .with_notional(notional)
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);
    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&instrument).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);

    // Accrued interest for a specific period
    let start_accrual = Date::new(2021, 9, 1);
    let end_accrual = Date::new(2021, 10, 1);
    let accrued = instrument
        .cashflows()
        .fold(0.0, |acc, cf| acc + cf.accrued_amount(start_accrual, end_accrual).unwrap_or(0.0));
    println!("Accrued ({} -> {}): {:.4}", start_accrual, end_accrual, accrued);

    // Cashflows maturing on the reference date
    let maturing = instrument
        .cashflows()
        .filter(|cf| cf.payment_date() == ref_date)
        .fold(0.0, |acc, cf| acc + cf.amount().unwrap_or(0.0));
    println!("Maturing today: {:.4}", maturing);

    // Par rate: the coupon rate that makes NPV = 0 given current market curves
    let par_visitor = ParValueConstVisitor::new(&model);
    let par_rate = par_visitor.visit(&instrument).unwrap();
    println!("Par Rate: {:.6}", par_rate);
}

// Forward-starting loan: starts 6 months from today.
// Accrual before the start date is zero by definition.
fn forward_starting_pricing() {
    print_title("Fixed Rate Loan — forward starting (+6M)");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date + Period::new(6, TimeUnit::Months);
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;
    let rate = InterestRate::new(0.05, Compounding::Simple, Frequency::Annual, DayCounter::Actual360);

    let instrument = MakeFixedRateInstrument::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_rate(rate)
        .with_payment_frequency(Frequency::Semiannual)
        .with_side(Side::Receive)
        .bullet()
        .with_currency(Currency::USD)
        .with_discount_curve_id(Some(0))
        .with_notional(notional)
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);
    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&instrument).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);

    // Accrual is 0 because the loan hasn't started yet
    let start_accrual = ref_date;
    let end_accrual = ref_date + Period::new(1, TimeUnit::Months);
    let accrued = instrument
        .cashflows()
        .fold(0.0, |acc, cf| acc + cf.accrued_amount(start_accrual, end_accrual).unwrap_or(0.0));
    println!("Accrued before start (expected 0): {:.4}", accrued);
}

// Already-started loan: began 2 months ago.
// NPV reflects the remaining cashflows discounted to today.
fn already_started_pricing() {
    print_title("Fixed Rate Loan — already started (-2M)");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date - Period::new(2, TimeUnit::Months);
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;
    let rate = InterestRate::new(0.05, Compounding::Simple, Frequency::Annual, DayCounter::Actual360);

    let instrument = MakeFixedRateInstrument::new()
        .with_start_date(start_date)
        .with_end_date(end_date)
        .with_rate(rate)
        .with_payment_frequency(Frequency::Semiannual)
        .with_side(Side::Receive)
        .bullet()
        .with_currency(Currency::USD)
        .with_discount_curve_id(Some(2))
        .with_notional(notional)
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);
    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&instrument).unwrap();

    print_separator();
    println!("NPV: {:.4}", npv);

    // Accrued interest for the current partial period
    let start_accrual = ref_date - Period::new(2, TimeUnit::Months);
    let end_accrual = ref_date;
    let accrued = instrument
        .cashflows()
        .fold(0.0, |acc, cf| acc + cf.accrued_amount(start_accrual, end_accrual).unwrap_or(0.0));
    println!("Accrued since start: {:.4}", accrued);

    let par_visitor = ParValueConstVisitor::new(&model);
    let par_rate = par_visitor.visit(&instrument).unwrap();
    println!("Par Rate: {:.6}", par_rate);
}

fn main() {
    starting_today_pricing();
    println!();
    forward_starting_pricing();
    println!();
    already_started_pricing();
}
