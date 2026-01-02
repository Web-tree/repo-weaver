# UX Requirements Quality Checklist: Repo Weaver MVP

**Feature**: `001-repo-weaver-mvp`
**Created**: 2026-01-02
**Focus**: UX & Interactivity (CLI, AI Resolve, Drift)

## 1. Requirement Completeness (Interactive Flows)

- [ ] CHK001 - Are the exact prompts and options for "Drift Resolution" defined? (e.g., "Overwrite [y/N]", "Diff [d]", "AI Resolve [a]") [Completeness, Spec FR-010]
- [ ] CHK002 - Is the visual output format for `rw plan` specified (e.g., color-coded diffs, symbol prefixes)? [Completeness, FR-007]
- [ ] CHK003 - Are "success" AND "failure" messages explicitly defined for the Bootstrap User Story? [Completeness, Story 1]
- [ ] CHK004 - Is the behavior of `rw apply` defined when `weaver.yaml` is malformed (e.g., inline error reporting vs stack trace)? [Completeness, Exception]
- [ ] CHK005 - Are valid input types specified for answers (e.g., validating a string is a valid AWS region)? [Completeness, Gap]

## 2. Requirement Clarity (AI & Automation)

- [ ] CHK006 - Is the "AI Resolve" user flow clearly step-by-step defined? (e.g., "Preview patch -> Confirm -> Apply") [Clarity, FR-010]
- [ ] CHK007 - Is the "non-interactive" default behavior for AI Resolve (Fail vs Auto-accept) explicitly stated? [Clarity, FR-004a]
- [ ] CHK008 - Is "AI Resolve" failure behavior defined? (e.g., fallback to manual merge, or abort?) [Clarity, Edge Case]

## 3. Consistency (CLI Ergonomics)

- [ ] CHK009 - Do CLI flags follow a consistent naming convention? (e.g., `--non-interactive` vs `--yes` vs `--force`) [Consistency, FR-004a]
- [ ] CHK010 - Is the verbosity flag behavior (`--verbose` vs `--quiet`) consistent with the logging requirement? [Consistency, FR-014]
- [ ] CHK011 - Are prompt defaults (e.g., `[Y/n]`) consistent across all command prompts? [Consistency]

## 4. Edge Cases & Resilience

- [ ] CHK012 - Is the UX defined for when networking fails during an "AI Resolve" operation? [Coverage, Resilience]
- [ ] CHK013 - Is the behavior specified when `.rw/answers.yaml` contains invalid/corrupted data? [Coverage, Edge Case]
- [ ] CHK014 - Is the user notified if a plugin authentication (AWS SSM) fails mid-operation? [Coverage, FR-012]
- [ ] CHK015 - Is there a clear "undo" or "rollback" path defined if `rw apply` partially fails? [Coverage, Recovery]

## 5. Measurability

- [ ] CHK016 - Can "bootstraps ... in under 30 seconds" be measured with a specific "done" state (e.g., "all files written")? [Measurability, SC-001]
- [ ] CHK017 - Is the "Drift Detection" success criteria (100% detection) clear on whether it includes whitespace/formatting changes? [Measurability, SC-002]

## 6. Workflow UX (Updates & Pipelines)

- [ ] CHK018 - Is the "Update Pin" workflow defined? (e.g. Does the user manually edit weaver.yaml or run `rw update module@v2`?) [Clarity, Story 2]
- [ ] CHK019 - Is the diff presentation for a module update specified? (e.g. "Showing 3 changed files...") [Clarity, Story 2]
- [ ] CHK020 - Are pipeline step outputs (stdout/stderr) visible by default or hidden behind a flag? [Clarity, Story 3]
- [ ] CHK021 - Is the failure output for a pipeline step specific enough to debug? (e.g. "Step 'tf-plan' failed with exit code 1") [Clarity, Story 3]

## 7. Feedback & Progress

- [ ] CHK022 - Are "long-running" operations (e.g. Terraform Apply) required to show a spinner or progress bar? [Completeness, UX]
- [ ] CHK023 - Is there a clear visual distinction between "Weaver Ops" (internal) and "Child Process Output" (external tools)? [Consistency, UX]
