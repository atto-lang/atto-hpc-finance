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

## Repository ownership and migration (2026-11-08)

**Current:** `ores-truffle-oreslang/oreslang-hpc-finance` in the old Oreslang org. **Public language:** AttoLang. Retain existing `.ores` files and historical repository/package/CLI aliases until compiler-backed `.atto` support and consumers have been validated.

- **This repo owns:** reproducible numerical/HPC finance kernels, overflow and negative-case testing.
- **Not owned here:** investment services, live trading or unverified performance claims.
- **Organizations:** [atto-lang](https://github.com/atto-lang) is for language/compiler/tooling, portable ML/scientific libraries, conformance and demonstrations. [atto-co](https://github.com/atto-co) is for independently governed cloud hosting, deployment, tenant identity, billing and commercial services. Similar `atto-*` repo names do not imply transferred source ID or content parity.
- **Migration proposal:** identity-preserving GitHub transfer under the original repository name to `atto-lang/oreslang-hpc-finance` after reviews. No legacy-org direct move to `atto-co` is currently authorized. Later commercialization spinouts need their own signed off runbook.

**Repository transfer gates:** coordinate a single owner/rollback plan at [issue #26](https://github.com/ores-truffle-oreslang/.github/issues/26); consult private [runbook #25](https://github.com/ores-truffle-oreslang/.github/pull/25). Record exact source commit SHA/tree and numeric ID, target visibility and protection/rulesets, source/target admin and independent security approvals, PRs/tags/releases, packages/import consumers, webhooks, GitHub Apps, Actions/billing/runners, secrets and Pages. Never mirror private source without separate approval. After actual GitHub transfer verify identical ID/history/tree, restore protections, update references independently and run exact-head CI with nonzero executed steps. `steps=[]`/`null` is `NOT_EXECUTED`; host/static tests cannot establish compiled AttoLang numerical or guest execution parity.
