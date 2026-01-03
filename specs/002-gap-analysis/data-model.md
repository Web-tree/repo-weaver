# Data Model: 002-Gap Analysis

**Feature**: Gap Analysis for Repo Weaver MVP  
**Date**: 2026-01-03

---

## New Entities

### Include

Represents a glob pattern or path for loading additional config fragments.

| Field | Type | Description |
|-------|------|-------------|
| pattern | `String` | Glob pattern relative to weaver.yaml (e.g., `weaver.d/*.yaml`) |

**Relationships**: 
- One `WeaverConfig` has many `Include` patterns
- Fragments merge into parent config (maps deep-merge, arrays concat)

**Validation Rules**:
- Pattern must be valid glob syntax
- Matching no files: warn, not error
- Duplicate app names across fragments: error

---

### Check

Represents a validation command associated with an app.

| Field | Type | Description |
|-------|------|-------------|
| name | `String` | Human-readable check identifier |
| command | `String` | Shell command to execute |
| description | `Option<String>` | Optional description shown in output |

**Relationships**:
- One `AppConfig` has many `Check` definitions

**Validation Rules**:
- Command must not be empty
- Check name must be unique within app

---

### EnsureGitSubmodule

Represents a git submodule ensure operation.

| Field | Type | Description |
|-------|------|-------------|
| url | `String` | Remote repository URL |
| ref | `String` | Git ref (tag, branch, commit) to checkout |
| path | `String` | Destination path relative to app root |

**State Transitions**:
- `absent` → `present`: git submodule add + checkout
- `present` (clean) → `updated`: git checkout new ref
- `present` (dirty) → `error`: "dirty working tree" error

---

### EnsureGitClonePinned

Represents a pinned git clone (non-submodule) ensure operation.

| Field | Type | Description |
|-------|------|-------------|
| url | `String` | Remote repository URL |
| ref | `String` | Git ref to checkout |
| path | `String` | Destination path relative to app root |

**State Transitions**:
- `absent` → `present`: git clone + checkout
- `present` → `updated`: git fetch + checkout (if not dirty)

---

### EnsureNpmScript

Represents an npm script ensure operation.

| Field | Type | Description |
|-------|------|-------------|
| name | `String` | Script name (key in package.json scripts) |
| command | `String` | Script command value |

**Preconditions**:
- package.json must exist at app path

---

### EnsureAiPatch

Represents an AI-generated patch operation.

| Field | Type | Description |
|-------|------|-------------|
| prompt | `String` | AI instruction prompt |
| verify | `String` | Verification command to run after patch |
| target | `Option<String>` | Target file/dir (optional, defaults to app path) |

**State Transitions**:
- Request AI → Apply diff → Verify → Commit (success) or Rollback (failure)

---

## Modified Entities

### WeaverConfig

Add field:

| Field | Type | Description |
|-------|------|-------------|
| includes | `Vec<String>` | Glob patterns for config fragments |

### AppConfig

Add field:

| Field | Type | Description |
|-------|------|-------------|
| checks | `Vec<CheckDef>` | Validation checks for this app |

---

## Type Hierarchy

```text
Ensure (trait)
├── EnsureFolderExists
├── EnsureFileFromTemplate
├── EnsureTaskWrapper
├── EnsureGitSubmodule      [NEW]
├── EnsureGitClonePinned    [NEW]
├── EnsureNpmScript         [NEW]
└── EnsureAiPatch           [NEW]
```
