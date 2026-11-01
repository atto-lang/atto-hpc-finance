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

**Total: six Oreslang guest programs, 56 expected boolean assertions.**

The new `src/risk_scalar.ores` independently implements the exact risk
decision/precedence contract **in pure Oreslang**. The 23-case guest uses
overflowed finite inputs to construct +/-Infinity and their difference
to construct NaN under strict IEEE-754. This is separate from both the Rust
host oracle and the production `Region<T>` compute kernel.

The Rust test checks that every TSV case ID occurs exactly once and in order
in the Oreslang guest. **A Rust source-contract pass is not execution of an
Oreslang program**. The runtime must evaluate all 56 checks against the real
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
