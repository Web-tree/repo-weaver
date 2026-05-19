# Rust Code Quality Validation Checklist

**Purpose:** Detect + fix common LLM-generated Rust anti-patterns. Run before declaring Rust code complete.

**How to use:**
- Apply top-to-bottom on every diff with `.rs` files.
- Each rule: **anti-pattern → fix → detection** (clippy lint or grep).
- Fail check → fix, no suppress. Must suppress → add comment explain invariant.
- Hard stop: 3 successive `cargo check` iterations fail same module → escalate human. Architecture wrong, not syntax.

---

## 0. Pre-flight: Tooling Gate

Before review, project must have these on. Missing → add.

### `clippy.toml`
```toml
cognitive-complexity-threshold = 15
type-complexity-threshold = 200
too-many-arguments-threshold = 5
```

### Workspace `Cargo.toml` `[workspace.lints.clippy]`
```toml
# Correctness gates
await_holding_lock = "deny"
await_holding_refcell_ref = "deny"
significant_drop_in_scrutinee = "warn"
let_underscore_must_use = "warn"
let_underscore_future = "deny"
unwrap_used = "warn"          # deny in lib code
expect_used = "warn"
panic = "warn"                # deny in lib code
todo = "warn"
unimplemented = "warn"
unreachable = "warn"
dbg_macro = "deny"
print_stdout = "warn"

# Style / perf
needless_collect = "warn"
redundant_clone = "warn"
unused_async = "warn"
wildcard_imports = "warn"
needless_pass_by_value = "warn"
ptr_arg = "warn"
redundant_pub_crate = "warn"
useless_format = "warn"
slow_vector_initialization = "warn"
```

### Mandatory commands per validation cycle
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo deny check          # supply chain
cargo udeps               # unused deps (nightly)
```

---

## 1. Ownership & Borrowing

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 1.1 | `.clone()` silence E0382/E0502 | Refactor take `&T` / `&mut T`; restructure ownership | `redundant_clone`; grep `\.clone\(\)` hot paths |
| 1.2 | Own when borrow fine (`fn f(s: String)` then only reads) | Accept `&str` / `&[T]` | `needless_pass_by_value`, `ptr_arg` |
| 1.3 | Return `&String` / `&Vec<T>` | Return `&str` / `&[T]` | `ptr_arg` |
| 1.4 | `Arc<Mutex<T>>` default shared state | Prefer channels (`tokio::sync::mpsc`), `Arc<T>` read-only, `RwLock` read-heavy | grep `Arc<Mutex<` — justify each |
| 1.5 | `Rc`/`Arc` cycles → memory leak | `Weak` for back-refs | Review graph structures |
| 1.6 | Needless `.to_owned()` / `.to_string()` on owned value | Drop call | `redundant_clone`, `useless_conversion` |

---

## 2. Error Handling

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 2.1 | `.unwrap()` / `.expect()` production paths | `?` propagation; `anyhow::Context` bins; `thiserror` libs | `unwrap_used`, `expect_used` |
| 2.2 | `Box<dyn Error>` lib public API | `thiserror` enum + `#[from]` variants | grep `Box<dyn (std::)?[Ee]rror>` in `pub fn` of `lib.rs` |
| 2.3 | `let _ = fallible()` silent drop `Result` | Handle or propagate; intentional → `.unwrap_or_default()` + comment | `let_underscore_must_use` |
| 2.4 | `.ok()?` swallow error → `None` | `.map_err()?` preserve context | grep `\.ok\(\)\?` |
| 2.5 | `panic!`/`todo!`/`unimplemented!` lib code | Return `Result::Err` | `panic`, `todo`, `unimplemented` lints |
| 2.6 | `#[from]` too many variants → flattened noisy errors | Split distinct variants, manual conversion where context differs | Review enum defs |
| 2.7 | Generic `anyhow!("failed")` no context | `.context("what was being done")` every `?` boundary | grep `anyhow!\("[^{]*"\)` |

