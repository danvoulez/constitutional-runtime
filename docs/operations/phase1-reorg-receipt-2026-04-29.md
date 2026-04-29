# Phase 1 Reorganization Receipt - 2026-04-29

**Scope:** behavior-preserving module reorganization after Phase 0 baseline verification.

## Constitutional Runtime

`crates/constitutional-runtime/src/` now follows the planned shape:

```text
ir/
compile/
execute/
contracts/
evidence/
```

Moved modules:

- `ir.rs` -> `ir/mod.rs`
- `act_identity.rs` -> `ir/action_identity.rs`
- `refs.rs` -> `ir/refs.rs`
- `decision.rs` -> `compile/decision.rs`
- `lowering.rs` -> `compile/lowering.rs`
- `operational_grammar.rs` -> `compile/operational_grammar.rs`
- `planning_compiler.rs` -> `compile/planning_compiler.rs`
- `plan_executor.rs` -> `execute/plan_executor.rs`
- `capability.rs` -> `contracts/capability.rs`
- `failure.rs` -> `contracts/failure.rs`
- `idempotency.rs` -> `contracts/idempotency.rs`
- `ingress.rs` -> `contracts/ingress.rs`
- `policy.rs` -> `contracts/policy.rs`
- `validation.rs` -> `contracts/validation.rs`
- `evidence.rs` -> `evidence/mod.rs`
- `evidence_sqlite.rs` -> `evidence/sqlite.rs`
- `evidence_supabase.rs` -> `evidence/supabase.rs`

Compatibility modules in `lib.rs` preserve previous public import paths.

## Minilab Store

`crates/minilab-store/src/` now follows the planned shape:

```text
slices/
dispatch/
legacy/
persistence/
```

Moved modules:

- `host_pair.rs` -> `slices/host_pair/mod.rs`
- `outbound_orchestrator.rs` -> `slices/outbound_send/orchestrator.rs`
- `eligibility.rs` -> `slices/outbound_send/eligibility.rs`
- `premium.rs` -> `slices/outbound_send/premium.rs`
- `policy.rs` -> `slices/outbound_send/policy.rs`
- `optout_gate.rs` -> `slices/outbound_send/optout_gate.rs`
- `dispatcher.rs` -> `dispatch/mod.rs`
- `real_dispatcher.rs` -> `dispatch/real.rs`
- `client.rs` -> `persistence/client.rs`
- `store.rs` -> `persistence/store.rs`
- `webhook.rs` -> `persistence/webhook.rs`
- `outbound.rs` -> `legacy/outbound.rs`
- `outreach.rs` -> `legacy/outreach.rs`
- `campaign.rs` -> `legacy/campaign.rs`
- `scoring.rs` -> `legacy/scoring.rs`
- `reply.rs` -> `legacy/reply.rs`

Compatibility modules in `lib.rs` preserve previous public import paths.

## Verification

Post-reorganization gates:

| Command | Result | Test count |
|---|---:|---:|
| `cargo test --workspace` | pass | 214 |
| `cargo test --workspace --features sqlite-evidence` | pass | 214 |
| `cargo test --workspace --features supabase-evidence` | pass | 214 |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass | n/a |

## Exit Condition

Phase 1 exits cleanly:

- Same test count as baseline.
- No intended semantic changes.
- Public re-exports preserved.
- Legacy substrate is physically named under `legacy/`.
- New material behavior now has an obvious home under `slices/`.
