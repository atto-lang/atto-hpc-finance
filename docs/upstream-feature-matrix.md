# Upstream Oreslang feature matrix

The library intentionally targets Oreslang's active next language/runtime surface rather than freezing workarounds for today's `main`.

| Library surface | Upstream contract | Why it is used here |
| --- | --- | --- |
| closed `struct` values | oreslang-source.java PR #176 / #177 | AOT-safe market data, errors, messages; no dynamic object shapes |
| pointer-free ownership (`rt borrow/take/copy/share`, `T mut x`) | PR #186 | Java-like references with explicit mutation/ownership transfer |
| `Region<T>`, compute effects, layout and placement | PR #213 | analyzable DOP kernels and CPU/GPU-neutral logical data |
| ECS/DOP effect discipline | PR #215 | reinforces state/behavior separation and declared access sets |
| actor class + one `receive` ingress + typed `ActorRef.send` | PR #185 + hardening PR #208 | stateful trading orchestration without multiple concurrent actor entrypoints |
| actor carrier locality | PR #216 | preserves cache affinity for stateful actor turns |
| Regent-style mapper/variants | PR #222 | scalar/CPU-SIMD/GPU variants share one logical kernel contract |
| Ores-owned await/continuations | PR #167/#182/#192/#193/#232 | asynchronous compute/device completion releases actor carriers |
| native Oreslang tests | PR #233 | future replacement for the bootstrap smoke program |
| transferable region ownership + async actor compute | runtime issue #236 | required to safely connect actor mailboxes to `Region<T>` CPU/GPU tasks |

## Compatibility policy

Source under `src/` follows the intended contracts above. It is acceptable for
a file to require an upstream draft feature that has not yet reached
`oreslang-source.java/main`; the dependency is documented here rather than
papered over with Java interop or a second library API.

The scalar module is deliberately closest to today's runtime: ordinary arrays,
loops, arithmetic, `Result`, and explicit mutable reference parameters. The
`Region<T>` kernels are the heterogeneous-compute implementation of the same
finance semantics.

## Validation boundary

Before an accelerated kernel launch, the control/scalar layer validates:

- logical input/output lengths;
- finite numeric inputs;
- finance-domain preconditions such as nonzero return denominators;
- window/chunk dimensions.

Accelerated kernels therefore operate on validated data and do not allocate,
perform I/O, await, acquire actor state, or discover capabilities. This is the
same division required by the compute-safe subset in PR #213.

## No host escape hatch

This library does not use Java interop to obtain vector math, device pointers, or
thread pools. Missing numerical intrinsics/backends are an upstream/library
dependency, not a reason to bypass Oreslang's authority and isolation model.
