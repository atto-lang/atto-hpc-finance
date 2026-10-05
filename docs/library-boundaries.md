# Library boundaries: finance vs data science

`oreslang-hpc-finance` should compose with `oreslang-data-science`, not become a second generic numerical standard library.

## Dependency direction

```text
Oreslang core/runtime
        ^
        |
oreslang-data-science
        ^
        |
oreslang-hpc-finance
```

The dependency is one-way. The data-science library must not import finance-domain types.

## Belongs in oreslang-data-science

Once the sibling library has stable APIs, generic primitives should move there:

- mean / variance / standard deviation;
- rolling/windowed reductions;
- covariance / correlation;
- quantiles / histograms;
- generic matrix/tensor and linear-algebra kernels;
- generic random streams and sampling;
- generic optimization building blocks;
- reusable Region/chunk/tiling helpers.

The finance repository may temporarily carry scalar/reference versions while the
underlying Oreslang HPC contracts are still being proven. Extraction requires
golden/differential parity first.

## Stays in oreslang-hpc-finance

Finance semantics remain here:

- simple/log return conventions and price-domain validation;
- drawdown and finance-specific annualization;
- Sharpe/Sortino/beta/alpha and exposure/risk conventions;
- instruments, curves, pricing, Greeks;
- order books, order intents, venue/execution models;
- backtest clocks, fills, slippage and transaction costs;
- portfolio/accounting semantics;
- finance-specific Monte Carlo scenarios;
- trading actors and risk/execution orchestration.

## Core rule

A generic primitive becoming important to finance is not a reason to move it
into the Oreslang language core. Core owns the execution substrate
(`Region<T>`, actors, ownership, scheduling, task mapping, await semantics).
External libraries own algorithms and domain APIs.
