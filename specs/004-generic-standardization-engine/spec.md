# Feature Specification: Generic Repository Standardization Engine

**Feature Branch**: `004-generic-standardization-engine`
**Created**: 2026-05-29
**Status**: Draft
**Input**: Brainstorm — reposition repo-weaver from a webtree-specific tool into a generic, broadly-usable repository-standardization engine. Ship small orthogonal *building-block* primitives; let standards be authored as *parameterized modules* (Terraform-style) that consumers adopt with `rw add module @org/repo` and converge with `rw plan` / `rw apply`. webtree-dev-standards-k8s becomes the *first* such module, proving the engine is generic.

## Summary

repo-weaver becomes **"Terraform for repositories."** A *module* declares a desired repository state via composable, idempotent *ensures*; consumers adopt it, select features through an onboarding wizard, and converge their repo to the standard. Updating a module and re-applying propagates the change to every consuming repo. The low-level work is done by *plugins* (WASM/WIT today); the core orchestrates; modules are pure declarative composition. Nothing organization-specific lives in the core — webtree-dev-standards-k8s is just the first published module.

### Three-tier architecture

| Tier | Role | Sourcing |
|---|---|---|
| **Plugins** | Low-level workers performing the actual operations (git, terraform, npm, MCP-server registration, AI-agent skills, …) | WASM/WIT today; other runtimes possible later |
| **Core engine** | Resolves modules + plugins, runs onboarding, computes `plan`, executes `apply`, tracks state + lock | built-in (Rust) |
| **Modules** | Parameterized *standards*: inputs + defaults, feature toggles, ensures, may bundle plugins | git / local / inline |

Strict boundary: **modules declare the "what"; plugins (and built-in primitives) do the "how."** Modules never reimplement low-level logic.

## Clarifications

### Session 2026-05-29

- Q: Is this webtree-specific or generic? → A: **Generic.** repo-weaver is a broadly-usable standardization engine; webtree-dev-standards-k8s is merely the first consumer module. Rules must be expressed generically, never in dev-standards-only terms.
- Q: Where does convergence logic live — module or app? → A: **Module-owned, parameterized like Terraform modules.** A module ships defaults that consumers can override. Updating the module + re-applying propagates to all consumers.
- Q: Conflict policy when upstream and local both changed an owned region? → A: **Plan is explicit.** Every change a module will apply is shown to the user before it happens; the user can override (opt out / pin / customize) anything they disagree with. Never silently overwrite. Generic rule, not standard-specific.
- Q: How do updates land across many repos? → A: Each repo's `weaver.yaml` records which module(s) it uses plus enabled/disabled **feature sets**. `rw apply` converges per that config. An **interactive onboarding** on `rw add module` lets the user choose features (e.g. backend/Hono, Zero cache, Zitadel client) and writes the selections into `weaver.yaml`.
- Q: What is a "plugin"? → A: A WASM/WIT unit (other runtimes possible later) that performs domain work — git, terraform, npm, adding MCP servers, adding AI-agent skills, etc. A plugin may be sourced as **bundled in a module**, **external**, **local in the repo**, or **inline** (declared directly in `weaver.yaml`).

### Locked defaults (unless revised)

- **Lockfile (`​.rw/weaver.lock`) is committed to git** (reproducible across a team); `​.rw/answers.yaml` and `​.rw/state.yaml` stay gitignored.
- **v1 module/plugin sources are raw git URLs**; `@org/repo` shorthand is later sugar over a namespace resolver.
- **`ensure.yaml.key`** ships key-path edits with a *warn-on-reformat* fallback if a comment-preserving YAML editor is not yet available.
- **AI-assisted conflict resolution and named profiles/presets are deferred** to post-v1. (Feature toggles + onboarding cover most of the "preset" need.)

## Verified current state & the delta

Grounding from a source-level audit (file:line as found). The desired flow is largely *designed* (PRD §6, examples 18–29, `contracts/cli.md`) but *unimplemented*.

