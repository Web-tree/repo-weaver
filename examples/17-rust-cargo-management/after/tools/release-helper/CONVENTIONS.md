# Release Helper Conventions

- Prefer `clap` derive macros for CLI argument parsing.
- Keep JSON payloads typed via `serde` structs.
- Run `cargo fmt --check` and `cargo clippy` before release.
