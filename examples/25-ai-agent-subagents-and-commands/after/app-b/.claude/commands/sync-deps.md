---
description: Resync app-b dependencies against the lockfile and report drift.
allowed-tools: Bash, Read
---

# /sync-deps

Bring app-b's dependencies back in line with the lockfile.

## Procedure

1. Detect the package manager (npm, cargo, go, uv, pip-tools).
2. Run the manager's install command against the existing lockfile — never regenerate it unless the user asks.
3. If the install reports drift (missing or extra packages), list the drift. Do not fix it; that's a separate decision.
4. Report: package manager, install duration, drift summary.
