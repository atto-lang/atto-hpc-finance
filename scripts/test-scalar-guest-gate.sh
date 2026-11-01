#!/usr/bin/env bash
# Self-test of receipt refusal paths. Mock executable is NOT an Oreslang compiler.
set -euo pipefail
root="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/hpc-gate-selftest.XXXXXXXX")"
trap 'rm -rf -- "$tmp"' EXIT
mkdir -p "$tmp/source/scripts" "$tmp/source/tests" "$tmp/bin" "$tmp/evidence"
cp "$root/scripts/run-scalar-guest-exact-head.sh" "$tmp/source/scripts/"
printf 'placeholder\n' > "$tmp/source/tests/scalar_smoke.ores"
printf 'placeholder\n' > "$tmp/source/tests/scalar_overflow_atomic.ores"
printf 'placeholder\n' > "$tmp/source/tests/mean_extremes.ores"
printf 'placeholder\n' > "$tmp/source/tests/batch_shape.ores"
cat > "$tmp/bin/mock-cli" <<'CLI'
#!/bin/sh
case "$1" in
  */scalar_smoke.ores) printf 'true:true:true:true' ;;
  */scalar_overflow_atomic.ores) printf 'true:true:true:true:true:true:true' ;;
  */mean_extremes.ores|*/batch_shape.ores) printf 'true:true:true:true:true' ;;
  *) exit 22 ;;
esac
CLI
chmod +x "$tmp/bin/mock-cli"
git -C "$tmp/source" init -q
git -C "$tmp/source" -c user.name=local-test -c user.email=none@example.invalid add .
git -C "$tmp/source" -c user.name=local-test -c user.email=none@example.invalid commit -qm initial
source_sha="$(git -C "$tmp/source" rev-parse HEAD)"
source_tree="$(git -C "$tmp/source" rev-parse 'HEAD^{tree}')"
compiler_sha="$(sha256sum "$tmp/bin/mock-cli")"; compiler_sha="${compiler_sha%% *}"
gate="$tmp/source/scripts/run-scalar-guest-exact-head.sh"
common=("EXPECTED_SOURCE_SHA=$source_sha" "EXPECTED_SOURCE_TREE=$source_tree" "EXPECTED_COMPILER_SHA256=$compiler_sha" "ORESLANG_BIN=$tmp/bin/mock-cli" "HPC_EVIDENCE_DIR=$tmp/evidence")
env "${common[@]}" bash "$gate" > "$tmp/pass.log"
grep -qx 'HPC_GUEST_CASE=PASS scalar_smoke' "$tmp/pass.log"
grep -qx 'HPC_GUEST_CASE=PASS scalar_overflow_atomic' "$tmp/pass.log"
grep -qx 'HPC_GUEST_CASE=PASS mean_extremes' "$tmp/pass.log"
grep -qx 'HPC_GUEST_CASE=PASS batch_shape' "$tmp/pass.log"
grep -qx 'HPC_GUEST_STATUS=EXACT_HEAD_SCALAR_ONLY' "$tmp/pass.log"
evidence_path="$(sed -n 's/^HPC_GUEST_EVIDENCE_DIR=//p' "$tmp/pass.log")"
[[ -f "$evidence_path/fixture-receipts.tsv" ]]
[[ "$(wc -l < "$evidence_path/fixture-receipts.tsv")" -eq 5 ]]
# Every fixture receipt must carry a Git blob and three SHA-256 hashes.
awk -F '\t' 'NR > 1 {
  if (NF != 5 || length($2) != 40 || length($3) != 64 || length($4) != 64 || length($5) != 64) exit 1;
  checked++;
} END { if (checked != 4) exit 1; }' "$evidence_path/fixture-receipts.tsv"
receipt_sha="$(sha256sum "$evidence_path/fixture-receipts.tsv")"; receipt_sha="${receipt_sha%% *}"
grep -qx "HPC_GUEST_RECEIPT_SHA256=$receipt_sha" "$tmp/pass.log"
# A malformed identity pin must fail before any mock execution.
set +e
env "${common[@]}" EXPECTED_SOURCE_SHA="${source_sha}0" bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 65 ]] && grep -q 'invalid expected source commit' "$tmp/bad.log"
# A checksum mismatch must fail closed.
set +e
env "${common[@]}" EXPECTED_COMPILER_SHA256="${source_sha}000000000000000000000000" bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 65 ]] && grep -q 'compiler digest differs' "$tmp/bad.log"
# Wrong output must never be certified as actual guest success.
sed -i 's/true:true:true:true:true:true:true/false:false:false:false:false:false:false/' "$tmp/bin/mock-cli"
new_digest="$(sha256sum "$tmp/bin/mock-cli")"; new_digest="${new_digest%% *}"
set +e
env "${common[@]}" EXPECTED_COMPILER_SHA256="$new_digest" bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 65 ]] && grep -q 'unexpected scalar_overflow_atomic stdout' "$tmp/bad.log"
# A hung compiler process must terminate; timeout failures are not receipts.
cat > "$tmp/bin/hung-cli" <<'HUNG'
#!/bin/sh
sleep 3
printf 'true:true:true:true'
HUNG
chmod +x "$tmp/bin/hung-cli"
hung_sha="$(sha256sum "$tmp/bin/hung-cli")"; hung_sha="${hung_sha%% *}"
set +e
env "${common[@]}" EXPECTED_COMPILER_SHA256="$hung_sha" ORESLANG_BIN="$tmp/bin/hung-cli" HPC_GUEST_TIMEOUT_SECONDS=1 bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 1 ]] && grep -q 'HPC_GUEST_FAILED: scalar_smoke' "$tmp/bad.log"
# Invalid timeout configuration must fail without running guest code.
set +e
env "${common[@]}" HPC_GUEST_TIMEOUT_SECONDS=0 bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 65 ]] && grep -q 'invalid guest timeout' "$tmp/bad.log"
# A dirty source checkout must be rejected without testing it.
printf 'modified\n' >> "$tmp/source/tests/scalar_smoke.ores"
set +e
env "${common[@]}" bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 65 ]] && grep -q 'source checkout is dirty' "$tmp/bad.log"
printf 'HPC_SCALAR_GATE_SELFTEST=PASS (MOCK_ONLY_NOT_ORESLANG)\n'
