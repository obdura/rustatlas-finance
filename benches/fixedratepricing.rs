extern crate rustatlas;

use std::collections::BTreeMap;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use rustatlas::{
    alm::npvengine::NPVEngine,
    cashflows::side::Side,
    currencies::enums::Currency,
    instruments::{
        constructors::makefixedrateinstrument::MakeFixedRateInstrument,
        loandepos::instrument::Instrument,
        traits::{RateType, Structure},
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
    visitors::{npvvisitors::npvconstvisitor::NPVConstVisitor, traits::ConstVisit},
    alm::{
        positiongenerator::RolloverStrategy,
        rolloversimulationengine::RolloverSimulationEngine,
    },
};

mod common;
use crate::common::common::*;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// --- helpers -----------------------------------------------------------------

fn make_homogeneous_portfolio(n: usize) -> Vec<Instrument> {
    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();
    let start_date = ref_date;
    let end_date = start_date + Period::new(10, TimeUnit::Years);
    let rate = InterestRate::new(0.05, Compounding::Simple, Frequency::Annual, DayCounter::Thirty360);

    (0..n)
        .into_par_iter()
        .map(|_| {
            Instrument::FixedRateInstrument(
                MakeFixedRateInstrument::new()
                    .with_start_date(start_date)
                    .with_end_date(end_date)
                    .with_rate(rate)
                    .with_payment_frequency(Frequency::Semiannual)
                    .with_side(Side::Receive)
                    .with_currency(Currency::USD)
                    .bullet()
                    .with_discount_curve_id(Some(2))
                    .with_notional(100_000.0)
                    .build()
                    .unwrap(),
            )
        })
        .collect()
}

fn make_heterogeneous_portfolio(n: usize) -> Vec<Instrument> {
    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();
    MockMaker::generate_random_instruments(n, ref_date)
}

fn base_redemptions(ref_date: rustatlas::time::date::Date) -> BTreeMap<rustatlas::time::date::Date, f64> {
    (0..12)
        .map(|i| (ref_date + Period::new(i, TimeUnit::Months), 1_000.0))
        .collect()
}

fn rollover_strategies() -> Vec<RolloverStrategy> {
    vec![
        RolloverStrategy::new(0.5, Structure::Bullet, Frequency::Semiannual, Period::new(1, TimeUnit::Years), Side::Receive, RateType::Fixed, RateDefinition::default(), 0, None),
        RolloverStrategy::new(0.5, Structure::Bullet, Frequency::Semiannual, Period::new(2, TimeUnit::Years), Side::Receive, RateType::Fixed, RateDefinition::default(), 0, None),
    ]
}

// --- benchmarks --------------------------------------------------------------

// Measures pure instrument construction cost (no pricing).
// Useful to understand how much of the total time is build vs. eval.
fn bench_portfolio_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("portfolio_build");
    for n in [1_000, 10_000, 50_000] {
        group.bench_with_input(BenchmarkId::new("homogeneous", n), &n, |b, &n| {
            b.iter(|| make_homogeneous_portfolio(n));
        });
        group.bench_with_input(BenchmarkId::new("heterogeneous", n), &n, |b, &n| {
            b.iter(|| make_heterogeneous_portfolio(n));
        });
    }
    group.finish();
}

// Measures NPV of a pre-built portfolio.
// The portfolio is built once outside the loop so criterion only times the pricing.
fn bench_npv_pricing(c: &mut Criterion) {
    let mut group = c.benchmark_group("npv_pricing");
    let market_store = create_store().unwrap();
    let model = SimpleModel::new(&market_store);
    let npv_visitor = NPVConstVisitor::new(&model, true);

    for n in [1_000, 10_000, 50_000] {
        let instruments = make_homogeneous_portfolio(n);
        group.bench_with_input(BenchmarkId::new("homogeneous", n), &instruments, |b, insts| {
            b.iter(|| {
                insts.iter().for_each(|inst| {
                    npv_visitor.visit(inst).unwrap();
                });
            });
        });

        let instruments = make_heterogeneous_portfolio(n);
        group.bench_with_input(BenchmarkId::new("heterogeneous", n), &instruments, |b, insts| {
            b.iter(|| {
                insts.iter().for_each(|inst| {
                    npv_visitor.visit(inst).unwrap();
                });
            });
        });
    }
    group.finish();
}

// Measures NPVEngine (parallel) with different chunk sizes.
// Helps find the optimal chunk_size for a given portfolio size.
fn bench_npv_engine_chunk_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("npv_engine_chunk_size");
    let market_store = create_store().unwrap();

    for chunk_size in [100, 500, 1_000, 5_000] {
        let mut instruments = make_homogeneous_portfolio(10_000);
        group.bench_with_input(
            BenchmarkId::new("chunk", chunk_size),
            &chunk_size,
            |b, &chunk_size| {
                b.iter(|| {
                    NPVEngine::new(&mut instruments, &market_store)
                        .with_chunk_size(chunk_size)
                        .run()
                        .unwrap();
                });
            },
        );
    }
    group.finish();
}

// Measures the full rollover simulation loop for different horizons.
// This is the most expensive path: advance_to_date + par rate + fixing per new position.
fn bench_rollover_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rollover_simulation");
    let market_store = create_store().unwrap();
    let ref_date = market_store.reference_date();
    let redemptions = base_redemptions(ref_date);
    let strategies = rollover_strategies();

    for years in [1, 2, 3] {
        let horizon = Period::new(years, TimeUnit::Years);
        group.bench_with_input(
            BenchmarkId::new("horizon_years", years),
            &horizon,
            |b, &horizon| {
                b.iter(|| {
                    RolloverSimulationEngine::new(
                        &market_store,
                        redemptions.clone(),
                        Currency::USD,
                        horizon,
                    )
                    .run(strategies.clone())
                    .unwrap();
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_portfolio_build,
    bench_npv_pricing,
    bench_npv_engine_chunk_sizes,
    bench_rollover_simulation,
);
criterion_main!(benches);
