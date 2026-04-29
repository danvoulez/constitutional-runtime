# Baseline Receipt - 2026-04-29

**Scope:** extracted `/Users/ubl-ops/Downloads/constitutional-runtime.zip` into `/Users/ubl-ops/constitutional-runtime`.

## Archive

The active baseline is `constitutional-runtime.zip`.

`new-runtime.zip` remains context only. It contains a broader bundle and an older nested `constitutional-runtime/` copy. The extracted runtime used for this receipt includes files absent from the nested copy, including `act_identity.rs`, `failure.rs`, `idempotency.rs`, `operational_grammar.rs`, `planning_compiler.rs`, `plan_executor.rs`, `minilab-api/src/agent_runtime.rs`, `minilab-store/src/real_dispatcher.rs`, and `docs/PLAN.md`.

## Baseline Repair Before Receipt

The first `cargo test --workspace` found a compile failure in `minilab-api`: runtime/MCP handlers referenced `AppState.agent_runtime`, but `AppState` and `main` did not construct that field.

Repair:

- Added `agent_runtime: AgentRuntimeService` to `AppState`.
- Constructed `AgentRuntimeService` in `main`.
- Updated `app.rs` test state constructors.

Clippy repairs:

- Fixed doc list indentation in `capability.rs` and `minilab-store/src/policy.rs`.
- Derived `Default` for `PrimitiveName`.
- Replaced one cloned test slice with `std::slice::from_ref`.
- Replaced lazy `json!(null)` fallbacks with `Value::Null`.
- Removed needless `Option::as_deref()` in `mcp_query.rs`.
- Added narrow `#[allow]` attributes for intentionally wide boundary functions and existing large error result shapes.

## Verification Matrix

| Command | Result | Test count |
|---|---:|---:|
| `cargo test --workspace` | pass | 214 |
| `cargo test --workspace --features sqlite-evidence` | pass | 214 |
| `cargo test --workspace --features supabase-evidence` | pass | 214 |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass | n/a |

## Feature Matrix

Only `crates/constitutional-runtime` declares Cargo features:

| Feature | Dependency enabled | Status |
|---|---|---|
| `sqlite-evidence` | `rusqlite` | tested green |
| `supabase-evidence` | `reqwest` | tested green |

## Orphan Module Audit

| Module | Current role | Decision for Phase 0 |
|---|---|---|
| `crates/constitutional-runtime/src/ingress.rs` | Defines `IngressMode`, exported from `lib.rs`; no internal runtime consumer yet. | Keep as declared ingress vocabulary. Move under `contracts/` or `ir/` during Phase 1 reorganization unless Agent Runtime wiring claims it first. |
| `crates/constitutional-runtime/src/refs.rs` | Defines `NodeId`, `DataRef`, `TargetRef`, `SurfaceRef`, `PolicyId`; widely used and exported. | Keep. Fold into `ir/refs.rs` during Phase 1. |
| `crates/constitutional-runtime/src/policy.rs` | Defines `PolicyClass`, exported and used by validation/tests. | Keep. Fold into `contracts/policy.rs` or `ir/policy.rs` during Phase 1. |

## Exit Condition

Phase 0 baseline is trustworthy after the small compile/clippy repairs above:

- Tests are green.
- Clippy is green with warnings denied.
- Test count is recorded.
- Feature matrix is recorded.
- Orphan-module audit is recorded.
