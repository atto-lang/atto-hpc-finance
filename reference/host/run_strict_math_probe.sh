#!/usr/bin/env bash
# Native HOST demonstration only; never Oreslang guest/backend evidence.
set -euo pipefail
root="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
compiler="${CC:-clang}"
command -v "$compiler" >/dev/null || { echo 'HOST_MATH_PROBE_NO_COMPILER' >&2; exit 65; }
tmp="$(mktemp -d "${TMPDIR:-/tmp}/hpc-math-host.XXXXXXXX")"
trap 'rm -rf -- "$tmp"' EXIT
"$compiler" --version | head -1
"$compiler" -O2 -Wall -Wextra -Werror -fno-fast-math "$root/strict_math_probe.c" -o "$tmp/strict"
"$compiler" -O2 -Wall -Wextra -Werror -ffast-math "$root/strict_math_probe.c" -o "$tmp/fast"
strict="$($tmp/strict)"
relaxed="$($tmp/fast)"
[[ "$strict" == 0 ]] || { echo 'HOST_STRICT_MATH_UNSAFE' >&2; exit 1; }
[[ "$relaxed" == 0 || "$relaxed" == 1 ]] || { echo 'HOST_FAST_MATH_OUTPUT_INVALID' >&2; exit 1; }
printf 'HOST_STRICT_AUTHORIZED=%s\n' "$strict"
printf 'HOST_FAST_MATH_AUTHORIZED=%s\n' "$relaxed"
printf 'HOST_MATH_SCOPE=HOST_C_ONLY_NOT_ORESLANG\n'
