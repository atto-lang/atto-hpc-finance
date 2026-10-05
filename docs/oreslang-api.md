# Oreslang API direction

This document describes the intended public shape. It is deliberately an API contract, not a parser conformance fixture; exact package/import spellings should track the compiler and zed-pkg conventions rather than being frozen by this bootstrap PR.

## Public design

The library should expose dense data containers and functions, while actor types stay in a separate orchestration namespace/module.

Conceptually:

```text
finance.market_data
  BarColumns
  QuoteColumns
  TradeColumns

finance.returns
  simple_returns_into(...)
  log_returns_into(...)

finance.stats
  mean(...)
  sample_variance(...)
  rolling_mean_into(...)
  rolling_variance_into(...)

finance.risk
  max_drawdown(...)
  sharpe_ratio(...)
  sortino_ratio(...)
  var(...)
  cvar(...)

finance.actors
  MarketDataActor
  StrategyActor
  RiskActor
  ExecutionActor
```

## Oreslang-specific rules

- Public finance values should use ordinary Oreslang structs/interfaces/maps/arrays; no VM-internal types.
- Hot columnar containers should have statically known fields. Open-ended metadata belongs in `Map<K,V>`, not dynamic structs.
- Passing a batch/view follows Oreslang's normal reference/handle semantics. Copies are explicit with `rt copy`; ownership transfer is explicit with `rt take`; shared ownership is explicit with `rt share`; temporary access uses `rt borrow`.
- Compute routines should declare effects/access sufficiently for task parallelism and CPU/GPU placement analysis.
- Finance kernels should not use `await` internally. Submission to a compute device may return an `Awaitable<T>`, and the calling actor may await that handle.
- Any actor-based API remains subject to actor isolation: actor code may mutate actor-owned state and runtime-owned response channels, not arbitrary external state.
- Hot-loaded strategies must satisfy their declared interface/ABI and capability manifest; finance library internals are not introspected by OresVM.

## Example orchestration shape

Illustrative only:

```text
MarketDataActor
  receives venue bytes
  normalizes into owned BarColumns/ring segment
  freezes a bounded read view
  submits returns/indicator/risk kernels
  awaits only when work is actually asynchronous
  sends immutable result handles to StrategyActor

StrategyActor
  receives signals + current portfolio snapshot
  mutates only strategy-owned state
  emits an order intent

RiskActor
  evaluates order intent against explicit portfolio/risk inputs
  emits approved/rejected intent

ExecutionActor
  performs venue I/O and owns exchange-session state
```

This keeps the scheduler/actor model responsible for concurrency and failure isolation, while dense finance math remains analyzable and data-oriented.