---

## 3. Async / Tokio

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 3.1 | `MutexGuard` / `RefCell::borrow()` held across `.await` | Scope lock in `{}` + drop before await | `await_holding_lock`, `await_holding_refcell_ref` |
| 3.2 | `tokio::sync::Mutex` short sync critical section | `std::sync::Mutex` when guard never cross `.await` | grep `tokio::sync::Mutex` + immediate sync ops |
| 3.3 | `std::thread::sleep` / blocking I/O async fn | `tokio::time::sleep`; offload CPU work `spawn_blocking` | grep `std::thread::sleep`, `std::fs::` async fn |
| 3.4 | `block_on` inside async runtime → deadlock | Restructure caller async; never nest runtimes | grep `block_on` inside `#[tokio::`-annotated trees |
| 3.5 | Sequential `.await` accept loop starve connections | `tokio::spawn` per accepted connection | AST: `loop { ... accept().await ... .await }` no spawn |
| 3.6 | `tokio::select!` recreate non-cancel-safe futures each iter | Hoist + `tokio::pin!` + `.fuse()` | grep `loop` + `select!` together |
| 3.7 | Unbounded `mpsc::unbounded_channel()` | Bounded `mpsc::channel(n)` + backpressure | grep `unbounded_channel` |
| 3.8 | External call no `tokio::time::timeout` | Wrap `timeout(dur, fut)` | grep `reqwest::`, `sqlx::` no nearby `timeout` |
| 3.9 | `#[tokio::test]` on fn no `.await` | `#[test]` | `unused_async` |
| 3.10 | Counters/state mutated before `.await` not decrement on cancel | `scopeguard::guard` or RAII drop type | Review pre-await mutations |
| 3.11 | Missing `Send` bound spawned future | Add `+ Send + 'static`; restructure if `!Send` types held across await | rustc error; clippy `future_not_send` |

---

## 4. Type Safety & API Design

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 4.1 | "Stringly typed" — raw `String`/`usize` for domain concepts | Newtype: `struct UserId(Uuid)`, `struct InputTokens(usize)` | Review fn signatures |
| 4.2 | "Validate, don't parse" — runtime validation after construction | Parse into struct that *can't represent* invalid state | Review constructors returning `Result<Self, _>` |
| 4.3 | `Option<Option<T>>`, `Result<Result<T, E>, E>` from naive `?` | Flatten `.flatten()` / `.and_then()` | `option_option`, `result_unit_err` |
| 4.4 | `f64`/`f32` for currency | `rust_decimal::Decimal` | grep `f(32\|64)` near `price`/`amount`/`money`/`cost` |
| 4.5 | `bool` param triple `(true, false, true)` call sites | Enum or builder | Review call sites |
| 4.6 | `pub` on internal items | `pub(crate)` / `pub(super)` | `redundant_pub_crate`, `unreachable_pub` |

---

## 5. Traits & Generics

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 5.1 | `dyn Trait` when monomorphization works | `impl Trait` return position; generics elsewhere | Review `Box<dyn` hot loops |
| 5.2 | Bare `trait_object` (`fn f(x: Trait)`) | `&dyn Trait` / `Box<dyn Trait>` | `bare_trait_objects` (hard error 2021) |
| 5.3 | Over-bound `where T: Clone + Debug + Send + Sync + 'static` when only `Clone` used | Drop unused bounds | `trait_duplication_in_bounds`, `type_repetition_in_bounds` |
| 5.4 | Generic over-engineering — single concrete caller | Concrete type until 2nd caller appears | Review |
| 5.5 | Orphan rule violations (`impl Foreign for Foreign`) | Newtype wrapper | rustc E0117 |

---

