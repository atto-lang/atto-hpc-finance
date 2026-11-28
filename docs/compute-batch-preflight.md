# HPC compute-batch preflight and Welford index safety

Status: **DRAFT / SOURCE_CONTRACT_AND_REFERENCE_ONLY**.

## Why this exists

The accelerated `src/kernels.ores` functions accept `Region<T>` buffers,
index directly, and rely on their caller to supply valid dimensions.
`latest_signal_kernel` immediately reads the last return and rolling
value, and `welford_chunk_kernel` forms an index for the final padded chunk.
An out-of-range or overflowed index must never be handed to a backend.

This change adds `finance_scalar.validate_compute_batch_shape`, taking a
`ComputeBatchShape` with logical dimensions and physical capacities, plus
a matching independent Rust reference oracle.

### Required invariants

- `bar_count >= 2` because a signal requires at least one computed return.
- `return_count == bar_count - 1`.
- `1 <= window <= return_count`.
- `rolling_count == return_count - window + 1`.
- `chunk_size >= 1`.
- `chunk_count == 1 + floor((return_count - 1) / chunk_size)` as an
  **integer mathematical equality**. The Oreslang implementation uses
  `exact_nonnegative_quotient` (bounded doubling and subtraction) rather
  than the `/` operator, which some evaluator paths lower to floating
  arithmetic and can miscount beyond 2^53.
- Physical bar/return/rolling/chunk capacities exactly equal their logical
  counts. Aggregate moment, signal, order and decision Regions each have
  exactly one slot.
- Signed Oreslang sizes must be rejected for negative/zero invalid counts
  before converting to native allocator/indexing types.
- The final Welford chunk must compute `start = chunk * chunk_size` only
  after shape preflight. It then computes `remaining = count - start` and
  iterates only `lane_count = min(remaining, chunk_size)` actual elements,
  forming `index = start + offset` only for `offset < lane_count`.
  This avoids masked-lane integer overflow **and** near-unbounded wasted
  iterations when a valid `chunk_size` is much larger than the remaining
  input (for example one return and a maximum-size chunk).

For a validated full pipeline, `chunk < chunk_count` implies
`start < count` and `start + offset < count` for admitted offsets.
The preflight's ceiling formula avoids `count + chunk_size - 1` overflow.

## Trust and enforcement boundary

**Never** treat the counts in an actor-supplied message or an arbitrary
`ComputeBatchShape` object as proof of physical Region allocation.
Physical lengths must be read from a trusted runtime Region descriptor;
that descriptor must remain stable while the compute task executes.
Unique ownership or safe scoped borrowing must be verified separately.

This helper does not currently wrap every compute entrypoint, and the actor
pipeline in `design/pipeline.ores` is intentionally non-executable while
upstream transferable-region ownership and async submission (source.java
issue #236) remain unresolved. Until a real dispatcher always invokes
preflight on trusted metadata, **the entrypoints themselves are not
certified memory-safe**. No implicit approval or raw Region mailbox
allowlisting is introduced by this change.

## Acceptance / release gates

1. Review the exact finance branch head and root Git tree, and preserve
   the risk gate and numerical protections from integrated PR #13.
2. Execute `cargo fmt --all -- --check`, `cargo test --offline
   --all-targets`, and `cargo clippy --offline --all-targets -- -D
   warnings` at exact head. `reference/rust/src/batch_shape.rs` covers
   valid, negative, mismatched and near-`usize::MAX` count paths.
3. Compile and run `tests/batch_shape.ores` at an explicitly pinned
   Oreslang compiler SHA and strict integer overflow policy; expected
   stdout is `true:true:true:true:true:true:true`. The sixth case checks
   2^53+4 returns / chunk size 2 (exact chunk count 4503599627370498);
   a floating quotient incorrectly yields 4503599627370499. The seventh
   checks the signed-i64 maximum as the logical bar count.
4. Compile/run the exact Region kernel under ASan/UBSan-capable native
   configurations and each claimed backend. Include final partially filled
   chunk, maximum checked integer index, and missing/undersized allocation
   adversarial tests. Assert no attempted invalid read/write.
5. Prove trusted descriptor capacities and borrow/transfer gates cannot
   be bypassed by untrusted actor metadata or stale mutable views.
6. Verify CPU deterministic reduction order and report separate GPU/SIMD
   numerical and hardware-specific evidence. Do not claim WCET without a
   measured bounded execution harness.

Latest inspected source-org runner jobs had `runner_id=0` and
`steps=[]`; such results mean **RUNNER_NOT_ADMITTED**, not passing
or failing arithmetic. Private-source cross-org transfer still requires
separate authorization; do not bypass that gate to obtain CI minutes.

## Exact quotient safety model and current limits

The source-local `exact_nonnegative_quotient(dividend, divisor)` expects
`dividend >= 0` and `divisor > 0`. It repeatedly doubles a chunk only
when `chunk <= remaining - chunk`. Consequently the next doubled value is
no greater than `remaining` and cannot overflow a signed `int`. Its
matching multiplicity cannot overflow either because `divisor >= 1`.
Each subtraction reduces the remainder; the algorithm is bounded by the
binary length of the dividend rather than the number of chunks. The batch
validator establishes these preconditions before calling it.

**Independent local model check:** 46,524 Python integer arithmetic cases
(including deterministic random signed-64-bit inputs and boundary values)
matched Python's exact `//` result. This verifies a model, not the Oreslang
parser, signed arithmetic semantics, compiler lowering, or actual Region
allocations. Exact guest tests are still required. At present none have run.

The ceiling/count proof **assumes the actual Region capacity values were
obtained from trusted stable runtime metadata**. It does not authorize the
actor to supply arbitrary capacity claims, nor does it connect the helper
to the compute dispatcher. Unchecked launch entrypoints remain blocked.
