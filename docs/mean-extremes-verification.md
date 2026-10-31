# Opposite-sign finite mean parity and executable gate

## Defect and scope

The scalar Oreslang `finance_scalar.mean` previously used
`result += (value - result) / n`. If the running mean and incoming finite
value have opposite signs and extreme magnitudes, the subtraction can overflow
to infinity even though the correct weighted average is finite.

The Rust oracle in `reference/rust/src/lib.rs` already avoids this defect.
The Oreslang scalar implementation now mirrors the reference's conditional
update: when the signs differ, weight the two values before addition; for
same-sign values, retain the original incremental update.

This is a **numerical correctness patch**, not a guarantee of exact
bit-for-bit equivalence under arbitrary reassociation, fast-math, parallel
reduction trees, CPU/GPU implementations, or all Oreslang backends.

## Regression inventory

`tests/mean_extremes.ores` is a dedicated Oreslang guest fixture with five
checks and expected stdout `true:true:true:true:true`:

1. `[1e308, -1e308]` -> `0`, with no intermediate overflow;
2. `[-1e308, 1e308]` -> `0`;
3. `[1e308, 1e308]` -> `1e308`;
4. `[-1e308, -1e308]` -> `-1e308`;
5. `[1, 2, 3]` -> `2`.

These are source-committed **acceptance fixtures, not executed tests**.
The existing Rust oracle independently tests `f64::MAX` in both opposite-sign
orders, equal extremes and ordinary values.

## Promotion checks

- Pin the exact finance PR **commit and Git tree**, fixture blob,
  compiler/toolchain source commit and binary digest.
- Require a nonzero-step, nonzero-runner-id CI job. A failed run with
  `steps=[]` is an admission failure, not an arithmetic failure.
- Execute Rust oracle formatting, tests and Clippy on the checked-in source.
- Compile and execute `tests/mean_extremes.ores` with the pinned compiler;
  assert exit code zero and byte-exact stdout. Record stdout, stderr and
  artifact digests. Test the original failing inputs before promotion.
- Repeat on each compiler backend for which numerical parity is claimed,
  preserving deterministic floating-point settings and recording deviations.
- Keep `src/kernels.ores` and the actor/device boundary outside this patch's
  claims. There is no evidence here for GPU, hard-real-time, latency bounds,
  or transferable `Region<T>` ownership.

Source-org CI admission and the upstream compiler's integrated feature set
currently prevent a verified Oreslang guest claim; leave that gate **OPEN**.
