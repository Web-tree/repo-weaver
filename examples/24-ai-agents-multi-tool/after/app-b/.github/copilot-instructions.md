# app-b — GitHub Copilot instructions

Go 1.23.

Shared conventions live in `AGENTS.md`. VS Code automatically loads this file and applies it to every chat request in this workspace.

## Conventions

- Errors are returned, never ignored — `_ = err` is a review-blocker.
- Interfaces live with their consumers, not their implementers.
- No `init()` functions; construct explicitly in `main`.

## Copilot-specific

- Path-specific rules belong in `.github/instructions/<name>.instructions.md`.
- Reusable prompts belong in `.github/prompts/<name>.prompt.md`.
