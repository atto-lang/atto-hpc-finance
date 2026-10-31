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

## Source-local exact-head scalar guest runner

The checked-in `scripts/run-scalar-guest-exact-head.sh` is an **opt-in**
compiler-backed execution gate, not a mock runner. It executes exactly
`tests/scalar_smoke.ores` and `tests/scalar_overflow_atomic.ores`
through one reviewed Oreslang CLI and compares byte-exact expected stdout.
Any compiler stderr, nonzero exit, output mismatch, dirty checkout, source
identity mismatch, or compiler digest mismatch fails closed.

For an independently reviewed source commit/root tree and compiler binary:

```bash
# Review and supply these exact values independently; do not derive approval
# by trusting whichever branch happened to be checked out.
export EXPECTED_SOURCE_SHA='<reviewed 40-char commit SHA>'
export EXPECTED_SOURCE_TREE='<reviewed 40-char root tree SHA>'
export EXPECTED_COMPILER_SHA256='<reviewed compiler binary SHA256>'
export ORESLANG_BIN='/absolute/path/to/reviewed/oreslang'
export HPC_EVIDENCE_DIR='/absolute/path/outside/source/for/evidence'
bash scripts/run-scalar-guest-exact-head.sh
```

Run in a clean checkout of that exact commit. The script hashes the compiler
before and after execution and rechecks the checkout identity and cleanliness.
Its emitted status `EXACT_HEAD_SCALAR_ONLY` deliberately does **not**
certify the `Region<T>` compute kernels or the 23 risk-gate guest cases:
those still need a real compiler-backed Region allocator/harness and separate
exact-head positive/negative receipts.

`scripts/test-scalar-guest-gate.sh` tests **only this script's refusal
behavior** with a mock executable. The existing Rust workflow runs this
self-test; its `MOCK_ONLY_NOT_ORESLANG` status is not guest evidence.
The self-test cannot repair GitHub Actions runner admission failures.


## Source-free admission diagnostic

The separate `.github/workflows/source-free-runner-canary.yml` runs
a single shell echo on `ubuntu-latest`, without checkout, dependencies,
private source, secrets, or network access from job steps.

At commit `4bd0f8689d8079b54e2d1e038052cec692348e9c`:
- canary run `41173692416`: **runner_id=0, steps=[]**;
- Rust oracle run `41173692310`: **runner_id=0, steps=[]**.

These are **RUNNER_NOT_ADMITTED** and prove neither a defect in the
tests nor an actionable cause such as exhausted billing minutes. An
authorized repository/organization administrator must inspect Actions
eligibility, usage/billing, policies, and runner access. See issue #15.

## Numeric regression seed correctness

For strict IEEE-754 binary64, `1e300 + 1e300 = 2e300` is
**finite**, so it is *not* a valid rolling-overflow regression.
`1e308 + 1e308` exceeds the maximum finite double and becomes
positive infinity. The scalar guest target uses `1.0e308`,
requires seven colon-delimited `true` assertions, and also checks
that opposite-sign finite endpoints have mean zero. The mock-only
runner refusal self-test must use the same **seven** outputs.

The host-model arithmetic observation is **not** evidence that the
current Oreslang compiler accepts the scientific-notation syntax,
supports the imported `Result` APIs, or executes a numeric kernel.
Always require a pinned executed guest receipt for those claims.

## Overlapping draft integration

PR #12 adds independent extreme-mean coverage; PR #13 combines an
earlier PR #11 head with PR #12; PR #14 stacks compute-batch shape
and Welford index bounds on PR #13. PR #11 has advanced beyond the
ancestor in PR #13 and now also includes mean parity and extra
runtime-gate fixtures. Before merging, resolve this stack on a
reviewed **single** tree, preserving both PR #14 batch bounds and
PR #11 late changes. Do not merge the overlapping drafts in parallel,
and do not infer executed compiler or Rust coverage from mergeability.
