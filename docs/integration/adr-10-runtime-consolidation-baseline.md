# ADR 10 - Runtime Consolidation Baseline

**Status:** accepted
**Date:** 2026-04-29
**Scope:** constitutional-runtime workspace baseline after extraction from `constitutional-runtime.zip`

## Context

The runtime has crossed from memo-shaped architecture into a working kernel with two live constitutional slices:

- `outbound.send`, a provider/business material act.
- `host.pair`, a platform/infrastructure material act.

The extracted archive also contains the consolidation tail needed by the final work plan: canonical act identity, operational grammar, planning compiler, plan executor, structured runtime failures, dispatch handoff, evidence stores, Agent Runtime service code, and the real dispatcher bridge.

Before Phase 1 reorganization, the baseline must be trusted. That means the repo must build, tests must pass, clippy must be clean, and the remaining orphan-shaped modules must be named rather than hand-waved.

## Decision

Adopt the extracted `constitutional-runtime.zip` contents as the active baseline after the following Phase 0 repairs:

- `AppState` now carries `AgentRuntimeService`, matching the already-landed Agent Runtime and MCP handler code.
- `main` constructs the `AgentRuntimeService` at startup.
- Test state constructors construct the same service.
- Clippy-only documentation and style issues are cleaned up.
- Existing large error result shapes in the planning compiler are retained for Phase 0 and narrowly allowed, because boxing or redesigning them would change public error semantics during a baseline verification phase.

The baseline verification receipt is recorded in `docs/operations/baseline-receipt-2026-04-29.md`.

## Verification

The Phase 0 matrix is green:

```text
cargo test --workspace                                  pass, 214 tests
cargo test --workspace --features sqlite-evidence       pass, 214 tests
cargo test --workspace --features supabase-evidence     pass, 214 tests
cargo clippy --workspace --all-targets -- -D warnings   pass
```

## Orphan Module Ruling

The Phase 0 audit names three small constitutional-runtime modules:

- `ingress.rs` remains declared ingress vocabulary. It is exported but not load-bearing yet.
- `refs.rs` remains shared identity/reference vocabulary and is load-bearing.
- `policy.rs` remains shared policy-class vocabulary and is load-bearing.

None are deleted in Phase 0. They are reserved for Phase 1 module reorganization:

- `refs.rs` should move under `ir/`.
- `policy.rs` should move under `contracts/` or `ir/`.
- `ingress.rs` should move under `contracts/` unless Agent Runtime ingress wiring claims a better home.

## Consequences

Phase 1 can reorganize without arguing about whether the baseline itself is broken.

The runtime is not yet constitutionally complete. The remaining gaps are still the same:

- `install.reconcile` is missing.
- Strong Grammar is not yet a compiler.
- natural-language Agent Runtime ingress is not yet closed through IR, planning, dispatch, and evidence.
- Business Canon and Elastic Config are not runtime authority yet.
- `reply.received` remains legacy-shaped.
- production executors are incomplete.
- receipt/operator surface is not first-class.
- legacy substrate still exists.

This ADR does not authorize new material side doors. It only freezes the repaired baseline from which the monopoly work begins.

## Later Status

This ADR records the Phase 0/ADR 10 baseline. Later scoped receipts changed two of the gaps above:

- `install.reconcile` has since landed as the Phase 2 third slice.
- Strong Grammar Phase 3 compiler-only v0 has since closed through JSON -> AST -> IR -> admissibility -> lowering -> stop before dispatch.

Strong-originated execution and evidence closure remain outside Phase 3. Natural-language Agent Runtime ingress remains future work and must enter through the pocket pipeline runtime described in `docs/FINAL_WORK_PLAN.md`.
