# app-c — GitHub Copilot instructions

Rust 1.92.

Shared conventions live in `AGENTS.md`. VS Code automatically loads this file and applies it to every chat request in this workspace.

## Conventions

- Prefer `?` over `.unwrap()` outside of tests.
- New dependencies require justification in the commit message.

## Copilot-specific

- Path-specific rules belong in `.github/instructions/<name>.instructions.md`.
- Reusable prompts belong in `.github/prompts/<name>.prompt.md`.
