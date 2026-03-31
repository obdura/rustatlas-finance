RustAtlas
=========

[![CI](https://github.com/your-org/rustatlas/actions/workflows/rust.yml/badge.svg)](https://github.com/your-org/rustatlas/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/rustatlas.svg)](https://crates.io/crates/rustatlas)
[![docs.rs](https://docs.rs/rustatlas/badge.svg)](https://docs.rs/rustatlas)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82.0-orange.svg)](https://www.rust-lang.org)

**RustAtlas** is a high-performance quantitative finance library written in Rust (edition 2021, toolchain 1.82.0), designed for precision and speed in financial calculations. It is built around a visitor pattern for analytics, a `MarketStore` for market data, and a `SimpleModel` for resolving discount factors, forward rates, and FX rates.

Architecture Overview
---------------------

```
MarketStore ──► SimpleModel ──► Visitors (NPV, Par, Duration, Z-spread, …)
     │
     ├── IndexStore  (yield curves, Ibor/Overnight indices)
     └── ExchangeRateStore (spot rates, historical rates, forecasts)
```

- Instruments are built via fluent `Make*` constructors and hold typed cashflows.
- Visitors traverse cashflows without mutating instruments (`ConstVisit`) or with mutation (`Visit`).
- The `prelude` module re-exports everything needed for day-to-day use.

Quick Start
-----------

Add to `Cargo.toml`:

```toml
[dependencies]
rustatlas = { path = "." }
```

### Build a fixed-rate instrument

```rust
use rustatlas::prelude::*;

let start_date = Date::new(2020, 1, 1);
let end_date = start_date + Period::new(2, TimeUnit::Months);
let rate = InterestRate::new(
    0.05,
    Compounding::Compounded,
    Frequency::Annual,
    DayCounter::Actual360,
);

let instrument = MakeFixedRateInstrument::new()
    .with_start_date(start_date)
    .with_end_date(end_date)
    .with_payment_frequency(Frequency::Monthly)
    .with_rate(rate)
    .with_notional(1_000.0)
    .with_side(Side::Receive)
    .with_currency(Currency::USD)
    .equal_payments()
    .build()?;

instrument.cashflows().for_each(|cf| println!("{}", cf));
```

### Price with NPV visitor

Pricing is done through visitors. The `SimpleModel` resolves discount factors and index fixings from a `MarketStore`:

```rust
use rustatlas::prelude::*;

let market_store = create_store(); // see examples/common/common.rs
let ref_date = market_store.reference_date();

let instrument = MakeFixedRateInstrument::new()
    .with_start_date(ref_date)
    .with_end_date(ref_date + Period::new(10, TimeUnit::Years))
    .with_rate(InterestRate::new(0.05, Compounding::Simple, Frequency::Annual, DayCounter::Thirty360))
    .with_payment_frequency(Frequency::Semiannual)
    .with_side(Side::Receive)
    .with_currency(Currency::USD)
    .bullet()
    .with_discount_curve_id(Some(2))
    .with_notional(100_000.0)
    .build()?;

let model = SimpleModel::new(&market_store);
let npv = NPVConstVisitor::new(&model, true).visit(&instrument)?;
assert_ne!(npv, 0.0);
```

For more examples, see the [examples](examples) folder.

Examples
--------

| Example | Description |
|---------|-------------|
| `fixedratepricing` | Fixed rate loan: NPV, accrual, par rate — starting today, forward-starting, and already-started |
| `floatingratepricing` | Floating rate loan with index fixing, NPV, and par spread |
| `swappricing` | Vanilla IRS (fixed vs. floating): NPV and par rate, with and without spread |
| `fxforward` | Vanilla FX Forward and Non-Deliverable Forward (NDF) pricing |
| `rolloversimulation` | ALM rollover engine: constant portfolio, growth mode, and parallel NPV by date |

```bash
cargo run --example fixedratepricing
cargo run --example floatingratepricing
cargo run --example swappricing
cargo run --example fxforward
cargo run --example rolloversimulation
```

Features
--------

### Market Data & Curves

| Feature | Status |
|---------|--------|
| Ibor / Overnight indices | ✅ |
| Accrual for Ibor / Overnight (floating coupons) | ✅ |
| Flat-forward term structure | ✅ |
| Zero-rate term structure (date-based & tenor-based) | ✅ |
| Discount-factor term structure | ✅ |
| Composite term structure (spread + base, multiplicative DFs) | ✅ |
| Synthetic term structure (ratio of DF products) | ✅ |
| Tenor-based spread term structure | ✅ |
| Curve shock / parallel shift analysis | ✅ |
| Bootstrapping engine (multi-curve, multi-instrument) | ✅ |
| Fixing period adjustments | ✅ |
| Advance `MarketStore` to T+1 | ✅ |
| Curves with parametric models (Nelson-Siegel-Svensson, Vasicek, …) | 🔜 Planned |

### Cashflows & Coupons

| Feature | Status |
|---------|--------|
| Simple cashflow (disbursement / redemption) | ✅ |
| Fixed-rate coupon | ✅ |
| Floating-rate / Ibor coupon | ✅ |
| Indexed FX cashflow | ✅ |

### Financial Instruments

| Instrument | Structures | Status |
|------------|-----------|--------|
| Fixed-rate loan / deposit | Bullet, Amortizing, Zero-coupon, Equal installments, Irregular | ✅ |
| Floating-rate loan / deposit | Bullet, Amortizing, Zero-coupon, Irregular | ✅ |
| Mixed-rate (double-rate) instrument | — | ✅ |
| Fixed-rate bond | — | ✅ |
| Current account | — | ✅ |
| Time deposit | — | ✅ |
| Vanilla IRS (fixed vs. floating) | — | ✅ |
| Cross-currency swap | — | ✅ |
| Vanilla FX Forward | — | ✅ |
| Non-Deliverable Forward (NDF) | — | ✅ |
| Forward-starting instruments | — | ✅ |
| Options | — | 🔜 Planned |

### Analytics (Visitors)

| Visitor | Description | Status |
|---------|-------------|--------|
| `NPVConstVisitor` | Net present value | ✅ |
| `NPVByDateConstVisitor` | NPV bucketed by payment date | ✅ |
| `NPVByTenorConstVisitor` | NPV bucketed by tenor | ✅ |
| `ParValueVisitor` | Par rate / par spread | ✅ |
| `DurationConstVisitor` | Macaulay duration | ✅ |
| `DV01ConstVisitor` | Dollar value of 1 bp (analytical) | ✅ |
| `ZSpreadConstVisitor` | Zero spread (Z-spread) via Brent solver | ✅ |
| `YieldRateConstVisitor` | Yield-to-maturity | ✅ |
| `FixingVisitor` | Resolves forward rates into floating coupons | ✅ |
| `FixingFxVisitor` | Resolves FX rates into indexed cashflows | ✅ |
| `AccruedInterestConstVisitor` | Accrued interest | ✅ |
| `AccruedAmountConstVisitor` | Accrued amount | ✅ |
| `InterestPaymentConstVisitor` | Scheduled interest payments | ✅ |
| `OutstandingsConstVisitor` | Outstanding notional over time | ✅ |
| `RedemptionsConstVisitor` | Redemption cashflows | ✅ |
| `PlacementConstVisitor` | Placement (disbursement) cashflows | ✅ |
| `NotPayInterestConstVisitor` | Capitalised (not-yet-paid) interest | ✅ |
| `CashflowCompressorConstVisitor` | Compress cashflows by date | ✅ |
| `CashflowsAggregatorConstVisitor` | Aggregate cashflows across instruments | ✅ |

### ALM / Simulation

| Feature | Description | Status |
|---------|-------------|--------|
| `RolloverSimulationEngine` | Day-by-day portfolio rollover with configurable strategies | ✅ |
| `NPVEngine` | Parallel portfolio NPV (Rayon-backed, chunked) | ✅ |
| Growth modes | `PaidAmount`, `Annual`, `CustomGrowth` (interpolated vector) | ✅ |
| Scale factor | Rescale base redemptions before simulation | ✅ |

### Currencies & FX

| Feature | Status |
|---------|--------|
| Spot exchange rates | ✅ |
| Historical exchange rate store | ✅ |
| Forward FX via currency forecast curves | ✅ |
| FX triangulation | ✅ |
| Date-window averaging | ✅ |
| Automatic currency conversion in cashflows | ✅ |
| Load currencies from JSON | ✅ |

### Math

| Feature | Status |
|---------|--------|
| Linear interpolation | ✅ |
| Log-linear interpolation | ✅ |
| Cubic interpolation | ✅ |
| Brent root-finding solver | ✅ |
| Brent optimisation solver | ✅ |
| Newton-Raphson solver | ✅ |
| Gauss-Newton solver | ✅ |
| Forward-mode AD (`ADReal` + `Tape`) | ✅ |
| Compile-time automatic differentiation | 🔜 Planned (tracking [rust-lang#124509](https://github.com/rust-lang/rust/issues/124509)) |

### Time Utilities

| Feature | Status |
|---------|--------|
| Date arithmetic | ✅ |
| Period (tenor) arithmetic | ✅ |
| Schedule generation | ✅ |
| IMM dates | ✅ |
| Day counters: Actual/360, Actual/365, Actual/Actual, Business/252, 30/360 | ✅ |
| Calendars: NullCalendar, WeekendsOnly, Chile, USA, Brazil, Colombia, Mexico, TARGET | ✅ |
| Business day conventions | ✅ |

### Rust-specific

| Feature | Status |
|---------|--------|
| Unified error type (`AtlasError`) via `thiserror` | ✅ |
| No panics in library code (all errors propagated via `Result`) | ✅ |
| Thread-safe market data (`Arc<RwLock<…>>`) | ✅ |
| Parallel computation via Rayon | ✅ |
| Serde serialization for core types | ✅ |

### Known Issues / Tracked Work

| Issue | Status |
|-------|--------|
| Weekend fixings | ✅ Feature + unit tests |
| Grace periods in loans | ✅ Feature + unit tests |
| Automatic currency conversion | ✅ Feature + unit tests |

Dependencies
------------

| Crate | Purpose |
|-------|---------|
| `chrono` | Date parsing |
| `rayon` | Parallel iterators |
| `thiserror` | Error derivation |
| `serde` / `serde_json` | Serialization |
| `nalgebra` | Linear algebra (solvers) |
| `bumpalo` | Arena allocator (AD tape) |
| `colored` | Terminal output formatting |

Contributing
------------

Contributions to RustAtlas are welcome! If you have suggestions for improvements or have identified issues, please open an issue or submit a pull request.

License
-------

RustAtlas is released under the MIT License. Details can be found in the [LICENSE](LICENSE) file.
