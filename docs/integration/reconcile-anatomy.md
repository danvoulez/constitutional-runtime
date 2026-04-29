# Reconcile Anatomy

**Status:** descriptive, extracted from landed `install.reconcile`
**Type:** integration memo
**Scope:** constitutional-runtime, minilab-store

## Decision

`install.reconcile` is the first Reconcile-shaped constitutional slice. It is deliberately not forced into the single-act slice pattern described in `slice-pattern.md`.

The anatomy is:

```text
install.reconcile.planned
-> install.reconcile.step.applied*
-> install.reconcile.reconciled | install.reconcile.failed
```

This proves a third runtime shape:

- `outbound.send` proves a business material act.
- `host.pair` proves a platform material act.
- `install.reconcile` proves state convergence.

## Runtime Contract

Input is intentionally minimal:

```text
{ installation_id, correlation_id }
```

The orchestrator resolves desired and observed state from the `installation` row. Callers do not submit drift claims directly.

Current substrate:

- table: `installation`
- desired state: `desired_manifest`
- observed state: `observed_manifest`
- current manifest shape: top-level `services` object

The narrow manifest shape is a proving substrate, not the final install model.

## Evidence Contract

Success path:

- `install.reconcile.planned`
- zero or more `install.reconcile.step.applied`
- `install.reconcile.reconciled`

Failure path:

- `install.reconcile.failed`

Failure payloads include:

- `reason_code`
- `reason_detail`
- `phase`
- `applied_steps`
- `remaining_steps`
- `partial_convergence`

`partial_convergence = true` means at least one sub-step applied and at least one sub-step remained unapplied when the slice failed.

## Idempotency

The slice checks for a previous `install.reconcile.reconciled` row with the same `installation_id` and desired manifest hash.

If found, a rerun still writes a reconstructable chain under the new `correlation_id`:

```text
install.reconcile.planned
-> install.reconcile.reconciled { idempotent: true, applied_steps: 0 }
```

It does not reapply sub-steps.

## Lowering and Dispatch

Canonical action:

```text
install.reconcile
```

Lowered command:

```text
OperationalCommand {
  namespace: "install",
  verb: "reconcile",
  target_runtime: Platform,
  args: { installation_id, correlation_id }
}
```

Dispatcher handoff:

```text
dispatch_operational_command
-> submit_install_reconcile
```

## Proving Surface

HTTP route:

```text
POST /installations/{installation_id}/reconcile
```

Body:

```json
{
  "correlation_id": "optional uuid"
}
```

## Tests

Landed tests:

- happy path plans, applies one drift step, and reconciles
- sub-step failure closes as `install.reconcile.failed`
- idempotent rerun plans and reconciles without reapplying steps
- partial convergence is explicit when a later step fails
- lowered command dispatches into the reconcile slice

## Deferred

- Real host-side install executor.
- Rich payload/service manifest model.
- Verify-step closure before `reconciled`.
- Canon/Elastic authority checks over desired manifests.
- Production rejection of unanchored implementation claims.
