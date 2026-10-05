# oreslang-hpc-finance

High-performance finance, trading, quantitative research, and HPC routines for Oreslang.

This repository is intentionally **outside Oreslang core**. It is the proving ground for data-oriented kernels, actor-driven streaming pipelines, deterministic numerical routines, and future SIMD/GPU backends without coupling finance-domain APIs to the language/runtime.

## Architectural rule

Use **actors for ownership, orchestration, I/O, clocks, feeds, order routing, supervision, and stateful strategy lifecycles**. Use **data-oriented kernels for dense numerical work**.

An actor should normally hand a dense batch or bounded view to a pure kernel instead of representing every quote, bar, position, or path as an object with behavior. That keeps the scheduler responsive while giving the compiler/runtime a clean target for vectorization, parallel execution, and eventual CPU/GPU placement.

## Initial module map

- `market_data` — columnar quote/trade/bar batches and normalization.
- `returns` — simple/log returns and transforms.
- `stats` — reductions, rolling/windowed statistics, covariance/correlation.
- `indicators` — EMA/SMA/VWAP/volatility and signal primitives.
- `risk` — drawdown, exposure, VaR/CVaR, Sharpe/Sortino, Greeks.
- `pricing` — option/fixed-income pricing kernels.
- `simulation` — deterministic Monte Carlo and scenario generation.
- `backtest` — event ordering, clocks, fills, portfolio/accounting.
- `orderbook` — L1/L2/L3 data structures and matching/simulation.
- `optimization` — portfolio and execution optimization.
- `actors` — feed/strategy/execution/risk orchestration.
- `accelerator` — future SIMD/GPU device selection and transfer planning.

## Performance rules

1. Prefer struct-of-arrays (SoA) / columnar layouts on hot paths.
2. Keep numerical kernels allocation-free where practical: caller supplies output buffers.
3. No hidden global state, implicit clocks, or implicit randomness.
4. Kernels are synchronous compute units; actor `await` stays at orchestration/I/O boundaries.
5. Preserve deterministic ordering for replay/backtests.
6. Make invalid numeric inputs explicit rather than silently propagating corrupted state.
7. Optimize only behind a correctness oracle and benchmark contract.
8. Do not leak accelerator-specific semantics into finance-domain APIs.

## Bootstrap status

The first feature branch establishes architecture and a dependency-free Rust reference oracle for core kernels. The Rust crate is **not** the public Oreslang API; it exists to pin numerical semantics and provide a portable test/benchmark oracle while the Oreslang package surface is built out.

See:

- `docs/architecture.md`
- `docs/oreslang-api.md`
- `docs/numerical-contracts.md`
- `docs/roadmap.md`
- `reference/rust/`
