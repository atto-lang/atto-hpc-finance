# Actor → compute → trading pipeline

The first library pipeline is:

```text
validated dense batch
    |
    v
MarketDataActor
    |  returns + rolling stats + chunked moments
    v
StrategyActor
    |  signal + order-intent kernels
    v
RiskActor
    |  pre-trade risk gate
    v
ExecutionActor
    |  terminal owner of approved/rejected work
    v
venue capability / simulator (application integration)
```

## Ownership

The pipeline uses one `ValidatedFinanceWorkBatch` ownership envelope containing
preallocated logical regions. The producer must validate dimensions and
finance-domain input constraints before mailbox admission.

Each actor:

1. owns the message for its mailbox turn;
2. invokes only the compute kernels for its stage;
3. never exposes its own mutable actor state to a kernel;
4. transfers the work envelope with `rt take` when calling
   `ActorRef.send`;
5. does not retain a writable alias after transfer.

This is intentionally different from sharing a mutable "market context" object
between actors.

## Current execution floor

The current Oreslang reference interpreter in the active data-oriented compute
PR executes a direct `compute fnc` call synchronously. Therefore
`src/pipeline.ores` is the **semantic/reference pipeline**, not a claim that a
large GPU launch already releases the actor carrier.

The production lowering must preserve the same source-level kernel contract but
change the physical execution boundary:

```text
actor turn
  -> submit logical compute task
  -> Future/Awaitable
  -> actor suspends and releases execution lease
  -> CPU/GPU completion settles Future
  -> continuation is re-enqueued on the actor's scheduler domain
```

No callback may resume actor code inline.

Until that compute-submission API is present upstream, the library will not
invent a fake `compute.submit` API or hide blocking behind Java interop.

## CPU path

The current kernels already expose two levels of CPU parallelism:

- independent loops are marked `parallel` / `parallel simd`;
- variance uses explicit fixed-size Welford chunks plus a deterministic merge.

The runtime mapper can turn those contracts into coarse CPU chunk tasks rather
than scheduler work per market row. Chunk boundaries should prefer SIMD widths
and cache/NUMA locality but remain stealable.

## GPU path

The kernels use `place auto`, explicit read/write sets, and SoA layout.
Therefore a GPU variant can be selected later without changing actor messages or
finance APIs.

GPU selection must account for current region residency, transfer/writeback
bytes, launch cost, queue pressure, working-set/device-memory limits, and the
availability of a semantically compatible variant. Explicit GPU execution must
fail closed when no valid backend exists.

The deterministic `combine_moments_kernel` stays `place cpu` for now so
strict-mode reduction order is obvious.

## ExecutionActor

The bootstrap `ExecutionActor` is deliberately a terminal ownership boundary.
It does not perform ambient network I/O. A trading application should compose a
narrow venue/simulator capability behind that actor and inspect
`RiskDecisionPoint` before submitting an order.

This prevents a numerical library from silently acquiring exchange credentials
or network authority.
