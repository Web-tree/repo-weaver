# app-b

Go 1.23.

This repository follows the [AGENTS.md open standard](https://agents.md/) — every agent that reads it (Claude Code, Cursor, Codex CLI, GitHub Copilot, Gemini CLI, Aider, Windsurf, and others) should respect the conventions below.

## Conventions

- Errors are returned, never ignored — `_ = err` is a review-blocker.
- Interfaces live with their consumers, not their implementers.
- No `init()` functions; construct explicitly in `main`.

## Tool-specific configs

The following tool-specific files are generated from the same source of truth and must not diverge from this file's conventions:

- `.github/copilot-instructions.md` — GitHub Copilot
- `.windsurfrules` — Windsurf
- `GEMINI.md` — Gemini CLI
- `CONVENTIONS.md` + `.aider.conf.yml` — Aider
