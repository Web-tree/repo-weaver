# app-b — Gemini CLI context

Go 1.23.

Shared conventions live in `AGENTS.md`. Gemini CLI loads `GEMINI.md` files hierarchically as context; adjust `.gemini/settings.json` if you want a different filename.

## Conventions

- Errors are returned, never ignored — `_ = err` is a review-blocker.
- Interfaces live with their consumers, not their implementers.
- No `init()` functions; construct explicitly in `main`.
