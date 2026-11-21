> **AI agent instructions (organization-wide):** Before working on code, issues, or pull requests, read and follow the [organization `AGENTS.md`](https://github.com/ores-truffle-oreslang/.github/blob/main/AGENTS.md), including its **merge-first stealth-development policy**. Honor any repository-local `AGENTS.md`, explicit holds, and GitHub-enforced rules.

# oreslang-hpc-finance

## AttoLang migration and current contracts (2026-11-18)

**AttoLang (formerly Oreslang)** is the public language name. This `ores-truffle-oreslang/oreslang-hpc-finance` repository is the legacy named checkout; new corresponding work is routed to **no verified exact successor**. Existing history, issue/PR records, CI results, commands, and implementation status are not assumed identical across the two repos. The content and example commands below document this checkout, and legacy `.ores`, `oreslang-*` names and ABI/wire interfaces remain compatibility surfaces until proven migrations.

**[atto-spec](https://github.com/atto-lang/atto-spec)** is the canonical normative language specification (not old Oreslang README files); [atto-platform-conformance](https://github.com/atto-lang/atto-platform-conformance) owns portable positive/negative guest tests, and [atto-common-source](https://github.com/atto-lang/atto-common-source) owns portable `.atto` library code. Source design targets include `snake_case`, explicit `borrow!`/`take!`/`share!`/`copy!`/`copy!!`, [implicit argument borrowing](https://github.com/atto-lang/atto-spec/blob/main/language/implicit-argument-borrow-v1.md), [file/module `define const` with permitted one-time nonthrowing `os.read_env`](https://github.com/atto-lang/atto-spec/blob/main/language/file-module-const-v1.md) and [bitwise shifts `<<<`/`>>>`/`ashr` versus `<<`/`>>` stream operations](https://github.com/atto-lang/atto-spec/blob/main/language/bitwise-and-stream-operators-v1.md). These are intended semantics, not evidence this old project executes them.

No Python scripts, generators or test/CI harnesses for new AttoLang work; prefer Rust tools when necessary. Compiler/runtime parity requires exact commits/tree, toolchain, actual positive and negative compiled guest tests, and explicit unsupported/failed/not-executed results. Host-only tests or zero-step Actions cannot certify language support.

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
exact-head script specifies eleven Oreslang scalar and risk guest fixtures, but none
is certified until a reviewed compiler actually executes them. The Rust
model is not an Oreslang Region risk/actor integration test. See
`docs/unified-reconciliation.md` and `docs/hpc-verification-gates.md`.

The exact-head Oreslang-language guest suite now covers **eleven** programs,
**182** expected booleans, and a separate **180-case** finite risk cross-product;
all await actual compiler-backed execution. See `docs/oreslang-guest-parity.md`.