## 6. Concurrency Safety

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 6.1 | `unsafe impl Send for X {}` no invariant proof | Prove `Send` (document why) or restructure | `non_send_fields_in_send_ty`, `arc_with_non_send_sync` |
| 6.2 | Two-lock ordering undocumented → deadlock | Single lock; or document lock-ordering invariant | Review |
| 6.3 | `Mutex` guard temp lifetime extension in `match` scrutinee | Bind `let` before match | `significant_drop_in_scrutinee` |
| 6.4 | Atomic ordering = `SeqCst` everywhere default | `Relaxed`/`Acquire`/`Release` per actual need | Review atomics; ask if ordering not justified |

---

## 7. Iterators & Allocations

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 7.1 | `.collect::<Vec<_>>().iter()` re-iter | Drop `collect`; chain directly | `needless_collect` |
| 7.2 | `.iter().cloned().collect()` | `.to_vec()` or `.clone()` | `iter_cloned_collect` |
| 7.3 | `Vec::push` in loop no `with_capacity(n)` | Pre-allocate when size known | `slow_vector_initialization` |
| 7.4 | `format!("{}{}", a, b)` string concat in loops | `String::with_capacity` + `push_str`, or `[a, b].concat()` | `useless_format`, `format_in_format_args` |
| 7.5 | `BTreeMap` default | `HashMap` unless ordered iter or range queries needed | grep `BTreeMap` no `.range(` / iter |
| 7.6 | `Vec<u8>` param when `&[u8]` works | `&[u8]` | `ptr_arg` |
| 7.7 | `SmallVec`/`ArrayVec` opportunity missed hot path with bounded N | Stack-alloc via `SmallVec<[T; N]>` | Profile-driven |

---

## 8. Pattern Matching

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 8.1 | `match x { Some(_) => true, None => false }` | `.is_some()` | `redundant_pattern_matching` |
| 8.2 | Nested `match` / `if let` | Collapse | `collapsible_match`, `collapsible_if` |
| 8.3 | Catch-all `_ => {}` hide new enum variants | List variants explicit, or use `#[non_exhaustive]` deliberately | `wildcard_enum_match_arm`, `match_wildcard_for_single_variants` |
| 8.4 | `if let` chain + `else` branches obscure control flow | `let-else` (Rust 1.65+) | Review |

---

## 9. Unsafe & FFI

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 9.1 | `unsafe` block no `// SAFETY: ...` comment | Add comment document invariant | `undocumented_unsafe_blocks` |
| 9.2 | `mem::transmute` for trivial casts | `as` cast, `from_ne_bytes`, `*const T as *const U` | `transmute_*` family, `useless_transmute` |
| 9.3 | Raw pointer arithmetic no bounds reasoning | `slice::get`/`get_unchecked` + invariant doc | Review |
| 9.4 | Missing `Drop` impl on FFI handle | Implement `Drop` for cleanup | Review FFI bindings |

---

## 10. Cargo & Dependencies

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 10.1 | Wildcard versions (`foo = "*"` or `"0.*"`) | Pin minor: `"1.2"` | `cargo deny`; `multiple_crate_versions` |
| 10.2 | `default-features = true` on heavy crates (tokio, serde, reqwest) | Opt-in features only | Review `Cargo.toml`; `cargo features-manager` |
| 10.3 | Unused deps | Remove | `cargo udeps` |
| 10.4 | Duplicated transitive deps from feature non-unification | Align feature flags; check `cargo tree -e features --duplicates` | `cargo tree` |
| 10.5 | Hallucinated crate / wrong crate name | Verify crates.io before add | `cargo add <name>` fails fast |

---

## 11. Testing

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 11.1 | Only happy-path asserts | Add error-path + boundary tests | Review test coverage |
| 11.2 | `#[should_panic]` no `expected = "..."` | Add expected substring — wrong panic still passes test | grep `#\[should_panic\]$` (no parens) |
| 11.3 | No property tests for parsers / data structures | Add `proptest` for invariants | Review presence of `proptest` |
| 11.4 | No snapshot tests multi-line output | Add `insta` | Review |
| 11.5 | Tests share global state no isolation | `serial_test` or per-test setup | Review |
| 11.6 | Mocks where integration test catch real bug | Hit real DB / real HTTP via testcontainers | Review |

