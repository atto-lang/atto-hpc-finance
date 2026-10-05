# Architecture

## 1. Separation of concerns

`oreslang-hpc-finance` sits above the Oreslang runtime.

The language/runtime should provide scheduling, actors, isolation, ownership/capabilities, arrays/buffers, task placement, and eventually accelerator hooks. This library supplies finance-domain data models and algorithms.

Finance-specific semantics must not become VM/compiler intrinsics unless a primitive proves broadly useful outside finance.

## 2. Actor plane vs compute plane

### Actor plane

Actors own long-lived mutable resources and external effects:

- market-data connections;
- venue/session state;
- clocks and timers;
- strategy state machines;
- order routing;
- account/portfolio state;
- risk limits;
- supervision/restarts;
- persistence and checkpoints.

Each actor still obeys the Oreslang single-execution-lease rule: at most one carrier thread executes an actor at a time. Actor scheduling should preserve the existing cache-locality/home-worker preference where possible.

### Compute plane

Dense work runs as pure or explicitly state-passing kernels:

- returns and transforms;
- rolling reductions;
- covariance/correlation;
- pricing batches;
- Monte Carlo paths;
- Greeks;
- portfolio aggregation;
- optimization primitives;
- order-book scans.

A compute kernel should not perform network I/O, sleep, block on actor mailboxes, or call `await`. It should consume explicit inputs and produce explicit outputs.

This distinction lets OresVM schedule actors fairly while large numerical batches can be mapped to CPU workers, SIMD lanes, or GPU devices without changing finance semantics.

## 3. Data-oriented layout

Hot-path market and portfolio data should prefer columnar/SoA form.

Instead of:

```text
Bar { ts, open, high, low, close, volume }
Bar { ts, open, high, low, close, volume }
...
```

prefer:

```text
BarColumns {
  ts[]     = ...
  open[]   = ...
  high[]   = ...
  low[]    = ...
  close[]  = ...
  volume[] = ...
}
```

Benefits:

- sequential loads;
- fewer cache lines for kernels that use only a subset of columns;
- easier vectorization;
- straightforward bulk transfer to accelerators;
- simpler zero-copy slicing;
- predictable memory footprints.

Object-rich APIs may exist at boundaries for ergonomics, but they should lower into dense batches before hot numerical loops.

## 4. Batches and ownership

A recommended ingestion path is:

```text
FeedActor
  -> normalize
  -> append to owned ring/segment
  -> freeze bounded batch/view
  -> ComputeActor or task kernel
  -> result batch
  -> StrategyActor / RiskActor
```

Rules:

- only the owning actor mutates its live ingest buffer;
- published compute batches are immutable or uniquely borrowed for the duration of a kernel;
- output buffers have a single writer;
- a kernel declares the data it reads and writes;
- cross-actor transfer uses handles/views compatible with Oreslang ownership rules, not VM implementation objects.

This aligns with the broader Oreslang DOP/ECS direction: identity/orchestration stays separate from densely stored data and stateless systems/kernels.

## 5. Scheduler interaction

Long-running finance kernels must not accidentally monopolize an actor turn.

The intended execution choices are:

1. small kernel: execute inline inside the current actor reduction budget;
2. medium/large CPU kernel: submit as an analyzable compute task and yield the actor;
3. accelerator kernel: submit to the accelerator execution layer and await the resulting `Awaitable<T>`.

An awaited compute submission must follow normal Oreslang semantics: suspension stores the continuation, releases the actor execution lease, unwinds the current stack, and resumes only through scheduler dispatch.

No completion callback may resume actor code inline.

## 6. Placement contract

The finance API should be device-neutral. The execution layer may eventually attach metadata equivalent to:

```text
reads:  [prices, volumes]
writes: [signals]
parallel: true
device_preference: auto | cpu | gpu
determinism: strict | reproducible | relaxed
```

This metadata is execution policy, not finance-domain meaning.

CPU remains the correctness baseline. GPU/SIMD implementations must match the documented numerical contract for the selected determinism mode.

## 7. Deterministic trading/backtesting

Backtests and simulations must make time/order explicit:

- monotonic simulation clock;
- source/venue sequence number;
- stable tie-break sequence for equal timestamps;
- explicit latency model;
- explicit fill model;
- seeded PRNG streams;
- no wall-clock reads inside kernels;
- no unordered-map iteration as an event-ordering mechanism.

A replay from the same input + config + seed should produce the same logical order stream under strict determinism.

## 8. Isolation and untrusted strategies

Untrusted strategies may run as untrusted actors in the secondary/further Graal isolate, while trusted shared actors and trusted isoactors remain in the primary isolate according to the OresVM topology.

The finance library must not hand untrusted code raw runtime objects, native addresses, device handles, or mutable buffers owned by another actor. Accelerator access should pass through capability-checked library/runtime APIs.

## 9. Correctness before acceleration

Every accelerated kernel should have:

- a scalar/reference implementation;
- exact edge-case semantics;
- deterministic fixtures;
- property tests where applicable;
- differential tests against the reference oracle;
- benchmark inputs that are separated from correctness fixtures.

The initial Rust oracle under `reference/rust` pins these semantics without making Rust part of the public Oreslang API.
