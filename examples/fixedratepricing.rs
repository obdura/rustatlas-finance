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
        npvvisitors::npvconstvisitor::NPVConstVisitor, parvaluevisitors::traits::ParValueConstVisitor, traits::{ConstVisit, HasCashflows}
    },
};

mod common;
use crate::common::common::*;

fn starting_today_pricing() {
    print_title("Pricing of a Fixed Rate Loan starting today");
    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date;
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;
    let rate = InterestRate::new(
        0.05,
        Compounding::Simple,
        Frequency::Annual,
        DayCounter::Actual360,
    );

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


    let ref_date = market_store.reference_date();
    let model = SimpleModel::new(&market_store);


    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&instrument);

    print_separator();
    println!("NPV: {}", npv.unwrap());

    let start_accrual = Date::new(2024, 9, 1);
    let end_accrual = Date::new(2024, 10, 1);
    let accrued_amount = instrument.cashflows().fold(0.0, |acc, cf| {
        acc + cf.accrued_amount(start_accrual, end_accrual).unwrap()
    });

    println!(
        "Accrued Amount between {} and {}: {}",
        start_accrual, end_accrual, accrued_amount
    );

    let maturing_amount = instrument.cashflows().fold(0.0, |acc, cf| {
        if cf.payment_date() == ref_date {
            acc + cf.amount().unwrap()
        } else {
            acc
        }
    });

    println!(
        "Maturing Amount between {} and {}: {}",
        start_accrual, end_accrual, maturing_amount
    );

    let par_visitor = ParValueConstVisitor::new(&model);
    let par_value = par_visitor.visit(&instrument).unwrap();
    println!("Par Value: {}", par_value);
}

fn forward_starting_pricing() {
    print_title("Pricing of a Fixed Rate Loan starting +2Y");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date + Period::new(6, TimeUnit::Months);
    let end_date = start_date + Period::new(5, TimeUnit::Years);

    let notional = 100_000.0;
    let rate = InterestRate::new(
        0.05,
        Compounding::Simple,
        Frequency::Annual,
        DayCounter::Actual360,
    );

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
    let npv = npv_visitor.visit(&instrument);

    print_separator();
    println!("NPV: {}", npv.unwrap());

    let start_accrual = Date::new(2024, 9, 1);
    let end_accrual = Date::new(2024, 10, 1);
    let accrued_amount = instrument.cashflows().fold(0.0, |acc, cf| {
        acc + cf.accrued_amount(start_accrual, end_accrual).unwrap()
    });
    println!(
        "Accrued Amount between {} and {}: {}",
        start_accrual, end_accrual, accrued_amount
    );
}

fn already_started_pricing() {
    print_title("Pricing of a Fixed Rate Loan starting +2Y");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date - Period::new(2, TimeUnit::Months);
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;
    let rate = InterestRate::new(
        0.05,
        Compounding::Simple,
        Frequency::Annual,
        DayCounter::Actual360,
    );

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
    let npv = npv_visitor.visit(&instrument);

    print_separator();
    println!("NPV: {}", npv.unwrap());

    let start_accrual = Date::new(2024, 9, 1);
    let end_accrual = Date::new(2024, 10, 1);
    let accrued_amount = instrument.cashflows().fold(0.0, |acc, cf| {
        acc + cf.accrued_amount(start_accrual, end_accrual).unwrap()
    });
    println!(
        "Accrued Amount between {} and {}: {}",
        start_accrual, end_accrual, accrued_amount
    );

    let par_visitor = ParValueConstVisitor::new(&model);
    let par_value = par_visitor.visit(&instrument).unwrap();
    println!("Par Value: {}", par_value);
}

fn main() {
    starting_today_pricing();
    println!("\n");
    forward_starting_pricing();
    println!("\n");
    already_started_pricing();
}
