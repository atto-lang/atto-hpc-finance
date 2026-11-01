#!/usr/bin/env bash
# Compiler-backed scalar finance gate; never substitutes host/Rust output.
set -euo pipefail
fail() { printf 'HPC_GUEST_NOT_CERTIFIED: %s\n' "$*" >&2; exit 65; }
root="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
: "${EXPECTED_SOURCE_SHA:?Set reviewed source commit (40 hex)}"
: "${EXPECTED_SOURCE_TREE:?Set reviewed source root tree (40 hex)}"
: "${EXPECTED_COMPILER_SHA256:?Set reviewed compiler binary digest (64 hex)}"
: "${ORESLANG_BIN:?Set absolute, reviewed Oreslang compiler/runtime executable}"
: "${HPC_EVIDENCE_DIR:?Set absolute path outside the source checkout}"
[[ "$EXPECTED_SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]] || fail "invalid expected source commit"
[[ "$EXPECTED_SOURCE_TREE" =~ ^[0-9a-f]{40}$ ]] || fail "invalid expected source tree"
[[ "$EXPECTED_COMPILER_SHA256" =~ ^[0-9a-f]{64}$ ]] || fail "invalid compiler digest"
[[ "$ORESLANG_BIN" = /* && -f "$ORESLANG_BIN" && -x "$ORESLANG_BIN" && ! -L "$ORESLANG_BIN" ]] || fail "missing/symlinked compiler binary"
[[ "$HPC_EVIDENCE_DIR" = /* ]] || fail "evidence directory must be absolute"
[[ -x /usr/bin/git && -x /usr/bin/sha256sum && -x /usr/bin/timeout && -x /usr/bin/cmp ]] || fail "trusted Linux tooling unavailable"
guest_timeout="${HPC_GUEST_TIMEOUT_SECONDS:-60}"
[[ "$guest_timeout" =~ ^[1-9][0-9]{0,2}$ ]] && ((guest_timeout <= 300)) || fail "invalid guest timeout (1..300 seconds)"
source_sha="$(/usr/bin/git -C "$root" rev-parse --verify HEAD)"
source_tree="$(/usr/bin/git -C "$root" rev-parse --verify 'HEAD^{tree}')"
[[ "$source_sha" = "$EXPECTED_SOURCE_SHA" ]] || fail "source commit differs from reviewed pin"
[[ "$source_tree" = "$EXPECTED_SOURCE_TREE" ]] || fail "source tree differs from reviewed pin"
[[ -z "$(/usr/bin/git -C "$root" status --porcelain --untracked-files=all)" ]] || fail "source checkout is dirty"
compiler_sha="$(/usr/bin/sha256sum "$ORESLANG_BIN")"
compiler_sha="${compiler_sha%% *}"
[[ "$compiler_sha" = "$EXPECTED_COMPILER_SHA256" ]] || fail "compiler digest differs from reviewed pin"
mkdir -p -- "$HPC_EVIDENCE_DIR"
evidence_root="$(cd -- "$HPC_EVIDENCE_DIR" && pwd -P)"
case "$evidence_root/" in "$root/"*) fail "evidence inside source checkout" ;; esac
out="$(mktemp -d "$evidence_root/hpc-guest.XXXXXXXX")"
receipt="$out/fixture-receipts.tsv"
printf 'fixture\tgit_blob\texpected_sha256\tstdout_sha256\tstderr_sha256\n' > "$receipt"
for test in scalar_smoke scalar_overflow_atomic mean_extremes batch_shape scalar_error_matrix risk_gate_scalar_23; do
  stdout="$out/$test.stdout"
  stderr="$out/$test.stderr"
  case "$test" in
    scalar_smoke) expected='true:true:true:true' ;;
    scalar_overflow_atomic) expected='true:true:true:true:true:true:true' ;;
    mean_extremes|batch_shape) expected='true:true:true:true:true' ;;
    scalar_error_matrix) expected='true:true:true:true:true:true:true:true:true:true:true:true' ;;
    risk_gate_scalar_23) expected='true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true:true' ;;
  esac
  fixture="$root/tests/$test.ores"
  [[ -f "$fixture" && ! -L "$fixture" ]] || fail "missing or symlinked guest fixture $test"
  /usr/bin/git -C "$root" ls-files --error-unmatch -- "tests/$test.ores" >/dev/null || fail "untracked guest fixture $test"
  fixture_blob="$(/usr/bin/git -C "$root" hash-object -- "tests/$test.ores")"
  printf '%s' "$expected" > "$out/$test.expected"
  if (cd "$root" && /usr/bin/timeout -k 5s "${guest_timeout}s" "$ORESLANG_BIN" "$fixture") >"$stdout" 2>"$stderr"; then
    :
  else
    printf 'HPC_GUEST_FAILED: %s; evidence=%s\n' "$test" "$out" >&2
    exit 1
  fi
  [[ ! -s "$stderr" ]] || fail "unexpected $test stderr; evidence=$out"
  /usr/bin/cmp -s -- "$out/$test.expected" "$stdout" || fail "unexpected $test stdout; evidence=$out"
  fixture_expected_sha="$(/usr/bin/sha256sum "$out/$test.expected")"; fixture_expected_sha="${fixture_expected_sha%% *}"
  guest_stdout_sha="$(/usr/bin/sha256sum "$stdout")"; guest_stdout_sha="${guest_stdout_sha%% *}"
  guest_stderr_sha="$(/usr/bin/sha256sum "$stderr")"; guest_stderr_sha="${guest_stderr_sha%% *}"
  printf '%s\t%s\t%s\t%s\t%s\n' "$test" "$fixture_blob" "$fixture_expected_sha" "$guest_stdout_sha" "$guest_stderr_sha" >> "$receipt"
  printf 'HPC_GUEST_CASE=PASS %s\n' "$test"
done
new_sha="$(/usr/bin/sha256sum "$ORESLANG_BIN")"; new_sha="${new_sha%% *}"
[[ "$new_sha" = "$EXPECTED_COMPILER_SHA256" ]] || fail "compiler binary changed during test"
[[ "$(/usr/bin/git -C "$root" rev-parse --verify HEAD)" = "$EXPECTED_SOURCE_SHA" ]] || fail "source commit changed"
[[ "$(/usr/bin/git -C "$root" rev-parse --verify 'HEAD^{tree}')" = "$EXPECTED_SOURCE_TREE" ]] || fail "source tree changed"
[[ -z "$(/usr/bin/git -C "$root" status --porcelain --untracked-files=all)" ]] || fail "source checkout changed"
printf 'HPC_GUEST_SOURCE_SHA=%s\n' "$source_sha"
printf 'HPC_GUEST_SOURCE_TREE=%s\n' "$source_tree"
printf 'HPC_GUEST_COMPILER_SHA256=%s\n' "$compiler_sha"
receipt_sha="$(/usr/bin/sha256sum "$receipt")"; receipt_sha="${receipt_sha%% *}"
printf 'HPC_GUEST_RECEIPT_SHA256=%s\n' "$receipt_sha"
printf 'HPC_GUEST_EVIDENCE_DIR=%s\n' "$out"
printf 'HPC_GUEST_STATUS=EXACT_HEAD_SCALAR_ONLY\n'
