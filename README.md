# RustAtlas

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Edition](https://img.shields.io/badge/Rust-2021-blueviolet.svg)](https://www.rust-lang.org/)
[![Minimum Rust Version](https://img.shields.io/badge/MSRV-1.82.0-blue.svg)](https://www.rust-lang.org/)
[![GitHub](https://img.shields.io/badge/GitHub-Poss9368/rustatlas--finance-black?logo=github)](https://github.com/Poss9368/rustatlas-finance)

**RustAtlas** is a high-performance quantitative finance library written in Rust, designed for precision and speed in fixed-income pricing, FX derivatives, and ALM simulation. Built on a visitor pattern architecture with type-safe cashflows and pluggable market models.

**Status**: Production-ready for fixed-income instruments and ALM simulation. Derivatives pricing actively maintained.

### Key Highlights

- 🚀 **Type-safe**: Leverages Rust's type system for instrument and cashflow safety
- ⚡ **High-performance**: Parallel computation via Rayon, forward-mode AD for sensitivities
- 🏗️ **Extensible**: Visitor pattern + trait-based design for custom analytics
- 📊 **Comprehensive**: 20+ pricing visitors, multi-curve bootstrapping, ALM simulation
- 🌍 **Multi-currency**: Automatic FX conversion, triangulation, date-window averaging
- 🔒 **No panics**: All fallible operations return `Result<T>` via unified error type

## Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Examples](#examples)
- [Features](#features)
- [Installation](#installation)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [License](#license)

## Architecture

```
MarketStore ──► SimpleModel ──► Visitors (NPV, Par, Duration, Z-spread, …)
     │
     ├── IndexStore  (yield curves, Ibor/Overnight indices)
     └── ExchangeRateStore (spot rates, historical rates, forecasts)
```

- Instruments are built via fluent `Make*` constructors and hold typed cashflows.
- Visitors traverse cashflows without mutating instruments (`ConstVisit`) or with mutation (`Visit`).
- The `prelude` module re-exports everything needed for day-to-day use.

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rustatlas = "1.0"
```

Or for development (local path):

```toml
[dependencies]
rustatlas = { path = "../path/to/rustatlas" }
```

**Documentation**: [docs.rs/rustatlas](https://docs.rs/rustatlas) | **Source**: [github.com/Poss9368/rustatlas-finance](https://github.com/Poss9368/rustatlas-finance)

### Build a Fixed-Rate Instrument

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

### Price with NPV Visitor

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

## Examples

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

## Features

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

## Installation

Requirements:
- Rust 1.82.0 or later
- Cargo (comes with Rust)

Clone and build:

```bash
git clone https://github.com/Poss9368/rustatlas-finance.git
cd rustatlas-finance
cargo build --release
```

Run tests:

```bash
cargo test --all-features
```

Run examples:

```bash
cargo run --release --example fixedratepricing
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `chrono` | Date parsing |
| `rayon` | Parallel iterators |
| `thiserror` | Error derivation |
| `serde` / `serde_json` | Serialization |
| `nalgebra` | Linear algebra (solvers) |
| `bumpalo` | Arena allocator (AD tape) |
| `colored` | Terminal output formatting |

## Contributing

Contributions are welcome! Please follow these guidelines:

1. **Fork** the repository
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Commit** with clear messages: `git commit -m 'Add feature X'`
4. **Run tests**: `cargo test --all-features`
5. **Run formatter**: `cargo fmt`
6. **Run linter**: `cargo clippy -- -D warnings`
7. **Push** to your fork and open a **Pull Request**

For major changes, please open an issue first to discuss what you'd like to change.

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## Roadmap

### Near-term (v1.x)
- [ ] Parametric yield curve models (Nelson-Siegel-Svensson, Vasicek)
- [ ] Option pricing (Black-Scholes, Binomial)
- [ ] Enhanced benchmarking suite
- [ ] WebAssembly (WASM) build

### Medium-term (v2.x)
- [ ] Monte Carlo simulation engine
- [ ] Local volatility & stochastic vol models
- [ ] Multi-threaded curve bootstrapping
- [ ] REST API service wrapper

### Long-term
- [ ] Compile-time automatic differentiation (tracking [rust-lang#124509](https://github.com/rust-lang/rust/issues/124509))
- [ ] GPU-accelerated computations
- [ ] Real-time data connectors (Bloomberg, Reuters)

## License

RustAtlas is released under the MIT License. See [LICENSE](LICENSE) for details.

---

**Questions?** Open an [issue](https://github.com/Poss9368/rustatlas-finance/issues) or check the [examples](examples/) folder for more guidance.
