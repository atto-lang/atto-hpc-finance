# Oreslang compiler-backed HPC finance guest tests

**Evidence status: AUTHORED, not yet compiler-executed.** This is a source
acceptance matrix and exact-head execution recipe, not a passing test report.

## Exact guest inventory

| Actual Oreslang program | Assertions | Meaning |
| --- | ---: | --- |
| `tests/scalar_smoke.ores` | 4 | Returns, rolling mean, sample variance |
| `tests/scalar_overflow_atomic.ores` | 7 | Late failure, unchanged buffers, finite extreme mean |
| `tests/mean_extremes.ores` | 5 | Opposite-sign extreme means in both orders |
| `tests/batch_shape.ores` | 5 | Logical batch-shape admission, not runtime capacities |
| `tests/scalar_error_matrix.ores` | 12 | Invalid counts/lengths/windows/denominators, nonfinite input, untouched buffers |
| `tests/risk_gate_scalar_23.ores` | 23 | All unique rows from `fixtures/risk-gate-v1.tsv` |
| `tests/risk_gate_cross_product.ores` | 12 | 180 Oreslang decisions, full eight-reason distribution |
| `tests/batch_shape_adversarial.ores` | 37 | Seven accepted and 30 rejected shape/capacity combinations, including huge logical geometry |
| `tests/scalar_exact_values.ores` | 24 | Exact returns, rolling means, online means, and sample variances |
| `tests/scalar_failure_atomicity_matrix.ores` | 19 | Late faults, sentinels unchanged, negative/NaN/overflow boundaries |
| `tests/risk_precedence_matrix.ores` | 34 | Multi-fault risk reason precedence, signed zero, NaN and infinities |

**Total: eleven Oreslang guest programs, 182 expected boolean assertions, plus 180 in-guest risk evaluations.**

The new `src/risk_scalar.ores` independently implements the exact risk
decision/precedence contract **in pure Oreslang**. The 23-case guest uses
overflowed finite inputs to construct +/-Infinity and their difference
to construct NaN under strict IEEE-754. This is separate from both the Rust
host oracle and the production `Region<T>` compute kernel.

Rust source-contract tests check all four new case inventories for uniqueness, exact expected counts, and consistency with the guest runner and mock-only admission suite. The existing Rust test checks that every TSV case ID occurs exactly once and in order
in the Oreslang guest. **A Rust source-contract pass is not execution of an
Oreslang program**. The runtime must evaluate all 182 checks against the real
compiler and emit byte-identical expected output.

## Trusted, exact-head guest run

1. Independently review the finance commit, root tree, and the Oreslang
   compiler source/build identity. Pin the native or JVM CLI launcher
   executable SHA-256. Do not accept an arbitrary caller-chosen mock binary as
   compiler evidence.
2. Run on a source-organization runner or an otherwise specifically approved
   environment with the actual compiler installed. Do not transfer private
   source into another organization to obtain Actions minutes.
3. In a clean checkout of the *reviewed* finance commit, set:

```bash
export EXPECTED_SOURCE_SHA='<reviewed finance commit>'
export EXPECTED_SOURCE_TREE='<reviewed finance root Git tree>'
export EXPECTED_COMPILER_SHA256='<independently reviewed compiler SHA256>'
export ORESLANG_BIN='/absolute/path/to/reviewed/oreslang'
export HPC_EVIDENCE_DIR='/absolute/output/path/outside/checkout'
export HPC_GUEST_TIMEOUT_SECONDS=60
bash scripts/run-scalar-guest-exact-head.sh
```

The script rejects a dirty checkout, missing/symlinked binary, mismatched
commit/tree or compiler digest, invalid timeout, unexpected guest stderr,
nonzero guest exit, and output mismatch. For every Oreslang program it records
its Git blob and expected/stdout/stderr SHA-256 receipts.

The CLI accepts an absolute `.ores` program path and links imports.
`--check` can be used as an additional compiler-only admission check, but
is **not** guest execution. Do not silently claim guest success from
`--check`, host models, or the script's mock-only self-tests.

## Still separate and mandatory

The 23-case scalar risk guest does **not** prove the `Region<T>` risk kernel:
the latter requires trusted runtime Region allocation, physical-length
validation, actual compute execution, reason-code parity and a zero-call
mock-venue actor boundary for all rejected orders. The Rust host mock venue
also does not establish Oreslang actor isolation. Negative ownership
compilation, native/CPU/GPU lowering, and hardware timing require distinct
evidence. Source-free CI runner admission is a prerequisite, not a test pass.

Current diagnostic: prior exact-head finance Rust success belongs to
`c274ed3ac2e399b850d8ea7a9370c38f70efe44c`, not newer commits.
Later source-free and Rust jobs returned `steps=[]` and must be marked
`RUNNER_NOT_ADMITTED`; the cause remains unproven.

## Independent upstream GraalVM compiler characterization

The separately scoped [oreslang-source.java PR #2394](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2394)
adds two Java JUnit methods that **execute independently authored Oreslang
source via the actual GraalVM polyglot runtime**: strict IEEE-754 overflow,
NaN and weighted extreme means. They do **not** contain or mirror the private
finance repository's source bytes. Passing them would characterize the
upstream Oreslang compiler at that compiler PR SHA, **not** certify the
finance branch or its import/linker, `Region<T>`, or actor execution.

As inspected, the upstream targeted GitHub workflow has `steps=null` and
is classified `RUNNER_NOT_ADMITTED`, so there is **no executed compiler
result yet**. Keep the two evidence streams independent.

The 180-case Oreslang cross-product additionally checks each reason-code count:
`[2,24,12,6,60,48,16,12]` for reason codes `0..7`. These counts were
independently cross-checked against a JavaScript **host model**, not an
Oreslang execution. A real compiler must reproduce them exactly.

## Manual-only source-org compiler workflow

`.github/workflows/oreslang-guest-exact-head.yml` is an opt-in,
`workflow_dispatch`-only **real Oreslang** execution gate. It requires a
separately approved self-hosted Linux runner labeled `oreslang-approved`,
a protected `hpc-oreslang-approved` environment and independently reviewed
repository/environment variables: `HPC_REVIEWED_SOURCE_SHA`,
`HPC_REVIEWED_SOURCE_TREE`, `HPC_REVIEWED_COMPILER_SHA256`, and
`HPC_REVIEWED_COMPILER_PATH`. Do not configure, approve or dispatch this
workflow solely based on a PR author's request. It refuses incomplete pins,
checks the source ref, checks out **only this same source repository** using
read-only checkout, and executes the eleven Oreslang guests against the
approved CLI. No private source mirror or cross-org token is involved.

The workflow **has not been dispatched** and no authorized native compiler
binary or independent environment review has been established. Workflow
presence and mock-only Bash tests must never be described as guest evidence.

## November 1 source-only regression expansion

Four additional **pure Oreslang guest programs** contribute **114 new boolean
assertions** (37 batch admission, 24 numeric exact-value, 19 failure-atomicity,
34 risk-precedence). The pinned runner and its mock-only self-test cover all
**eleven** source fixtures (182 expected booleans). The batch geometry cases
are shape-only, not physical Region allocation. A dedicated Rust test checks
case-marker uniqueness and receipt/script consistency; it does not run guest
code. Do not report these as executed unless the approved compiler completes
all eleven exact-blob fixtures at one immutable source head.