---

## 12. Performance

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 12.1 | Alloc in hot loop | Hoist alloc; reuse buffer | Profile `flamegraph` |
| 12.2 | `String::from` / `format!` when `&'static str` works | Use literal | `useless_format` |
| 12.3 | Release profile missing LTO for bins | `[profile.release] lto = "thin"`, `codegen-units = 1` | Check `Cargo.toml` |
| 12.4 | SIMD-amenable loop left scalar | Explicit `std::simd` or `wide` crate when profile-justified | Profile-driven |

---

## 13. Module & Visibility

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 13.1 | `use foo::*;` outside prelude | Explicit imports | `wildcard_imports` |
| 13.2 | Over-pub items leak implementation | `pub(crate)`, `pub(super)` | `unreachable_pub` |
| 13.3 | Re-exports no intention | Audit `pub use` chains | Review |

---

## 14. Lifetime & Drop Hazards

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 14.1 | Return ref into temporary | Return owned, or take `&mut` buffer | rustc; `let_and_return` |
| 14.2 | `RefCell::borrow()` in `if let` outliving `else` branch | Drop borrow explicit | `await_holding_refcell_ref` (partial) |
| 14.3 | `'static` lifetime sprinkled to satisfy compiler | Almost always wrong — restructure ownership | grep `'static` non-trait-object contexts |
| 14.4 | Custom `Drop` may panic | `Drop` impls must not panic during unwinding | Review |

---

## 15. Hallucination & Convention Drift

| # | Anti-pattern | Fix | Detect |
|---|---|---|---|
| 15.1 | Call non-existent stdlib method (e.g. `Vec::push_all`, `String::contains_ignore_case`) | `cargo check` catches; verify against current rustdoc | rustc E0599 |
| 15.2 | Python/C++ idioms in Rust (getters/setters everywhere, `Self::new()` returning `Result` when infallible) | Idiomatic Rust: fields where appropriate, builder pattern when 4+ optional fields | Review |
| 15.3 | Re-implement std (custom `Option`, custom iterator) | Use std | Review |
| 15.4 | Wrong crate for job (using `regex` for parsing structured data) | Use `nom`/`winnow`/`serde` | Review |

---

## 16. Validation Workflow (run in order)

```bash
# 1. Format
cargo fmt --check

# 2. Compile cleanly
cargo check --all-targets --all-features

# 3. Lint with deny-warnings
cargo clippy --all-targets --all-features -- -D warnings

# 4. Tests
cargo test --all-features

# 5. Supply chain
cargo deny check
cargo audit

# 6. Dead code
cargo +nightly udeps

# 7. (Optional) Mutation testing for confidence
cargo mutants --check
```

**Iteration budget:** 3 cycles. Still fail same root error after 3 fix attempts → stop + report human. Issue architectural.

---

## 17. Escalation Triggers (stop and ask human)

Stop + request review when:
- Borrow-checker error needs change fn signature used in >5 call sites.
- Add `Send`/`Sync` needs `unsafe impl` + you cannot articulate invariant in one sentence.
- Two locks needed in same critical section + no obvious ordering exists.
- Failing test needs change assertion to pass.
- `clippy::pedantic` would need >20 `#[allow]` attributes.
- Perf regression after refactor + cause not single `clone()` or alloc.

---

## 18. What This Checklist Does NOT Cover

Out of scope; handle separately:
- Domain logic correctness (use property tests + integration tests).
- API ergonomics beyond Rust idioms (use design review).
- Cross-service contracts (use schema validation, e.g. OpenAPI, protobuf).
- Security review beyond basic dep audit (use `cargo-geiger`, `cargo-vet`, manual threat modeling).