| Desired capability | Status today | Evidence |
|---|---|---|
| `rw add module @org/repo` | Absent (`rw module` has only list/update) | `crates/cli/src/main.rs`; `module.rs` |
| Module's own ensures run on consumers | **Broken** — `ModuleManifest.ensures` parsed but never executed; only app-level ensures run | `apply.rs`; `config.rs` |
| Generic primitives (file/section/json/yaml/plugin) | Absent — only `ensure.npm.*` deserializes | `config.rs` |
| Plan showing "what does NOT conform" | Absent — `PlannedChange` defined but never populated | `plan.rs`; `apply.rs` |
| Brownfield convergence | Absent — drift fires only on files rw already wrote | `state.rs` |
| Lockfile for safe updates | **Dead code** — every call site passes `None`; cache keyed by symbolic ref, so floating `main` never re-resolves | `module.rs` |
| Examples 18–29 (incl. AGENTS.md section, #23) | Design-only; excluded from the runner (only 01–17 registered) | `tests/.../test-suite.yaml` |

**Conclusion:** build-the-designed-thing + wire-the-dead-code, not a redesign. The core is already split (core/ops/cli) and PRD §12 already calls webtree only the "first target."

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Adopt a standard with guided onboarding (Priority: P1)

As a developer with an existing repo, I want to adopt an organization's standard with one command and a few choices, so my repo conforms without hand-editing config.

**Independent Test**: In a brownfield repo, run `rw add module <git-url>`, answer the feature prompts, and verify `weaver.yaml` records the module + selected features and `.rw/weaver.lock` pins the resolved commit.

**Acceptance Scenarios**:

1. **Given** a repo with no `weaver.yaml`, **When** I run `rw add module <url>`, **Then** the module is resolved and pinned, an interactive wizard prompts for the module's declared features/inputs, and my selections are written to `weaver.yaml`.
2. **Given** I selected "no backend", **When** onboarding completes, **Then** backend-gated ensures are recorded as disabled and will not apply.
3. **Given** onboarding finished, **When** I re-run `rw add module <url>`, **Then** it is idempotent (updates the entry, does not duplicate it).

### User Story 2 — See exactly what will change before it changes (Priority: P1)

As a repo owner, I want `rw plan` to show every change a module will make, so I can review and trust it.

**Independent Test**: With a module adopted, run `rw plan` on a non-conforming repo and verify a structured, per-ensure, colored diff is printed and a non-zero detailed exit code is returned.

**Acceptance Scenarios**:

1. **Given** a repo missing a required file/section/key, **When** I run `rw plan`, **Then** each non-conformance is listed as an explicit add/change/remove with a before/after preview.
2. **Given** a fully-conforming repo, **When** I run `rw plan`, **Then** it reports "converged" with exit code 0.
3. **Given** `--detailed-exitcode`, **When** changes or drift exist, **Then** exit code 2 is returned (Terraform convention).

### User Story 3 — Converge idempotently without clobbering (Priority: P1)

As a repo owner, I want `rw apply` to converge my repo to the standard while never destroying content it does not own.

**Independent Test**: Apply a module that owns a marked section of `AGENTS.md`; edit a sibling line outside the marked region; re-apply; verify the sibling edit survives and re-apply is a no-op for the owned region.

**Acceptance Scenarios**:

1. **Given** an owned region is missing, **When** I run `rw apply`, **Then** it is authored (created) and recorded in the lock.
2. **Given** an owned region matches the last-authored hash and upstream changed it, **When** I apply, **Then** it is safely updated.
3. **Given** I edited content *outside* any owned region, **When** I apply, **Then** my edit is preserved byte-for-byte.
4. **Given** both upstream and I changed the *same* owned region, **When** I plan/apply, **Then** the conflict is surfaced explicitly and not silently overwritten; I can resolve via an override.

### User Story 4 — Override what I disagree with (Priority: P2)

As a repo owner, I want to override any change a module proposes, so I retain final control.

**Independent Test**: Set `enabled: false` (or an input override) for one ensure/feature in `weaver.yaml`; verify `plan`/`apply` skip or alter exactly that item and nothing else.

**Acceptance Scenarios**:

1. **Given** a feature toggle set to off, **When** I apply, **Then** that feature's ensures do not run.
2. **Given** an input overridden in `weaver.yaml`, **When** I apply, **Then** the overriding value is used instead of the module default.
3. **Given** a single ensure opted out, **When** I apply, **Then** all other ensures still converge.

### User Story 5 — Update the standard, propagate to all repos (Priority: P2)

As a standard maintainer, I want to publish a change once and have every consuming repo pick it up by re-applying.

**Independent Test**: Bump a module ref; in a consuming repo run `rw module update <name>` then `rw apply`; verify the lock re-resolves and the new desired state converges.

**Acceptance Scenarios**:

1. **Given** a new module ref, **When** I run `rw module update <name>`, **Then** `.rw/weaver.lock` re-resolves to the new commit and `weaver.yaml` comments are preserved.
2. **Given** the updated lock, **When** I run `rw apply`, **Then** the plan shows the delta introduced by the standard update and converges it.

### User Story 6 — Author a standard purely from primitives (Priority: P1, proof of generality)

As a platform engineer, I want to express my organization's standard as a module composed only of generic primitives, with zero changes to repo-weaver's core.

**Independent Test**: Author webtree-dev-standards-k8s as a module whose `ensures` use only `ensure.file.exists`, `ensure.text.section`, `ensure.json.key`/`yaml.key`, feature-gated `when:` conditions, and inherited `check.*`; verify a consumer can adopt and converge it.

**Acceptance Scenarios**:

1. **Given** a module manifest using only generic primitives, **When** a consumer adopts and applies it, **Then** the repo conforms (e.g. `AGENTS.md` standards section present, marketplace/MCP JSON entries present) with no core code specific to the module.
2. **Given** feature toggles (backend, zero cache, zitadel client), **When** a consumer disables one, **Then** the corresponding ensures are skipped.

## Requirements *(mandatory)*

### Functional Requirements — Engine & commands

- **FR-001**: The system MUST provide `rw add module <source>` that resolves a module, pins its commit in `.rw/weaver.lock`, records the module in `weaver.yaml`, and launches interactive onboarding for the module's declared inputs/features.
- **FR-002**: Onboarding MUST be driven by the module manifest's declared inputs and feature toggles, MUST persist selections into `weaver.yaml`, and MUST be re-runnable idempotently.
- **FR-003**: `rw plan` MUST compute desired state from the consumed module(s) (running each ensure's read-only `plan()`), classify each item against the lock, and render an explicit per-ensure diff (add/change/remove/drift). It MUST NOT mutate the repo.
- **FR-004**: `rw apply` MUST converge each ensure idempotently and atomically, author/update only owned regions, preserve non-owned content byte-for-byte, surface conflicts explicitly, and update the lock's region hashes.
- **FR-005**: Exit codes MUST follow: 0 converged; 2 changes/drift (under `--detailed-exitcode`); 1 user error; 3 system error; 4 plugin error. (Replaces the current string-match exit-code hack.)
- **FR-006**: Module-manifest `ensures` and `checks` MUST execute on consumers (the module *is* the standard). This is a behavioral change from today.
- **FR-007**: `rw module update <name> [--ref X]` MUST re-resolve and rewrite the lock while preserving `weaver.yaml` comments/formatting.
- **FR-008**: `.rw/weaver.lock` MUST be constructed, written, and **read** (the resolver MUST stop passing `None`); the module/plugin cache MUST be keyed by **resolved commit**, not symbolic ref.

### Functional Requirements — Module model

- **FR-010**: A module manifest MUST support typed **inputs** with **defaults**, overridable per-repo in `weaver.yaml` (Terraform `variable` semantics).
- **FR-011**: A module MUST support **feature toggles** (typed inputs, bool/enum) selectable during onboarding.
- **FR-012**: Ensures MUST support a `when:` condition referencing inputs/features so they apply conditionally (e.g. `when: features.backend`).
- **FR-013**: A consumer MUST be able to override any module input and to opt a specific ensure out (`enabled: false`) in `weaver.yaml`; overrides MUST affect only the targeted item.
- **FR-014**: A module MAY declare the plugins it requires; the engine MUST resolve/load them before running the module's ensures.

### Functional Requirements — Building-block primitives

All primitives MUST be **idempotent** and **region/key-scoped**, never clobbering content they did not author.

- **FR-020**: `ensure.file.exists` — assert/create a file, optionally seeded from a template.
- **FR-021**: `ensure.text.section` — converge ONE named region in a text/markdown file. Default selector = **block markers** (`<!-- rw start NAME --> … <!-- rw end NAME -->`); markdown **heading-path** selector offered as sugar. `on_exists`: `update | append_only | skip`. Content outside the region preserved byte-for-byte.
- **FR-022**: `ensure.json.key` — structurally insert/merge a key-path/section in a JSON file, preserving siblings (covers Claude marketplace entries, MCP-server registration). `ensure.npm.{script,dep,devDep,engine}` MUST be re-expressed as thin presets over this.
- **FR-023**: `ensure.yaml.key` — same for YAML (warn-on-reformat fallback per locked defaults).
- **FR-024**: `check.command` — shell command assertion with `expect` exit code and optional `stdout_contains`/regex and `remediate`.
- **FR-025**: Declarative read-only mirrors — `check.{file,section,key,plugin}` — for CI drift gating.
- **FR-026**: All ensures MUST be dispatched through a single unified `Ensure` trait (`plan()` → `PlannedChange`, `apply()`), unifying today's two divergent ensure enums.

### Functional Requirements — Plugins

- **FR-030**: Plugins MUST be loadable from four sources: **bundled in a module**, **external** (remote/registry), **local** in the repo, and **inline** (declared in `weaver.yaml`).
- **FR-031**: A second WIT world `ensure-provider` MUST be defined, exporting `plan`/`apply`/`check` and importing host capabilities (process exec, scoped fs), alongside the existing `secrets` world.
- **FR-032**: Plugins MUST be sandboxed with explicit capability grants (WIT-defined), consistent with the existing plugin security model.
- **FR-033**: Plugin domains in scope as demonstrators: git, terraform, npm, MCP-server registration, AI-agent skill installation. (Each ultimately composes the file/json/yaml primitives.)

### State & propagation

- **FR-040**: File state MUST track **sub-file ownership** (owned regions and owned key-paths), not only whole-file checksums, so partial-edit primitives can detect their own state on brownfield repos.
- **FR-041**: The conflict classifier MUST distinguish: (a) missing → author; (b) owned-region matches last hash & upstream changed → safe update; (c) user-modified only → leave alone; (d) both changed → surface conflict (default: stop-with-markers for that region, continue others), overridable per region.
- **FR-042**: `.rw/weaver.lock` MUST record, per module: source, symbolic ref, resolved commit, integrity hash, inputs snapshot, and the list of authored paths/regions with their hashes.

### Key entities

- **Module**: a published, versioned standard. Inputs (+defaults), feature toggles, ensures, checks, optional bundled plugins.
- **Ensure**: a single idempotent convergence action implementing the `Ensure` trait; owns a file/region/key.
- **Plugin**: a sandboxed worker (WASM/WIT) implementing ensures/checks for a domain; sourced bundled/external/local/inline.
- **weaver.yaml** (consumer-owned): which modules are used, feature selections, input overrides, per-ensure opt-outs. Comment-preserving.
- **.rw/weaver.lock** (committed): resolved pins + authored-region ledger.
- **.rw/answers.yaml**, **.rw/state.yaml** (gitignored): local onboarding answers and runtime state.

## Roadmap / phasing

- **Phase 0 — Foundations**: unified `Ensure` trait; merge the two ensure enums; `ensure.json.key` generic core with `npm.*` as presets; wire the dead lockfile (key cache by resolved commit); add sub-file region ownership to state.
- **Phase 1 — First primitive + flow**: `ensure.file.exists` + `ensure.text.section`; route module ensures through dispatch; `rw add module` (raw git URL); populate structured plan + typed exit codes; promote example #23 (AGENTS.md section) to the runner as the flagship proof.
- **Phase 2 — Full taxonomy + config**: `ensure.json.key`/`yaml.key` consumer-facing; feature toggles + `when:` gating; onboarding wizard; declarative `check.*` + inherited module checks; author webtree-dev-standards-k8s as a module.
- **Phase 3 — Propagation + plugins**: conflict surfacing + overrides; `rw module update` (comment-preserving) + thin fleet batch runner; second WIT world + plugin sourcing modes (external/local/inline).

## Success Criteria

- **SC-001**: A brownfield repo can adopt a standard via `rw add module` + onboarding and reach conformance via `rw apply`, with all non-owned content preserved.
- **SC-002**: Re-running `rw apply` on a conforming repo is a no-op (idempotent); editing content outside owned regions never triggers a false conflict.
- **SC-003**: webtree-dev-standards-k8s is expressible as a module using only generic primitives + feature toggles, with **zero** module-specific code in repo-weaver core.
- **SC-004**: Updating a module ref and re-applying propagates the change, shown explicitly in `plan` first.
- **SC-005**: The example suite includes a runnable test per primitive plus the end-to-end add→plan→apply→idempotent→override flow (example #23 promoted to flagship).

## Out of scope (v1)

- AI-assisted conflict resolution (`ensure.ai.patch` kept strictly as a last-resort escape hatch, not default).
- Named profiles/presets as a first-class feature (covered for now by feature toggles + onboarding).
- `@org/repo` namespace/registry resolver (raw git URLs in v1).
- Auto-opened per-repo PRs / fleet bot (a thin batch runner is the v1 ceiling; PR automation is later).
- Windows (macOS/Linux only, consistent with prior specs; WSL for Windows).
- Comment-preserving YAML *guaranteed* round-trip (warn-on-reformat fallback acceptable in v1).

## Relationship to existing specs

- Builds on **003-plugin-management-system** (resolution, lock, sandbox, WIT) — extends it with the `ensure-provider` world and the four sourcing modes.
- Supersedes the implicit webtree-coupling in earlier specs: per PRD §12 webtree is the *first target*; this spec makes "generic engine + first module" explicit. Demote k3s-nebula / dev-standards from acceptance criteria to *example consumers*.
