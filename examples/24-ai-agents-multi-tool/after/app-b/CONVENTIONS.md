# app-b — Conventions (Aider)

Go 1.23.

Aider loads this file automatically when `.aider.conf.yml` contains `read: CONVENTIONS.md`. Shared conventions live in `AGENTS.md` — this file restates them so the two stay in sync.

## Conventions

- Errors are returned, never ignored — `_ = err` is a review-blocker.
- Interfaces live with their consumers, not their implementers.
- No `init()` functions; construct explicitly in `main`.
