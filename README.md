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

The draft implementation now contains both a dependency-free Rust correctness oracle and the first Oreslang implementation tranche.

- `src/model.ores` — closed/AOT-safe finance data shapes.
- `src/scalar.ores` — validated scalar Oreslang kernels for simple returns, rolling mean, mean, sample variance, and maximum drawdown.
- `src/kernels.ores` — `Region<T>` data-oriented kernels with explicit effects, SoA layout, automatic placement, SIMD/parallel loops, chunked Welford statistics, signal generation, order intent, and risk gating.
- `design/pipeline.ores` — reviewed actor/compute topology kept outside source roots until OresVM has an explicit transferable-region capability; raw writable `Region<T>` is intentionally not mailbox-sendable.
- `tests/scalar_smoke.ores` — executable smoke fixture for the integrated upstream language surface.
- `fixtures/core-golden.json` — shared numerical vectors/error cases for cross-language parity.
- `reference/rust/` — scalar parity/correctness oracle; it is not the public implementation.

The Oreslang source deliberately targets active compiler/runtime contracts that are still split across upstream draft PRs. `docs/upstream-feature-matrix.md` records those dependencies. Two upstream seams remain explicit: direct `compute fnc` calls still execute synchronously in the current reference interpreter, and hardened actor boundaries correctly reject raw `Region<T>` mailbox values. Oreslang runtime issue #236 tracks the required unique region-transfer capability plus an awaitable compute-submission boundary; the library will not weaken either safety rule locally.

See:

- `docs/architecture.md`
- `docs/actor-pipeline.md`
- `docs/oreslang-api.md`
- `docs/numerical-contracts.md`
- `docs/library-boundaries.md`
- `docs/upstream-feature-matrix.md`
- `docs/roadmap.md`

## Compiler PR checkpoint (2026-10-06)

A [repository-specific compiler compatibility matrix](docs/compiler-pr-sync.md) tracks the 10 newest `oreslang-source.java` PRs. They remain unmerged upstream; existing demos, build flags, and stable compiler references stay unchanged pending exact-SHA conformance testing.

## Continuous integration

The `Numerical reference oracle` workflow runs Rust formatting, all oracle tests,
and Clippy with warnings denied on pushes and pull requests. It uses read-only
permissions and no repository secrets; the oracle has no external dependencies.
This gate validates the Rust reference only. It does not claim Oreslang kernel,
actor-region transfer, GPU, or cross-language parity coverage. Those remain
subject to the compiler integration contracts above.

## Unified risk and scalar hardening (draft)

The integration branch retains PR #11's latest failure-atomic scalar and
fail-closed risk work, PR #12's independent extreme-mean fixture, and PR #14's
batch-shape validator, Welford bounds, and Rust oracle. The source-local
exact-head script specifies four Oreslang scalar guest fixtures, but none
is certified until a reviewed compiler actually executes them. The Rust
model is not an Oreslang Region risk/actor integration test. See
`docs/unified-reconciliation.md` and `docs/hpc-verification-gates.md`.

The exact-head Oreslang-language guest suite now covers **seven** programs,
**60** expected booleans, and a separate **180-case** finite risk cross-product;
all await actual compiler-backed execution. See `docs/oreslang-guest-parity.md`.
