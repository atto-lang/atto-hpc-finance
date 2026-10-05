# Rust reference oracle

This crate is intentionally small and dependency-free.

It is **not** the public implementation of `oreslang-hpc-finance`. It exists to:

- pin edge-case and output-shape semantics;
- provide scalar correctness fixtures for Oreslang kernels;
- act as a differential-testing oracle for future SIMD/GPU paths;
- provide a portable baseline for benchmarking optimization work.

Public Oreslang APIs should remain finance/domain oriented and device neutral.
