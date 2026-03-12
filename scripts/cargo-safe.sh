#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -n "${KEYMOUSE_CARGO_TARGET_DIR:-}" ]]; then
  mkdir -p "${KEYMOUSE_CARGO_TARGET_DIR}"
  export CARGO_TARGET_DIR="${KEYMOUSE_CARGO_TARGET_DIR}"
elif [[ "$(uname -s)" == "Darwin" ]]; then
  :
elif [[ "$(uname -s)" =~ MINGW|MSYS|CYGWIN ]]; then
  :
fi

cd "${repo_root}"
exec cargo "$@"
