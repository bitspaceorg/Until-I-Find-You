#!/usr/bin/env bash
# CI driver. Run inside `nix develop .#ci`.
# Stages: fmt-check → lint → test → docs-check.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$here/.."

echo ">> treefmt (check)"
treefmt --fail-on-change

echo ">> clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo ">> test"
cargo nextest run --workspace

echo ">> docs"
python tools/docs/validate.py
python tools/docs/build_index.py

echo "ok"
