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
- `chunk_count == 1 + (return_count - 1) / chunk_size`.
- Physical bar/return/rolling/chunk capacities exactly equal their logical
  counts. Aggregate moment, signal, order and decision Regions each have
  exactly one slot.
- Signed Oreslang sizes must be rejected for negative/zero invalid counts
  before converting to native allocator/indexing types.
- The final Welford chunk must compute `start = chunk * chunk_size` only
  after shape preflight. It then computes `remaining = count - start` and
  forms `index = start + offset` **only when** `offset < remaining`.
  This avoids the previous masked-lane integer overflow path.

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
   stdout is `true:true:true:true:true`.
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
