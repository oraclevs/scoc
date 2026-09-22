#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

with_jc_diff=false
case "${1:-}" in
  "") ;;
  --with-jc-diff) with_jc_diff=true ;;
  *) echo "usage: $0 [--with-jc-diff]" >&2; exit 2 ;;
esac

for tool in cargo python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "required verification tool is unavailable: $tool" >&2
    exit 2
  fi
done

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
python3 -m py_compile compatibility/*.py
python3 -m unittest discover -s compatibility -p 'test_*.py' -v
python3 compatibility/check_catalog.py
python3 compatibility/static_rust_check.py
python3 compatibility/check_licenses.py

if $with_jc_diff; then
  python3 compatibility/diff.py --all-fixtures
fi
