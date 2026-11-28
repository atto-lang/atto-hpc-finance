# Numerical contracts

These rules are the baseline for finance kernels. Optimized Oreslang, SIMD, native, or GPU backends should be tested against the same contracts.

## Input validation

- Reject non-finite values for kernels whose semantics do not explicitly support NaN/Infinity.
- Price-domain logarithms require strictly positive prices.
- Simple returns reject a zero previous price.
- Window sizes must be non-zero and no larger than the input length.
- Output buffers must have the exact documented length.
- Annualization factors must be finite and strictly positive.

The default is to fail fast rather than let one invalid market value poison a long vector silently.

## Failure-atomic scalar outputs and overflow

A finite input is **not** sufficient evidence that an arithmetic result is
finite. For instance, dividing finite prices `1e300 / 1e-300` yields an
infinite simple return, and a rolling sum can overflow in a late window.

- `simple_returns_into` and `rolling_mean_into` must preflight all
  computed values before mutating any caller-owned output element.
- Output-domain failure uses `KernelError.code = "non_finite_result"`,
  with `index` identifying the first invalid **output** slot. Existing
  validation failures retain their own names and input indices.
- A failing call must preserve the caller's entire output buffer, even if
  earlier windows/returns were valid.
- The deterministic scalar preflight and publication passes must use the
  same arithmetic evaluation order; future fused/SIMD/GPU implementations
  must supply a separately verified atomic-publication strategy.
- An optimized backend must not compile finite-input validation using
  unsafe fast-math assumptions such as `x - x == 0` for NaN/infinity.
  Until a backend supplies an explicitly tested IEEE finite predicate,
  finite checks are **not** certified under relaxed math transformations.

Rust oracle tests and the Oreslang scalar source include this policy, but
an exact pinned compiler guest run is still required for parity claims.

## Output shape

Allocation-free `*_into` kernels receive caller-owned output buffers.

- Returns for `N` prices produce `N - 1` values.
- A valid rolling window `W` over `N` values produces `N - W + 1` values.
- Batch pricing kernels should preserve one output row per valid input instrument/scenario.

No sentinel warm-up NaNs are required when a compact output shape can represent the valid domain directly.

## Floating-point determinism

Three future execution modes are anticipated:

- **strict** — stable reduction order and reproducible results on the same supported backend;
- **reproducible** — backend-specific reproducibility within documented error tolerances;
- **relaxed** — permits reassociation/FMA/aggressive accelerator math for throughput.

The initial reference oracle uses ordinary IEEE-754 `f64` operations with deterministic scalar loop ordering.

Parallel reductions must not silently claim bit-for-bit equality with a sequential reduction unless they preserve the same reduction tree/order.

## Rolling statistics

The bootstrap rolling mean uses an O(N) sliding sum. More numerically robust implementations may replace it, but differential tests must define acceptable absolute/relative error bounds.

Variance/covariance should prefer stable online/pairwise algorithms over naive `sum(x*x) - sum(x)^2` formulations.

## Returns

Simple return:

```text
r_t = p_t / p_(t-1) - 1
```

Log return:

```text
r_t = ln(p_t / p_(t-1))
```

Log returns require strictly positive prices. The Rust reference keeps direct ratio logarithms for ordinary prices and uses a difference of logarithms when the ratio overflows or underflows, so finite positive extreme prices still produce finite log returns.

## Drawdown

Drawdown fraction is defined as:

```text
1 - current_price / running_peak_price
```

The maximum drawdown result records the peak index, trough index, and a non-negative fraction.

## Sharpe ratio

The bootstrap Sharpe ratio uses arithmetic excess returns and sample standard deviation:

```text
mean(excess) / sample_stddev(excess) * sqrt(periods_per_year)
```

The caller supplies the risk-free return **per period** and the annualization periods explicitly.

If volatility is zero, the reference implementation returns no finite Sharpe value instead of manufacturing Infinity.

## Pre-trade limit-order risk boundary

The final `risk_gate_kernel` decision is **fail closed**, even if the
upstream batch validator or strategy stage claims that the order was already
checked. An unapproved decision must never be passed to venue I/O.

- A limit-order side must be exactly `1` (buy) or `-1` (sell). `0`
  explicitly means no order and cannot be approved.
- Quantity must be finite and strictly positive; the configured maximum
  must be finite and nonnegative. A quantity exactly equal to the maximum is
  permitted.
- Limit price must be finite and strictly positive. A market order must
  use a distinct, explicitly authorized order kind rather than a zero-price
  limit order.
- Unknown side values, `NaN`, positive/negative infinity, negative/zero
  quantities, invalid prices, and over-limit quantities are denied.
- `RiskDecisionPoint.reason_code` is stable for this initial contract:
  `0=approved`, `1=no_side`, `2=negative_quantity`,
  `3=quantity_exceeds_limit`, `4=invalid_numeric`,
  `5=invalid_side`, `6=invalid_limit_price`, `7=zero_quantity`.
  The first matching rejection wins in this specified order: invalid
  numeric, no side, invalid side, negative quantity, zero quantity,
  invalid limit price, exceeding max quantity.

The shared acceptance inventory is `fixtures/risk-gate-v1.tsv` (23 reviewed
positive/negative vectors, with explicit `NaN` and infinity symbols). The
Rust oracle checks the inventory without third-party dependencies. A guest
adapter must parse these symbols explicitly and compare every decision and
reason code; a source-only inventory check is not runtime proof.

The Rust `pretrade_risk_gate` reference has adversarial unit tests for
this policy. **Rust-only tests do not establish that the Oreslang compute
kernel ran**, nor do they validate exchange integration. A future exact-head
guest test must compare the Oreslang decision/reason codes against these
vectors and assert that rejected orders never reach a venue capability.

## Signal overflow and execution isolation

A subtraction of individually finite inputs can yield an infinite signal:
`1e308 - (-1e308)` is positive infinity in strict IEEE-754 binary64.
Without a final signal check, this is incorrectly interpreted as a buy order.

The `latest_signal_kernel` must set **both** `SignalPoint.side=0` and
`SignalPoint.score=0.0` when the calculated score is nonfinite. This clears
a stale prior side in a reused output slot. The final risk gate rejects
`side=0` with `reason_code=1` regardless of upstream claims.

The design-only `RiskActor` must not forward denied batches; the
design-only `ExecutionActor` must independently reject them before any
future venue I/O. These actors currently live under `design/` and **are
not executable**. An approved decision is not itself a venue capability:
tenant identity, order generation, capability ownership, and authorization
must be checked again at the actual adapter.

**Compiler math mode is a security precondition.** Source patterns such as
`(score - score) == 0.0` are only reliable for NaN/infinity checks when
the backend actually preserves IEEE-754 special-value semantics. A local
Clang 17 host probe at `-O2 -fno-fast-math` correctly refused an
overflowed signal, while `-O2 -ffast-math` treated its finite check as
true and produced `side=1`. This is a host compiler demonstration,
**not evidence about Oreslang's generated code**. Reject/disable relaxed
math for financial authorization until each claimed backend provides
executed native guest negative receipts for NaN, infinity, and finite-input
subtraction overflow.

The Rust `latest_signal` oracle covers these numerical cases and checks
that the final `pretrade_risk_gate` rejects its no-order side. Neither
Rust source nor source-only Oreslang patches establish that the compute
kernel or actor pipeline actually executed.

## Randomness

Simulation APIs must accept explicit seed/stream state. No finance kernel may pull randomness from hidden process-global state.

Future parallel Monte Carlo APIs should use splittable/counter-based streams or another scheme where results do not depend on scheduler race order.
