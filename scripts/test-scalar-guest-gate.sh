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
  */mean_extremes.ores) printf 'true:true:true:true:true' ;;
  */batch_shape.ores) printf 'true:true:true:true:true:true:true' ;;
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
# A dirty source checkout must be rejected without testing it.
printf 'modified\n' >> "$tmp/source/tests/scalar_smoke.ores"
set +e
env "${common[@]}" bash "$gate" >"$tmp/bad.log" 2>&1
status=$?
set -e
[[ "$status" -eq 65 ]] && grep -q 'source checkout is dirty' "$tmp/bad.log"
printf 'HPC_SCALAR_GATE_SELFTEST=PASS (MOCK_ONLY_NOT_ORESLANG)\n'
