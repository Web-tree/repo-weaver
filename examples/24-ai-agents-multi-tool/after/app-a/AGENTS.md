# app-a

TypeScript on Node 20.

This repository follows the [AGENTS.md open standard](https://agents.md/) — every agent that reads it (Claude Code, Cursor, Codex CLI, GitHub Copilot, Gemini CLI, Aider, Windsurf, and others) should respect the conventions below.

## Conventions

- Small, reviewable PRs. Ship the smallest vertical slice that works.
- Tests must fail before implementation lands.
- No ad-hoc scripts in the repo root; put them in `scripts/`.

## Tool-specific configs

The following tool-specific files are generated from the same source of truth and must not diverge from this file's conventions:

- `CLAUDE.md` — Claude Code
- `.cursor/rules/project.mdc` — Cursor
