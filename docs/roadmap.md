# Roadmap

## Phase 0 — bootstrap

- [x] Repository architecture.
- [x] Numerical semantics document.
- [x] Dependency-free Rust reference oracle.
- [ ] Wire Oreslang package/build metadata once the package manifest convention is selected.
- [ ] Add Oreslang scalar kernels matching the oracle.
- [ ] Add cross-language golden fixtures.

## Phase 1 — market/statistics foundation

- [ ] Columnar quote/trade/bar batches.
- [ ] Ring/segmented buffers with bounded immutable views.
- [ ] Simple/log returns.
- [ ] SMA/EMA/VWAP.
- [ ] Rolling variance/stddev.
- [ ] Covariance/correlation matrices.
- [ ] Drawdown, Sharpe, Sortino, beta/alpha.
- [ ] Histogram/quantile primitives.

## Phase 2 — actor trading pipeline

- [ ] `MarketDataActor` ingestion contract.
- [ ] `StrategyActor` state/lifecycle contract.
- [ ] `RiskActor` pre/post-trade checks.
- [ ] `ExecutionActor` order routing.
- [ ] Deterministic simulation clock.
- [ ] Ordered event journal + replay.
- [ ] Backtest fill/latency models.

## Phase 3 — pricing and simulation

- [ ] Black-Scholes/Black-76 batch pricing.
- [ ] Greeks.
- [ ] Discount curves / interpolation.
- [ ] Fixed-income cash-flow batches.
- [ ] GBM and generic Monte Carlo path kernels.
- [ ] VaR/CVaR scenario aggregation.

## Phase 4 — order books and optimization

- [ ] L1/L2/L3 columnar order-book representations.
- [ ] Book update/reconstruction kernels.
- [ ] Matching-engine simulation.
- [ ] TWAP/VWAP/POV execution models.
- [ ] Mean-variance portfolio optimization.
- [ ] Constraints + transaction-cost models.
- [ ] Integrate external solver/MIP libraries without moving solver semantics into Oreslang core.

## Phase 5 — heterogeneous HPC

- [ ] Kernel descriptors with explicit read/write sets.
- [ ] Chunked CPU task execution.
- [ ] SIMD implementations.
- [ ] GPU backend(s).
- [ ] Transfer planner / device-resident batches.
- [ ] Differential correctness suite against scalar oracle.
- [ ] Roofline/cache/bandwidth benchmarks.

## Non-goals

- Finance-specific compiler intrinsics.
- Direct exchange/network I/O from numerical kernels.
- Hidden shared mutable state.
- Raw accelerator handles exposed to guest strategy code.
- Object-per-tick designs on performance-critical paths.
- Nondeterministic backtests caused by actor scheduling order.
