# Strong Grammar Compiler

**Status:** descriptive, extracted from Phase 3 scoped commits
**Type:** integration memo
**Scope:** constitutional-runtime

**Current closure:** Phase 3 compiler-only exit is closed for v0 JSON Strong Grammar. Execution and evidence closure for Strong-originated programs remain outside this phase.

## Purpose

Strong Grammar is a JSON-form constitutional language surface. It lets a caller express structured intent without gaining execution authority.

The Phase 3 boundary is:

```text
Strong JSON
-> Strong AST
-> IrGraph
```

Strong Grammar compiles into existing Constitutional Runtime IR. It does not execute.

## Pipeline

The admitted path is:

```text
Strong JSON
-> Strong AST
-> IrGraph
-> validate_admissibility
-> planning / lowering
-> dispatcher / evidence machinery outside compile/strong
```

`compile/strong` owns only parsing and compilation to IR graph artifacts. Validation, planning, lowering, dispatch, and evidence closure remain existing runtime responsibilities.

## Forbidden Boundaries

The Strong compiler must not introduce:

- direct dispatcher calls from `compile/strong`
- provider or database mutation from `compile/strong`
- evidence writes from `compile/strong`
- tools directly callable from Strong Grammar
- textual syntax in this phase
- an LLM behind the dispatcher

Strong Grammar is admissible language. It is not execution authority.

## Current v0 Constructs

The v0 AST admits:

- `Pipeline`
- `Emit`
- `Confirm`
- `Execute`
- `OnSuccess`
- `OnFailure`
- `SystemReview`
- `DriftReview`

The minimal proving fixture is:

```json
{
  "kind": "Pipeline",
  "steps": [
    {
      "kind": "Emit",
      "event": "demo.started"
    },
    {
      "kind": "Confirm",
      "subject": "demo.confirmed"
    }
  ]
}
```

Additional tests cover every v0 construct at the JSON parse and IR compile boundary. Representative programs also enter existing admissibility and lowering machinery.

This coverage proves the compiler boundary and v0 IR shapes, not execution or evidence closure.

## Current Receipts

Phase 3 scoped commits currently have these receipts:

- `cargo test -p constitutional-runtime strong` passed: 20 Strong tests
- `cargo test --workspace` passed: 241 tests
- `cargo clippy --workspace --all-targets -- -D warnings` passed
- forbidden token scan in `compile/strong` was inspected and found no execution path

Receipt scope:

- Full v0 Strong fixture parses from JSON.
- Full v0 Strong fixture compiles into `IrGraph`.
- Strong JSON parses into Strong AST.
- Strong AST compiles into `IrGraph`.
- Each v0 construct parses from JSON.
- Each v0 construct compiles into `IrGraph`.
- Unknown constructs reject before IR admission.
- Strong-compiled IR passes existing `validate_admissibility`.
- Strong-compiled IR lowers through `MinilabRuntimeLowerer`.
- Branch constructs compile to `Route` IR and preserve branch structure.
- `SystemReview` compiles to review-pipeline IR primitives.
- `DriftReview` compiles to canonical `Execute(flow.drift_review)` IR.
- Tests stop before dispatcher execution.
- Strong-originated execution and evidence closure remain outside Phase 3.

## Remaining Ghosts

- Full Strong product semantics are not complete.
- Execution correctness is intentionally untested here.
- Evidence closure for Strong-originated programs is not yet closed.
- Disk Ghost: local free space has been low during Phase 3 work.
- Git Continuity Ghost: status may be noisy from archive/layout rewrite.
- Docs Ghost: older planning docs may still describe pre-Phase-2 gaps.

These Ghosts are not failures. They are boundaries that must not be hidden by broad completion claims.

## Phase 3 Exit Criteria

Phase 3 compiler-only exit is closed when:

- Strong programs compile to IR graphs.
- Strong-compiled IR passes admissibility.
- Plans and lowered representations are produced through existing machinery.
- Execution occurs only through dispatcher and evidence machinery.
- Strong Grammar cannot call tools directly.
- The initial v0 constructs have tests covering their IR shapes and boundary behavior.

The current v0 compiler-only exit harness covers these criteria. This does not close Phase 4, natural-language ingress, runtime dispatch for Strong-originated programs, or evidence closure for Strong-originated execution.
