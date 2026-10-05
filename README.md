# oreslang-hpc-finance

High-performance finance, trading, quantitative research, and HPC routines for Oreslang.

This repository is intentionally **outside Oreslang core**. It is the proving ground for data-oriented kernels, actor-driven streaming pipelines, deterministic numerical routines, and future SIMD/GPU backends without coupling finance-domain APIs to the language/runtime.

## Design direction

- **Data-oriented first:** dense columnar / struct-of-arrays representations for hot market-data and portfolio paths.
- **Pure kernels first:** pricing, returns, rolling statistics, indicators, and risk calculations are deterministic functions over explicit buffers.
- **Actors at boundaries:** actors own feeds, clocks, order routing, orchestration, and mutable strategy/session state; compute kernels remain batchable and parallelizable.
- **Placement-ready:** kernels describe read/write data sets so Oreslang can later map independent work to CPU workers or GPU accelerators.
- **Reproducible finance:** explicit clocks, deterministic seeds, stable event ordering, and no hidden global state.
- **Zero-copy where safe:** pass bounded slices/views into kernels instead of materializing object graphs.
- **Core stays small:** reusable HPC primitives may eventually inform generic external libraries, but finance-specific code does not enter the language core.

The initial implementation is being bootstrapped on a feature branch after this repository-initialization commit.
