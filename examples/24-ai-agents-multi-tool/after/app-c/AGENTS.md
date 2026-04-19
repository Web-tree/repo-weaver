# app-c

Rust 1.92.

This repository follows the [AGENTS.md open standard](https://agents.md/) — every agent that reads it (Claude Code, Cursor, Codex CLI, GitHub Copilot, Gemini CLI, Aider, Windsurf, and others) should respect the conventions below.

## Conventions

- Prefer `?` over `.unwrap()` outside of tests.
- New dependencies require justification in the commit message.

## Tool-specific configs

The following tool-specific files are generated from the same source of truth and must not diverge from this file's conventions:

- `CLAUDE.md` — Claude Code
- `.cursor/rules/project.mdc` — Cursor
- `.github/copilot-instructions.md` — GitHub Copilot
