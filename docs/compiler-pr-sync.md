# Compiler PR compatibility — 2026-10-06

**Focus:** Finance HPC kernels and fixed data pipelines in `oreslang-hpc-finance`.

All of the latest ten `oreslang-source.java` PRs were **open** at inspection, not shipped on `main`: [#398](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/398), [#399](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/399), [#404](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/404), [#405](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/405), [#406](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/406), [#408](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/408), [#409](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/409), [#410](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/410), [#411](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/411), [#412](https://github.com/ores-truffle-oreslang/oreslang-source.java/pull/412).

**Branch dependencies:** #405 -> #406 -> #408 -> #412; #404 sits on a separate match feature branch. Runtime #399, optimization #409, channel #410, binding #411 and match #398 need reconciliation. No consumer should pin a partially stacked PR and assume its sibling semantics.

## Migration / hardening matrix

- Preserve deterministic numeric kernels regardless of select fairness; use select first only if strict priority is truly desired (#409).
- Proof mutable alias/ownership separation for GPU/CPU buffers; val is fixed mutable, const fixed readonly, let rebindable readonly (#411).
- Do not send mutable numeric buffers across actor mailboxes by reference (#399/#410).
- Keep existing CPU reference golden fixtures and model results unchanged.

## Verification before enabling new constructs

- Execute: `Validate tests/scalar_smoke.ores and compare fixtures/core-golden.json to reference/rust` (according to environment and CI setup).
- Run old-syntax and proposed-syntax tests against exact compiler SHAs; do **not** call tests passed until an actual runner executed them.
- Preserve existing CLI flags, compiler pins, public exports, and baseline behavior by default.
- Verify `const`/ `val`/ `let`/ `mut` and ownership changes do not create mutable cross-actor aliases.
- No pointer symbols (`&`, `*`) in new public Oreslang source; use `rt borrow`, `rt copy`, `rt take`, `rt share` and, where implemented, `rt proxy`.
- For exhausted source Actions minutes, CI may mirror the **exact source Git tree** into a funded test organization's repo without importing secrets; compare Git tree and blob identities first, and attach the exact runner results. No runner/steps means no test evidence.

This PR is a **compatibility plan**, not proof that the compiler proposals are integrated into this downstream project.
