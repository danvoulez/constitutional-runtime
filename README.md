# Constitutional Runtime

Rust library and documentation for **Minilab’s constitutional execution model**: intelligence proposes, policy and capability govern, the runtime lowers and executes only what is **admissible**, and evidence legitimizes outcomes.

## What this crate provides

| Module | Role |
|--------|------|
| `ir` | Sixteen canonical IR primitives (`IRPrimitive`) and `IrNode` (shapes frozen at **v0.1**) |
| `policy` | Policy classes (A–D): see `src/policy.rs` — **D is reserved** (not executable at normal runtime boundary) |
| `capability` | `CapabilityManifest` (primitives, optional kind filter, `evidence.write` guarantee) |
| `evidence` | `EvidenceContract`, `EvidenceRecord`, `EvidenceStore`, `FileEvidenceStore`; optional `SqliteEvidenceStore` / `SupabaseRestEvidenceStore` (features), `FailureToClose` |
| `lowering` | `OperationalCommand` (**single canonical** command shape), `LoweringPlan`, `MinilabRuntimeLowerer` |
| `decision` | `DecideResolver`, `compile_flow` / `compile_node`, `resolve_lower_one`, `lower_compiled_flow` — `Decide` → concrete IR, then same lowerer |
| `validation` | `validate_structure`, `validate_policy`, `validate_capability`, `validate_admissibility` |
| `ingress` | `IngressMode` (fast path → premium exception) |

Execution is **not sovereign**: a command must be semantically valid, policy-permitted, capability-realizable, and evidentially accountable.

**`Decide`** is not lowered by `MinilabRuntimeLowerer`. Resolve it in **`decision`** (`compile_*` / `resolve_lower_one`) so the runtime lowerer never sees an unresolved `Decide`.

## Build

```bash
cargo test
cargo test --features sqlite-evidence
cargo test --features supabase-evidence
# optional: all evidence backends
cargo test --features "sqlite-evidence,supabase-evidence"
```

This repository declares its own `[workspace]` so it stays independent of other `Cargo.toml` files higher in the directory tree.

## Documentation

- [Constitutional runtime](docs/constitutional-runtime.md) — definition, separation of powers, pipeline
- [Minilab architecture](docs/minilab-architecture.md) — system scope, adopted foundations, local sovereignty
- [IR and lowering](docs/ir-and-lowering.md) — primitives, boundary compiler, validation, decision pipeline

## Completion criteria (this phase)

For every **runtime-lowerable** primitive: structural validation, policy validation, capability validation, lowering, and evidence kinds in `LoweringPlan`. Closure is explicit: **`close_execution_evidence`** returns `Err(FailureToClose)` if any evidence write fails.

## Status

**Constitutional Runtime Core (closed for this phase)** — Same as v0.1 operational coverage, plus: explicit **`PolicyClass::D`** semantics, **`decision`** compilation boundary, optional **`SupabaseRestEvidenceStore`** (`supabase-evidence`), golden tests for routing/checkpoint. **Milestone:** IR + admissibility + lowering + evidence closure + **decide resolution** + **real REST sink path** without a second command representation.
