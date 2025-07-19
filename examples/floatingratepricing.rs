extern crate rustatlas;

use rustatlas::{
    cashflows::side::Side, currencies::enums::Currency, instruments::constructors::makefloatingrateinstrument::MakeFloatingRateInstrument, models::{simplemodel::SimpleModel, traits::Model}, rates::{interestrate::RateDefinition, traits::HasReferenceDate}, time::{
        enums::{Frequency, TimeUnit},
        period::Period,
    }, visitors::{
       indexingvisitors::{fixingvisitor::FixingVisitor, indexingvisitor::IndexingVisitor}, npvvisitors::npvconstvisitor::NPVConstVisitor, parvaluevisitors::traits::ParValueConstVisitor, traits::{ConstVisit, HasCashflows, Visit}
    }
};

mod common;
use crate::common::common::*;

fn starting_today_pricing() {
    print_title("Pricing of a Floating Rate Loan starting today");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let start_date = ref_date;
    let end_date = start_date + Period::new(5, TimeUnit::Years);
    let notional = 100_000.0;

    let rate_definition = RateDefinition::default();

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

    let indexer = IndexingVisitor::new();
    let result = indexer.visit(&mut instrument);
    match result {
        Ok(_) => (),
        Err(e) => panic!("IndexingVisitor failed with error: {}", e),
    }

    let model = SimpleModel::new(&market_store);
    let data = model.gen_market_data(&indexer.request()).unwrap();

    let fixing_visitor = FixingVisitor::new(&data);
    let _ = fixing_visitor.visit(&mut instrument);

    print_table(instrument.cashflows_as_vec(), &data);

    let npv_visitor = NPVConstVisitor::new(&data, true);
    let npv = npv_visitor.visit(&instrument);

    print_separator();
    println!("NPV: {}", npv.unwrap());

    let par_visitor = ParValueConstVisitor::new(&data);
    let par_value = par_visitor.visit(&instrument).unwrap();
    println!("Par Value: {}", par_value);
}

fn already_started_pricing() {
    print_title("Pricing of a Floating Rate Loan already started -1Y");

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

    let indexer = IndexingVisitor::new();
    let result = indexer.visit(&mut instrument);
    match result {
        Ok(_) => (),
        Err(e) => panic!("IndexingVisitor failed with error: {}", e),
    }

    let model = SimpleModel::new(&market_store);
    let data = model.gen_market_data(&indexer.request()).unwrap();

    let fixing_visitor = FixingVisitor::new(&data);
    let _ = fixing_visitor.visit(&mut instrument);

    print_table(instrument.cashflows_as_vec(), &data);

    let npv_visitor = NPVConstVisitor::new(&data, true);
    let npv = npv_visitor.visit(&instrument);

    print_separator();
    println!("NPV: {}", npv.unwrap());
}

fn main() {
    starting_today_pricing();
    println!("\n");
    already_started_pricing();
}
