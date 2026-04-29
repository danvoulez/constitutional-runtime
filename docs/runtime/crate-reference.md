# Runtime Crate Reference

**Updated:** 2026-04-29
**Status:** reflects Phase 1 module reorganization.

## Workspace Crates

### `constitutional-runtime`

Substrate-neutral kernel for IR, compilation, contracts, execution, and evidence.

```text
crates/constitutional-runtime/src/
├── ir/
│   ├── mod.rs
│   ├── action_identity.rs
│   └── refs.rs
├── compile/
│   ├── decision.rs
│   ├── lowering.rs
│   ├── operational_grammar.rs
│   └── planning_compiler.rs
├── execute/
│   └── plan_executor.rs
├── contracts/
│   ├── capability.rs
│   ├── failure.rs
│   ├── idempotency.rs
│   ├── ingress.rs
│   ├── policy.rs
│   └── validation.rs
├── evidence/
│   ├── mod.rs
│   ├── sqlite.rs
│   └── supabase.rs
└── lib.rs
```

Compatibility modules in `lib.rs` preserve the previous public paths, including `constitutional_runtime::lowering`, `constitutional_runtime::planning_compiler`, `constitutional_runtime::capability`, and `constitutional_runtime::refs`.

### `minilab-core`

Business-domain vocabulary and still-mostly-unwired authority material:

- `business_canon.rs`
- `elastic_config.rs`
- `entities.rs`
- `departments.rs`
- `evidence.rs`
- `exploration.rs`
- `registry.rs`
- `simulation.rs`
- `week1.rs`
- `workflows.rs`

Phase 5 is responsible for turning Business Canon and Elastic Config into runtime authority.

### `minilab-store`

Minilab persistence, dispatch, slices, and explicitly named legacy substrate.

```text
crates/minilab-store/src/
├── slices/
│   ├── host_pair/
│   ├── install_reconcile/
│   └── outbound_send/
├── dispatch/
│   ├── mod.rs
│   └── real.rs
├── legacy/
│   ├── campaign.rs
│   ├── outbound.rs
│   ├── outreach.rs
│   ├── reply.rs
│   └── scoring.rs
├── persistence/
│   ├── client.rs
│   ├── store.rs
│   └── webhook.rs
├── evidence.rs
└── lib.rs
```

Compatibility modules in `lib.rs` preserve the previous public paths, including `minilab_store::client`, `minilab_store::dispatcher`, `minilab_store::host_pair`, `minilab_store::install_reconcile`, `minilab_store::outbound_orchestrator`, `minilab_store::reply`, and `minilab_store::store`.

New material behavior should land under `slices/`. Existing code under `legacy/` is allowed to coexist only while its replacement slice is being built or verified.

### `minilab-api`

HTTP/API surface for webhooks, act-shaped proving routes, Agent Runtime, and MCP adapters.

Current important surfaces:

- `POST /outbound/send`
- `POST /host-pairings`
- `POST /installations/{installation_id}/reconcile`
- Twilio/SendGrid webhook ingestion
- Agent Runtime service and route builders
- MCP command/query/artifact route builders

The API should consume and expose public JSON contracts, not internal Rust-only types.

## Verification Baseline

After Phase 1:

```text
cargo test --workspace                                  pass, 214 tests
cargo test --workspace --features sqlite-evidence       pass, 214 tests
cargo test --workspace --features supabase-evidence     pass, 214 tests
cargo clippy --workspace --all-targets -- -D warnings   pass
```
