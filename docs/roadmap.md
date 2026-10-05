# Roadmap

## Phase 0 — bootstrap

- [x] Repository architecture.
- [x] Numerical semantics document.
- [x] Dependency-free Rust reference oracle.
- [x] Add `.oreslangc.cfg.toml` project metadata.
- [x] Add first Oreslang scalar oracle-compatible kernels (simple returns, rolling mean, mean, variance, drawdown).
- [ ] Add log returns and Sharpe once pure numeric `log`/`sqrt` intrinsics are executable outside compute-only analysis.
- [x] Add cross-language golden fixture data.
- [ ] Wire Rust/Oreslang differential runners to the shared fixtures.

## Phase 1 — market/statistics foundation

- [x] Add closed `Bar` model plus `Region<Bar>` SoA compute contract.
- [ ] Add concrete quote/trade batches and ingestion normalization.
- [ ] Ring/segmented buffers with bounded immutable views.
- [x] Simple-return scalar + Region kernel.
- [ ] Log-return scalar + Region kernel.
- [ ] SMA/EMA/VWAP.
- [ ] Rolling variance/stddev.
- [ ] Covariance/correlation matrices.
- [ ] Drawdown, Sharpe, Sortino, beta/alpha.
- [ ] Histogram/quantile primitives.

## Phase 2 — actor trading pipeline

- [x] Define `MarketDataActor -> compute -> StrategyActor -> RiskActor -> ExecutionActor` topology and ownership rules under `design/`.
- [ ] Promote the actor pipeline into `src/` after oreslang-source.java issue #236 supplies explicit unique region transfer + async compute submission.
- [ ] `MarketDataActor` production compute/orchestration implementation.
- [ ] `StrategyActor` production signal/order-intent implementation.
- [ ] `RiskActor` production pre-trade risk-gate implementation.
- [ ] `ExecutionActor` production terminal ownership/order-routing implementation.
- [ ] Venue/simulator order-routing capability behind `ExecutionActor`.
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

- [x] Kernel descriptors with explicit read/write/discard sets.
- [x] Chunked Welford algorithm + deterministic CPU merge contract.
- [ ] Runtime chunk-task executor.
- [x] Parallel/SIMD loop contracts on independent finance kernels.
- [ ] Concrete SIMD backend variants.
- [ ] Explicit transferable-region capability for actor/task ownership handoff (upstream issue #236).
- [ ] GPU backend variant(s) and asynchronous submission boundary.
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
