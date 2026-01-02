# UX Requirements Quality Checklist: Repo Weaver MVP

**Feature**: `001-repo-weaver-mvp`
**Created**: 2026-01-02
**Focus**: UX & Interactivity (CLI, AI Resolve, Drift)

## 1. Requirement Completeness (Interactive Flows)

- [x] CHK001 - Are the exact prompts and options for "Drift Resolution" defined? (e.g., "Overwrite [y/N]", "Diff [d]", "AI Resolve [a]") [Completeness, Spec FR-010]
- [x] CHK002 - Is the visual output format for `rw plan` specified (e.g., color-coded diffs, symbol prefixes)? [Completeness, FR-007]
- [x] CHK003 - Are "success" AND "failure" messages explicitly defined for the Bootstrap User Story? [Completeness, Story 1]
- [x] CHK004 - Is the behavior of `rw apply` defined when `weaver.yaml` is malformed (e.g., inline error reporting vs stack trace)? [Completeness, Exception]
- [x] CHK005 - Are valid input types specified for answers (e.g., validating a string is a valid AWS region)? [Completeness, Gap]

## 2. Requirement Clarity (AI & Automation)

- [x] CHK006 - Is the "AI Resolve" user flow clearly step-by-step defined? (e.g., "Preview patch -> Confirm -> Apply") [Clarity, FR-010]
- [x] CHK007 - Is the "non-interactive" default behavior for AI Resolve (Fail vs Auto-accept) explicitly stated? [Clarity, FR-004a]
- [x] CHK008 - Is "AI Resolve" failure behavior defined? (e.g., fallback to manual merge, or abort?) [Clarity, Edge Case]

## 3. Consistency (CLI Ergonomics)

- [x] CHK009 - Do CLI flags follow a consistent naming convention? (e.g., `--non-interactive` vs `--yes` vs `--force`) [Consistency, FR-004a]
- [x] CHK010 - Is the verbosity flag behavior (`--verbose` vs `--quiet`) consistent with the logging requirement? [Consistency, FR-014]
- [x] CHK011 - Are prompt defaults (e.g., `[Y/n]`) consistent across all command prompts? [Consistency]

## 4. Edge Cases & Resilience

- [x] CHK012 - Is the UX defined for when networking fails during an "AI Resolve" operation? [Coverage, Resilience]
- [x] CHK013 - Is the behavior specified when `.rw/answers.yaml` contains invalid/corrupted data? [Coverage, Edge Case]
- [x] CHK014 - Is the user notified if a plugin authentication (AWS SSM) fails mid-operation? [Coverage, FR-012]
- [x] CHK015 - Is there a clear "undo" or "rollback" path defined if `rw apply` partially fails? [Coverage, Recovery]

## 5. Measurability

- [x] CHK016 - Can "bootstraps ... in under 30 seconds" be measured with a specific "done" state (e.g., "all files written")? [Measurability, SC-001]
- [x] CHK017 - Is the "Drift Detection" success criteria (100% detection) clear on whether it includes whitespace/formatting changes? [Measurability, SC-002]

## 6. Workflow UX (Updates & Pipelines)

- [x] CHK018 - Is the "Update Pin" workflow defined? (e.g. Does the user manually edit weaver.yaml or run `rw update module@v2`?) [Clarity, Story 2]
- [x] CHK019 - Is the diff presentation for a module update specified? (e.g. "Showing 3 changed files...") [Clarity, Story 2]
- [x] CHK020 - Are pipeline step outputs (stdout/stderr) visible by default or hidden behind a flag? [Clarity, Story 3]
- [x] CHK021 - Is the failure output for a pipeline step specific enough to debug? (e.g. "Step 'tf-plan' failed with exit code 1") [Clarity, Story 3]

## 7. Feedback & Progress

- [x] CHK022 - Are "long-running" operations (e.g. Terraform Apply) required to show a spinner or progress bar? [Completeness, UX]
- [x] CHK023 - Is there a clear "undo" or "rollback" path defined if `rw apply` partially fails? [Consistency, UX]
