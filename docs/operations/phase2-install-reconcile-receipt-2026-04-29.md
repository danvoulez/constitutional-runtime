# Phase 2 Install Reconcile Receipt - 2026-04-29

**Scope:** land the first Reconcile-shaped constitutional slice: `install.reconcile`.

## What Landed

Runtime/evidence:

- Added evidence kinds:
  - `install.reconcile.planned`
  - `install.reconcile.step.applied`
  - `install.reconcile.reconciled`
  - `install.reconcile.failed`
- Added migration `011_install_reconcile_slice.sql` with minimal `installation` substrate.
- Added `submit_install_reconcile(client, input) -> InstallReconcileOutcome`.

Lowering/dispatch/API:

- Added canonical lowerer branch for `install.reconcile`.
- Added dispatcher handoff for `OperationalCommand { namespace: "install", verb: "reconcile", target_runtime: Platform }`.
- Added RealDispatcher outcome mapping.
- Added HTTP proving surface:

```text
POST /installations/{installation_id}/reconcile
```

Docs:

- Added `docs/integration/reconcile-anatomy.md`.
- Updated `docs/integration/slice-pattern.md` so reconcile is no longer future work.
- Updated migration and crate references.

## Behavior

The slice resolves desired and observed state from the `installation` row. The current manifest shape is deliberately narrow:

```json
{
  "services": {
    "api": "v2"
  }
}
```

It writes:

```text
install.reconcile.planned
-> install.reconcile.step.applied*
-> install.reconcile.reconciled | install.reconcile.failed
```

Failure payloads include `phase`, `reason_code`, `applied_steps`, `remaining_steps`, and `partial_convergence`.

Idempotent reruns detect a previous `install.reconcile.reconciled` row for the same `installation_id` and desired manifest hash, then close under the new correlation without reapplying steps.

## Tests

New tests cover:

- happy path
- sub-step failure
- idempotent rerun
- partial convergence
- lowerer contract
- dispatcher handoff

## Verification

Post-Phase-2 gates:

| Command | Result | Test count |
|---|---:|---:|
| `cargo test --workspace` | pass | 221 |
| `cargo test --workspace --features sqlite-evidence` | pass | 221 |
| `cargo test --workspace --features supabase-evidence` | pass | 221 |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass | n/a |

## Deferred

- Real host-side install executor.
- Rich payload/service manifest model.
- Verify-step closure before `install.reconcile.reconciled`.
- Canon/Elastic authority over desired manifests.
- Production rejection of unanchored implementation claims.
