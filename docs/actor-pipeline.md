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

The desired pipeline uses one uniquely owned work envelope containing
preallocated logical regions. However, **raw writable `Region<T>` is not an
actor message type today, and it should not become one by simple
whitelisting**.

The hardened actor checker deliberately fails closed for opaque nominal
capabilities. That is the correct behavior: a region may represent mutable,
NUMA-local, accelerator-resident, or otherwise runtime-managed storage, so
copying a raw region handle through a mailbox could manufacture writable aliases
or leak device authority.

Oreslang runtime issue #236 therefore tracks an explicit owned transfer
capability (exact source name still to be chosen). Its required semantics are:

1. transfer requires unique ownership and behaves like an `rt take`;
2. the sender loses writable authority after successful mailbox admission;
3. live borrows/aliases make transfer fail at compile time;
4. the receiver becomes the unique writable owner;
5. read-only sharing, if supported, uses an explicit frozen/versioned view;
6. raw native addresses and device handles never become guest-visible.

`design/pipeline.ores` shows the desired stage topology but is intentionally
outside `.oreslangc.cfg.toml` source roots until that capability exists.

This is intentionally different from sharing a mutable "market context" object
between actors.

## Current execution floor

The current Oreslang reference interpreter in the active data-oriented compute
PR executes a direct `compute fnc` call synchronously. Therefore
`design/pipeline.ores` is a **design target**, not a claim that a
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

Until that compute-submission API and unique region-transfer capability are
present upstream, the library will not invent a fake `compute.submit` API,
make raw `Region<T>` mailbox-sendable, or hide blocking behind Java interop.

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
