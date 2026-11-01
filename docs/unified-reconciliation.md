# Unified HPC hardening draft

Source foundation: PR #11 at 4e84714c727416207bfb0c0722fa78028bb18171.
The candidate preserves PR #11's strict numeric and risk validations, copies
PR #12's independent extreme-mean fixture, and imports PR #14's batch
shape and Welford index checks. The overlapping older mean implementation
does not override current PR #11's additional result-finiteness guard.

Four compiler-backed scalar fixtures are specified:
scalar_smoke (4), scalar_overflow_atomic (7), mean_extremes (5),
batch_shape (5). The mock verifier self-test is not Oreslang execution.

Evidence required, in order: source-org runner admission with real steps;
Rust format, tests and Clippy; exact-head pinned Oreslang scalar guests;
compiler-backed 23-row Region risk verification and no rejected mock venue
calls; then actor ownership and hardware-specific tests. No claim of runtime,
guest or device correctness follows from source inspection or a host model.

Keep all overlapping drafts open for independent review; never count their
overlapping patches as separate merges. No external organization source
mirroring without distinct authorization.
