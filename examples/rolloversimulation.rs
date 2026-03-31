extern crate rustatlas;

use std::collections::BTreeMap;

use rustatlas::{
    alm::{
        npvengine::NPVEngine,
        positiongenerator::RolloverStrategy,
        rolloversimulationengine::{GrowthMode, RolloverSimulationEngine},
    },
    cashflows::{cashflow::Cashflow, side::Side, traits::Payable},
    currencies::enums::Currency,
    instruments::{loandepos::instrument::Instrument, traits::{RateType, Structure}},
    rates::{interestrate::RateDefinition, traits::HasReferenceDate},
    time::{
        date::Date,
        enums::{Frequency, TimeUnit},
        period::Period,
    },
    visitors::traits::HasCashflows,
};

mod common;
use crate::common::common::*;

// Computes the total outstanding balance of a portfolio at a given evaluation date.
// Outstanding = sum of disbursements - sum of redemptions paid up to eval_date.
fn outstanding_at(instruments: &[Instrument], eval_date: Date) -> f64 {
    instruments.iter().map(|inst| {
        inst.cashflows().fold(0.0, |acc, cf| match cf {
            Cashflow::Disbursement(f) if f.payment_date() <= eval_date => {
                acc + f.amount().unwrap_or(0.0) * f.side().sign()
            }
            Cashflow::Redemption(f) if f.payment_date() <= eval_date => {
                acc + f.amount().unwrap_or(0.0) * f.side().sign()
            }
            _ => acc,
        })
    }).sum()
}

// Basic rollover: maturing positions are reinvested 1:1 using a fixed mix of tenors.
// The portfolio outstanding stays constant over time.
fn constant_portfolio() {
    print_title("Rollover Simulation — constant portfolio");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    // Base redemptions represent the existing portfolio's maturity schedule
    let base_redemptions: BTreeMap<Date, f64> = [
        (ref_date,                                        500.0),
        (ref_date + Period::new(1, TimeUnit::Months),     500.0),
        (ref_date + Period::new(2, TimeUnit::Months),     500.0),
        (ref_date + Period::new(3, TimeUnit::Months),     500.0),
    ].iter().cloned().collect();

    let horizon = Period::new(5, TimeUnit::Years);

    let engine = RolloverSimulationEngine::new(
        &market_store,
        base_redemptions,
        Currency::USD,
        horizon,
    );

    // 50% reinvested in 1Y bullet, 50% in 2Y bullet — both at par rate
    let strategies = vec![
        RolloverStrategy::new(
            0.5,
            Structure::Bullet,
            Frequency::Semiannual,
            Period::new(1, TimeUnit::Years),
            Side::Receive,
            RateType::Fixed,
            RateDefinition::default(),
            0,
            None,
        ),
        RolloverStrategy::new(
            0.5,
            Structure::Bullet,
            Frequency::Semiannual,
            Period::new(2, TimeUnit::Years),
            Side::Receive,
            RateType::Fixed,
            RateDefinition::default(),
            0,
            None,
        ),
    ];

    let simulated = engine.run(strategies).unwrap();

    print_separator();
    println!("Simulated instruments: {}", simulated.len());

    // Outstanding should remain ~2000 throughout the horizon
    for years in [1, 2, 3] {
        let eval = ref_date + Period::new(years, TimeUnit::Years);
        let outstanding = outstanding_at(&simulated, eval);
        println!("Outstanding at +{}Y: {:.2}", years, outstanding.abs());
    }
}

// Growth rollover: each maturing position is reinvested with a 10% annual growth rate.
// The portfolio grows linearly over time.
fn growing_portfolio() {
    print_title("Rollover Simulation — annual growth 10%");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let base_redemptions: BTreeMap<Date, f64> = [
        (ref_date,                                        500.0),
        (ref_date + Period::new(1, TimeUnit::Months),     500.0),
        (ref_date + Period::new(2, TimeUnit::Months),     500.0),
        (ref_date + Period::new(3, TimeUnit::Months),     500.0),
    ].iter().cloned().collect();

    let horizon = Period::new(3, TimeUnit::Years);

    // GrowthMode::Annual grows the total outstanding linearly at the given rate
    let engine = RolloverSimulationEngine::new(
        &market_store,
        base_redemptions,
        Currency::USD,
        horizon,
    )
    .with_growth_mode(GrowthMode::Annual)
    .with_growth_rate(0.10);

    let strategies = vec![
        RolloverStrategy::new(
            0.5,
            Structure::Bullet,
            Frequency::Semiannual,
            Period::new(1, TimeUnit::Years),
            Side::Receive,
            RateType::Fixed,
            RateDefinition::default(),
            0,
            None,
        ),
        RolloverStrategy::new(
            0.5,
            Structure::Bullet,
            Frequency::Semiannual,
            Period::new(2, TimeUnit::Years),
            Side::Receive,
            RateType::Fixed,
            RateDefinition::default(),
            0,
            None,
        ),
    ];

    let simulated = engine.run(strategies).unwrap();

    print_separator();
    println!("Simulated instruments: {}", simulated.len());

    for years in [1, 2, 3] {
        let eval = ref_date + Period::new(years, TimeUnit::Years);
        let outstanding = outstanding_at(&simulated, eval);
        println!("Outstanding at +{}Y: {:.2}", years, outstanding.abs());
    }
}

// NPV engine: computes the portfolio NPV broken down by date using parallel processing.
fn portfolio_npv() {
    print_title("Portfolio NPV — parallel NPVEngine");

    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();

    let base_redemptions: BTreeMap<Date, f64> = [
        (ref_date,                                        500.0),
        (ref_date + Period::new(1, TimeUnit::Months),     500.0),
        (ref_date + Period::new(2, TimeUnit::Months),     500.0),
        (ref_date + Period::new(3, TimeUnit::Months),     500.0),
    ].iter().cloned().collect();

    let engine = RolloverSimulationEngine::new(
        &market_store,
        base_redemptions,
        Currency::USD,
        Period::new(3, TimeUnit::Years),
    );

    let strategies = vec![RolloverStrategy::new(
        1.0,
        Structure::Bullet,
        Frequency::Annual,
        Period::new(2, TimeUnit::Years),
        Side::Receive,
        RateType::Fixed,
        RateDefinition::default(),
        0,
        None,
    )];

    let mut simulated = engine.run(strategies).unwrap();

    // NPVEngine runs fixing + NPV-by-date in parallel chunks
    let mut npv_engine = NPVEngine::new(&mut simulated, &market_store);
    let npv_by_date = npv_engine.run().unwrap();

    print_separator();
    println!("NPV by date (first 5 entries):");
    for (date, npv) in npv_by_date.iter().take(5) {
        println!("  {}: {:.4}", date, npv);
    }
    println!("  ... ({} total dates)", npv_by_date.len());
}

fn main() {
    constant_portfolio();
    println!();
    growing_portfolio();
    println!();
    portfolio_npv();
}
