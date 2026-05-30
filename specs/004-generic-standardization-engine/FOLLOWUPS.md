# 004 — Phase 0+1 Follow-ups

Tracked items from the final whole-implementation review. None block the Phase 0+1 merge (all 11 plan tasks delivered + reviewed; example 23 passes end-to-end). These are scoped to Phase 2/3.

## Phase 2 (do first)

- **`plan()` converged-detection for `md_section` / `from_template`.** Both ensures' `plan()` currently return non-empty `actions` unconditionally, so `rw plan --detailed-exitcode` can never exit 0 on a repo that is already converged with these ensures (violates spec SC-002 / User Story 2.2). Fix: in `plan()`, read the target and compute the would-be result (reuse `upsert_*` / render) and report changes only when the bytes differ. This should land before any user-facing `rw plan` polish.
- **`rw module add` registered example.** `ensure.file.exists` has unit tests but no registered example exercises it end-to-end (SC-005 "runnable test per primitive"). Add a small example or fold a `file.exists` ensure into example 23.
- **`describe --json` app-level ensures.** The `--json` output omits `app_config.ensures` (only `manifest.ensures` is serialized). Pre-existing for npm; now also hides the new file ensures. Add app-level ensures to the JSON object.

## Phase 3 / lower priority

- **Populate & enforce `owned_regions`.** The `FileState.owned_regions` schema (Task 4) is additive and not yet written/enforced. Phase 3 conflict classifier will populate region checksums (from `md_section`) and use them for 3-way drift detection.
- **Annotated-tag commit fidelity.** `rev_parse_remote` pins the tag-object SHA for annotated tags (clone+checkout still lands on the right commit; `resolved_commit` is documented as "object SHA"). If exact commit SHAs are needed in the lock, peel with `^{}`.
- **`md_section` v1 limitations** (documented in `crates/core/src/ensure/file.rs`): LF-only; does not skip headings inside fenced code blocks; heading `path` hierarchy not enforced (final element matched at `depth`); CRLF normalized to LF on write.
- **`weaver.yaml` comment preservation.** `rw module add`/`update` rewrite `weaver.yaml` via `serde_yml::to_writer`, dropping comments (accepted v1 per spec). A comment-preserving YAML edit would improve UX.

## Housekeeping

- **Pre-existing clippy in `crates/ops/src/npm.rs`** (`collapsible_if` ×2) predates this branch and fails `clippy --all-targets -D warnings`. Not introduced here; fix or `#[allow]` when next touching that file so CI can enforce `-D warnings`.
- **`unsafe { std::env::set_var("HOME", ...) }`** in `module.rs`'s `resolve_caches_by_commit_and_records_lock` test — single-threaded-safe today; consider injecting the cache dir to avoid the global env mutation.
