#!/usr/bin/env bash
# Pledge & Crown — pre-commit hook.
# Runs local CI before every commit. No GitHub Actions exists; this is the gate.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

echo "[pre-commit] running scripts/ci.ps1..."
powershell.exe -ExecutionPolicy Bypass -File scripts/ci.ps1
