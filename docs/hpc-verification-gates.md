# HPC finance exact-head verification and promotion gates

Status: **DRAFT / NO_EXECUTED_GUEST_EVIDENCE**. This is a test plan, not
a certificate of Rust or Oreslang execution.

## Repository and input identity

For every check retain the PR head commit SHA, **root Git tree SHA**,
compiler binary digest and source/tree pin (if relevant), fixture Git blob SHA,
workflow run ID, run attempt, job ID, explicit step names/exit codes, stdout
and stderr artifact digests. Compare these identities to the source under test
after execution; never infer a tree SHA from a branch name.

Current acceptance inventory: `fixtures/risk-gate-v1.tsv`, 23 rows.
Format: `id|side|quantity|limit_price|max_quantity|approved|reason_code`.
`NaN`, `+Infinity` and `-Infinity` are explicitly typed IEEE-754
test values, **not** unspecified JSON/string coercions. Every case ID must
be unique, and extra/missing cases are a failing inventory change.

## Mandatory independent evidence classes

| Gate | What must actually execute | Passing evidence | What it does *not* prove |
|---|---|---|---|
| Admission | Source-org Actions checkout/test jobs | Real runner ID; nonempty executed steps; matching PR SHA/tree | Rust, compiler, numeric correctness |
| Rust reference | `cargo fmt --all -- --check`; `cargo test --offline --all-targets`; `cargo clippy --offline --all-targets -- -D warnings` from `reference/rust` | Nonzero-step, exit-zero exact-head receipts, all Rust tests, 23-case inventory | Oreslang source compiled or ran |
| Scalar guest | Compile/execute `tests/scalar_smoke.ores` and separate overflow/atomicity negative fixtures with pinned Oreslang compiler | Exact expected output, negative codes, unchanged buffers, source and compiler hashes | SoA `Region`, GPU, actor state |
| Compute guest | Compile/execute `src/kernels.ores` against the 23-case risk inventory on each claimed backend | Exact decisions and reason codes for every case, explicit finite behavior, no hidden fallback | Real venue execution, WCET, hardware |
| Ownership | Compile-positive unique transfers and compile-negative raw writable `Region<T>` alias/mailbox cases | Separate valid/invalid guest receipts at exact compiler SHA | Tenant isolation or backend GPU |
| Execution | Inject approved/rejected mock venue intents through an authorized actor integration | Every rejected order reaches **zero** venue calls, allowed orders follow explicitly authorized route | Live exchange integration |
| Hardware / hard RT | Real named device, native kernel/fence, ABI and timing harness | Device receipt, allocation bounds, cancellation, sustained resource budget, WCET where claimed | General portability to other hardware |

Do not combine host reference tests, source-only contract checks, synthetic
receipt tests, or wrapper scripts into a `VERIFIED_GUEST` result.

## Admission-failure classification

An Actions run with `steps=[]` or `runner_id=0` is
`RUNNER_NOT_ADMITTED` regardless of its GitHub conclusion. Never count
it as a compiler, source, numeric, or ABI failure. Require at least one
executed source-local step before interpreting job logs.

The private source **must not** be transferred to a different organization
just to acquire CI minutes without separate review and authorization.
Restoring a source-org runner or obtaining an explicitly authorized
source-free execution path is a prerequisite, not evidence of correctness.

## Merge and promotion blockers

- All requested source-local gates need executed nonzero-step evidence
  for the exact PR head and a real pinned Oreslang compiler where required.
- The finite predicate must remain sound under the actual backend math mode;
  `x - x == 0` is not an acceptable NaN/infinity check under unsafe fast-math.
- Preserve `non_finite_result`/buffer-atomicity behavior and reject
  malformed side, quantity, limit price and risk limits at the last boundary.
- `main` is currently unprotected; independent approval and a defined
  branch/ruleset protection gate are needed before promoting this draft.
- No claims of CPU SIMD, accelerator, actor isolation, native ABI, real venue
  access, or hard real-time are permitted from Rust/reference tests alone.
