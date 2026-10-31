# Fail-closed HPC compute/venue dispatch contract

Status: **PROPOSED REFERENCE MODEL / NOT NATIVE EXECUTION OR COMPILER EVIDENCE**.

The Rust `reference/rust/src/dispatch_gate.rs` is an **independent, allocation-free logical state machine**, not an Oreslang actor implementation. It deliberately does not invent a callable `compute.submit`, transferable `Region<T>`, GPU fence, or network venue API. The existing `design/pipeline.ores` remains outside compiler source roots until upstream runtime issue `oreslang-source.java#236` supplies real ownership and async submission capabilities.

## Non-negotiable order of operations

1. Obtain each physical Region length, generation and ownership domain **from trusted runtime-owned metadata** under a stable task/borrow lease, not from a serialized message. Compare every untrusted claimed capacity against those actual lengths, then perform `validate_compute_batch_shape`. Reject mismatch before kernel dispatch.
2. Require **strict** IEEE-special-value semantics for all numerical values that can influence a trade. Reproducible/relaxed modes are currently denied. The source expressions `(x - x) == 0.0` are not certified under unsafe fast-math, vector reassociation or unchecked FP compilation; the real backend must prove executed NaN/inf/overflow negatives.
3. Allocate a monotonic **generation ticket** for every batch submission, including repeated batch IDs. Reject concurrent reuse of the same work envelope. Do not leak raw native/GPU pointers into guest messages.
4. Set the risk decision state to `PENDING`, invalidating any previously approved one-slot value. No venue permit exists while the risk kernel is executing, while its result is merely staged, or after cancellation.
5. Only the owning runtime dispatcher (not a guest or device callback) may publish a completed, release/acquire-synchronized **completion fence** for the matching ticket and region generation. A GPU callback must settle a Future and re-enqueue on the actor's scheduler domain; it must not run actor code inline or bypass the fence.
6. Consume the matching fenced decision **at most once**. An approval requires `approved=true`, `reason_code=0`, and a fresh terminal validation of side ±1, finite positive quantity and price, finite nonnegative limit, and quantity ≤ limit. Any denial discards the generation's decision and triggers **zero** venue calls. An approved mock permit is non-Clone and consumed by the mock venue.
7. Cancellation and completion are ordered by the owning scheduler. A late callback for a cancelled/reused generation cannot revive approval or duplicate an order. Producer completion never grants network I/O, tenant authority, or cross-isolate access on its own.

## What the reference does *not* prove

`BatchShape` and `RegionLengths` are plain Rust data in the reference. A malicious host caller can fabricate either one. Their names and equality checks do not establish runtime descriptor provenance, pinned lifetime, memory safety, or borrow isolation. Likewise, the reference `record_fence` call has no physical fence and must **not** be exposed to untrusted Oreslang guest code. The production runtime must gate it behind its scheduler and actual device/CPU completion primitive.

Until this is wired into every real compute-dispatch path and compiled at the claimed Oreslang backend head, all native Region memory-safety, scheduler, SIMD/GPU, WCET and live-trading claims remain unverified. The reference uses a synthetic counting `MockVenue`; it intentionally contains no networking or exchange credentials.

## Mandatory executable gates

- `cd reference/rust && cargo fmt --all -- --check && cargo test --offline --all-targets && cargo clippy --offline --all-targets -- -D warnings` at the exact reviewed finance commit and tree; specifically execute `dispatch_gate` negative cases (stale ticket, pre-fence read, cancel, invalid capacity, relaxed math, terminal order, rejection, one-use permit).
- Pin a real compiler binary digest and execute the existing four scalar/shape Oreslang guest fixtures, then separately run a compiled compute+actor harness proving decision generation isolation, real fence ordering, unique transfer, negative borrow checks, and no emitted venue calls for all denied cases.
- Compare against the 23-case risk inventory and run strict and `-ffast-math`-equivalent backend configurations. If the strict check fails or relaxed code can authorize nonfinite data, **block authorization** rather than silently substitute a host oracle.
- Before merge: restore source-org runner admission (issue #15), record exact commit/root-tree/runner/job/step evidence, obtain independent review, and protect `main`. Private-source cross-org mirroring requires separate security authorization.

## Host C math-mode probe (not Oreslang)

The standalone C fixture `reference/host/strict_math_probe.c` and
`reference/host/run_strict_math_probe.sh` compare `clang -O2 -fno-fast-math`
and `clang -O2 -ffast-math`. On local Clang 17, the strict build reported
`HOST_STRICT_AUTHORIZED=0` while the fast-math build reported
`HOST_FAST_MATH_AUTHORIZED=1` for overflowed finite operands.
The host probe requires strict=0 and reports fast-math output without assuming
it is portable across compiler versions. It has no venue I/O. This is a
native **host C** observation, NOT a compiled Oreslang finance guest receipt.
Promotion still requires actual backend math-mode enforcement and negative
Oreslang kernel execution evidence.
