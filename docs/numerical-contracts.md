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

## Randomness

Simulation APIs must accept explicit seed/stream state. No finance kernel may pull randomness from hidden process-global state.

Future parallel Monte Carlo APIs should use splittable/counter-based streams or another scheme where results do not depend on scheduler race order.
