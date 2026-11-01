# HPC Finance — backend parity gates (November 1, 2026)

**Status: NOT AT COMPILER OR BEHAVIORAL PARITY.** This is the canonical
implementation/verification backlog, not a release certification.

The authoritative *finance* suite on PR #18 has **11 native Oreslang
guest programs**, **182 expected boolean assertions**, an additional
**180-decision finite risk cross-product**, and **23 risk-case contract
rows**, Git blob `cec489c1f7603617797340dd1b4a71cc85acfebd`.

| Backend | Contract PR | Host-reference checks added | Production guest proof |
| --- | --- | --- | --- |
| Java/GraalVM | [Arithmetic guest #2394](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/2394) | Two independently authored Oreslang-in-GraalVM test methods | No admitted/stepful finance guest receipt; 182 exact-head finance assertions remain open |
| LLVM native | [Host+negative #1258](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/pull/1258) | C++17 native host oracle (23 TSV rows, 180 cross-product decisions, 4 mutation denials) and production compiler *negative admission* for unsupported f64/Region | **NOT VERIFIED**; binary64/Region finance source lowering is an open [P0 #1259](https://github.com/ores-truffle-oreslang/oreslang-source.llvm/issues/1259) |
| WASM | [Host+capability #220](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/pull/220) | Node 22 host oracle and fail-closed binary64 capability declaration; native source compiler delegated to LLVM | **NOT VERIFIED**; real wasm32 f64/Region guest is [P0 #221](https://github.com/ores-truffle-oreslang/oreslang-source.wasm/issues/221) |
| .NET/NativeAOT | [Host+negative #283](https://github.com/ores-truffle-oreslang/oreslang-source.net/pull/283) | C# host oracle, test entrypoint and source parser *negative admission* | **NOT VERIFIED**; real f64/Region CIL/NativeAOT is [P0 #284](https://github.com/ores-truffle-oreslang/oreslang-source.net/issues/284) |

## Shared contract identity

The LLVM, Wasm and .NET PRs contain byte-identical copies of exactly
`fixtures/risk-gate-v1.tsv` from this repo, under
`contracts/v1/hpc-finance/risk-gate-v1.tsv`. Verified Git blob SHA:
`cec489c1f7603617797340dd1b4a71cc85acfebd`.
This is within the same source-owning GitHub organization; **no private
source was exported to another organization**.

The 180-case reference outcome counts for reason codes 0..7 are
`[2,24,12,6,60,48,16,12]`. The host models *do not* establish
LLVM IR lowering, Wasm execution or managed NativeAOT guest correctness.

## Promotion blockers — every backend

1. Compile/execute the **same 11 actual finance `.ores` fixtures** with a
   reviewed compiler source SHA, compiler binary digest, immutable finance
   source commit and root Git tree. Validate all **182** byte-exact booleans,
   stdout/stderr artifact digests, error codes and no skipped cases.
2. Independently run `src/kernels.ores` risk gate over all **23** cases
   through real `Region<T>` allocations, ownership and physical-capacity
   checks (not a hand-rolled host reference).
3. Preserve strict IEEE-754 non-finite rejection, numeric precedence, atomic
   no-output-write on rejection, bounded loops and no implicit conversion,
   operator overloading, unsafe fast-math, or silent CPU fallback.
4. Execute negative compiler/ownership/malformed-source cases with no
   published guest artifact; verify actual actor mock venue receives **zero
   calls for rejected orders**.
5. Require real GitHub Actions runner admission, **nonempty steps**, native
   build/runtime receipts, independent review and branch protection.
   `steps=[]` is admission failure, **never guest or oracle evidence**.

The three backend parity PRs are **draft**. They are small, source-local
verification foundations, not proof that parity has been attained.
