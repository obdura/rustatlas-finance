extern crate rustatlas;

use std::sync::{Arc, RwLock};

use rustatlas::{
    core::marketstore::MarketStore,
    currencies::enums::Currency,
    instruments::constructors::{
        makenondeliverablefxforward::MakeNonDeliverableFxForward,
        makevanillafxforward::MakeVanillaFxForward,
    },
    models::simplemodel::SimpleModel,
    rates::{
        interestrate::RateDefinition,
        interestrateindex::iborindex::IborIndex,
        traits::HasReferenceDate,
        yieldtermstructure::flatforwardtermstructure::FlatForwardTermStructure,
    },
    time::{date::Date, enums::TimeUnit, period::Period},
    visitors::{npvvisitors::npvconstvisitor::NPVConstVisitor, traits::ConstVisit},
};

mod common;
use crate::common::common::*;

// Builds a market store with discount curves for USD, EUR and CLP,
// spot FX rates, and currency curve mappings needed for forward FX projection.
//
// Index layout:
//   0 -> USD discount (3%)
//   1 -> EUR discount (2%)
//   2 -> CLP discount (5%)
//
// FX spot: USD/EUR = 0.92, USD/CLP = 950
fn create_fx_store() -> MarketStore {
    let ref_date = Date::new(2021, 9, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);

    let usd_curve = Arc::new(FlatForwardTermStructure::new(ref_date, 0.03, RateDefinition::default()));
    let eur_curve = Arc::new(FlatForwardTermStructure::new(ref_date, 0.02, RateDefinition::default()));
    let clp_curve = Arc::new(FlatForwardTermStructure::new(ref_date, 0.05, RateDefinition::default()));

    store.mut_index_store()
        .add_index(0, Arc::new(RwLock::new(IborIndex::new(ref_date).with_term_structure(usd_curve)))).unwrap();
    store.mut_index_store()
        .add_index(1, Arc::new(RwLock::new(IborIndex::new(ref_date).with_term_structure(eur_curve)))).unwrap();
    store.mut_index_store()
        .add_index(2, Arc::new(RwLock::new(IborIndex::new(ref_date).with_term_structure(clp_curve)))).unwrap();

    // Currency curve mappings: required by SimpleModel to project forward FX rates for NDFs
    store.mut_index_store().add_currency_curve(Currency::USD, 0);
    store.mut_index_store().add_currency_curve(Currency::EUR, 1);
    store.mut_index_store().add_currency_curve(Currency::CLP, 2);

    // Spot FX rates
    store.mut_exchange_rate_store().add_exchange_rate(Currency::USD, Currency::EUR, 0.92);
    store.mut_exchange_rate_store().add_exchange_rate(Currency::USD, Currency::CLP, 950.0);

    store
}

// Vanilla FX Forward: physical delivery of two currencies at maturity.
// Pay USD 1,000,000 / Receive EUR 920,000 in 1 year.
// NPV is the present value of the net position using each currency's discount curve.
fn vanilla_fx_forward() {
    print_title("Vanilla FX Forward — USD/EUR physical delivery");

    let market_store = create_fx_store();
    let ref_date = market_store.reference_date();
    let maturity = ref_date + Period::new(1, TimeUnit::Years);

    // pay_discount_curve_id(0) discounts the USD leg; receive_discount_curve_id(1) discounts EUR.
    let forward = MakeVanillaFxForward::new()
        .with_negotiation_date(ref_date)
        .with_pay_date_pay_side(maturity)
        .with_pay_date_receive_side(maturity)
        .with_pay_currency(Currency::USD)
        .with_receive_currency(Currency::EUR)
        .with_pay_nominal(1_000_000.0)
        .with_receive_nominal(920_000.0)
        .with_pay_discount_curve_id(0)
        .with_receive_discount_curve_id(1)
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);
    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&forward).unwrap();

    print_separator();
    println!("Maturity: {}", maturity);
    println!("Pay USD: 1,000,000 | Receive EUR: 920,000");
    println!("NPV (in instrument currency): {:.4}", npv);
}

// Non-Deliverable Forward (NDF): no physical exchange.
// The net difference is settled in the compensation currency (USD) at maturity.
// The CLP leg is converted at the projected FX rate on the settlement date.
fn non_deliverable_fx_forward() {
    print_title("Non-Deliverable FX Forward (NDF) — CLP/USD settled in USD");

    let market_store = create_fx_store();
    let ref_date = market_store.reference_date();
    let maturity = ref_date + Period::new(6, TimeUnit::Months);

    // Both legs settle in USD (compensation_currency).
    // The CLP leg amount is converted at the projected FX rate on the settlement date.
    let forward = MakeNonDeliverableFxForward::new()
        .with_negotiation_date(ref_date)
        .with_pay_date_pay_side(maturity)
        .with_pay_date_receive_side(maturity)
        .with_pay_side_currency(Currency::USD)
        .with_receive_side_currency(Currency::CLP)
        .with_compensation_currency(Currency::USD)
        .with_pay_nominal(1_000_000.0)
        .with_receive_nominal(950_000_000.0) // ~950 CLP per USD
        .with_pay_discount_curve_id(0)
        .with_receive_discount_curve_id(2)
        .build()
        .unwrap();

    let model = SimpleModel::new(&market_store);
    let npv_visitor = NPVConstVisitor::new(&model, true);
    let npv = npv_visitor.visit(&forward).unwrap();

    print_separator();
    println!("Maturity: {}", maturity);
    println!("Pay USD: 1,000,000 | Receive CLP: 950,000,000");
    println!("NPV (settled in USD): {:.4}", npv);
}

fn main() {
    vanilla_fx_forward();
    println!();
    non_deliverable_fx_forward();
}
