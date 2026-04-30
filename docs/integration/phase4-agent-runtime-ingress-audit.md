# Phase 4A - Agent Runtime Ingress Boundary Audit

**Status:** scoped audit
**Date:** 2026-04-30
**Scope:** `crates/minilab-api/src/agent_runtime.rs` and adjacent MCP adapters

## Intent

Phase 4 closes natural-language ingress without making natural language an execution authority.

The governing law is:

```text
Natural language never executes.
```

This audit maps the existing Agent Runtime surface before new pocket-runtime state types are introduced.

## Current Route Shape

`AgentRuntimeService` already exposes an agent runtime router:

```text
POST /places/{place_id}/messages
GET  /sessions/{session_id}
```

Source: `crates/minilab-api/src/agent_runtime.rs`.

Current app wiring exports this router from `crates/minilab-api/src/lib.rs`, but `crates/minilab-api/src/app.rs` does not mount it into the main API router. The main API currently mounts health, webhooks, outbound, host-pairings, and installations.

Phase 4 implementation must decide the public mount deliberately. The governing plan names:

```text
POST /api/agent-runtime/places/:place_id/messages
```

Do not assume the current exported-but-unmounted route is already the final Phase 4 ingress.

## Observed Service Behavior

| Surface | Input | Observed behavior | Constitutional status |
| --- | --- | --- | --- |
| `submit_message` | `place_id`, message text, optional session/app/files | trims text, rejects empty input, calls `classify_message`, records a draft snapshot in the in-memory store | ingress shell; not a constitutional pipeline yet |
| `classify_message` material path | message containing simple action words such as `send`, `pair`, `delete`, `update`, `deploy`, `continue terminal`, or `run command` | returns a `proposal` with `waiting` status and `requires_confirmation=true` | advisory/proposal only; no IR admission |
| `classify_message` artifact path | message mentioning report/artifact or carrying files | returns an `artifact` output with completed shell status and artifact summary | shell-side artifact state; not evidence closure |
| `classify_message` advisory path | other text | returns an `advisory` output with completed shell status | advisory-only; not material action |
| `record_draft` | draft outcome | inserts an `AgentRuntimeSessionSnapshot` into `Arc<Mutex<AgentRuntimeStore>>` and returns an acknowledgement | in-memory shell mutation only |
| `submit_place_intent` | structured intent kind and payload | creates a governed proposal and records a draft snapshot | proposal surface; no IR/lowering |
| `submit_operational_action` | action kind and params | creates a governed proposal and records a draft snapshot | proposal surface; no IR/lowering |
| `continue_terminal_session` | terminal session id and command | returns `confirmation_required` for known terminal sessions or `unavailable` otherwise | no process execution observed |
| `request_confirmation` | action reference | returns a confirmation-request JSON object | advisory/control response only |
| `create_artifact` / `attach_chatgpt_summary` / `attach_run_output` / `store_external_result` | run/session references plus structured content | update existing in-memory snapshots and audit trail fields | side-effecting shell updates; not canonical evidence |
| `append_evidence_note` | run/session references and note | appends an audit event to the in-memory snapshot | misleading name risk: this is not canonical `EvidenceStore` closure |

## Side-Door Scan

Within `crates/minilab-api/src/agent_runtime.rs`, this audit did not find direct calls to:

- `lower_and_dispatch_execute`
- `dispatch_operational_command`
- `execute_compiled_plan`
- provider clients
- database clients
- `EvidenceStore`
- canonical evidence ledger writes
- process spawning

The service mutates only the in-memory `AgentRuntimeStore` under `Arc<Mutex<_>>`.

## Adjacent MCP Surfaces

`crates/minilab-api/src/mcp_command.rs` exposes tools that call Agent Runtime proposal methods:

- `submit_place_intent`
- `start_governed_handoff`
- `continue_terminal_session`
- `request_confirmation`
- `submit_operational_action`

These tools currently return governed proposal or confirmation JSON. They do not call the constitutional dispatcher directly.

`crates/minilab-api/src/mcp_artifacts.rs` exposes tools that update Agent Runtime shell snapshots:

- `create_artifact`
- `attach_chatgpt_summary`
- `attach_run_output`
- `store_external_result`
- `append_evidence_note`

These methods mutate in-memory session state. They should be treated as shell-state updates until Phase 4 gives them pocket-runtime state records and names their non-authoritative evidence shape.

`crates/minilab-api/src/mcp_query.rs` exposes read-only place/session/doc query tools. It reads Agent Runtime snapshots and docs; it does not create proposals or execute material acts.

## Current Boundary Ruling

The existing Agent Runtime is not yet the Phase 4 pocket pipeline runtime.

It is closer to an adapter shell:

```text
message
-> heuristic classification
-> draft snapshot
-> acknowledgement
```

It does not currently enter:

```text
candidate
-> IR
-> validation/admissibility
-> planning/lowering
-> stop before dispatch
```

It also does not appear to dispatch or execute directly.

## Risks To Fix In Phase 4B-4E

- **Completion vocabulary risk:** advisory/artifact paths mark `session_status`, `run_status`, and `phase` as `completed`. Phase 4 should distinguish language-shell completion from constitutional closure.
- **Evidence naming risk:** `append_evidence_note` and audit events use evidence-like language but do not write canonical evidence. Phase 4 should rename or explicitly type these as non-canonical ingress records.
- **Classifier shape risk:** `classify_message` returns `DraftOutcome`, not a data-only `CandidateClassification`. Phase 4 should introduce explicit `IngressState`, `CandidateKind`, and `AdmissionState`.
- **Route continuity risk:** Agent Runtime routes are exported but not mounted in `build_app`. Phase 4 must choose the public mount intentionally.
- **Policy field risk:** `policy_overrides` is accepted by `AgentRuntimeSendRequest` but is not consumed by `submit_message`. Phase 4 must either reject, record, or intentionally ignore it with a receipt.
- **Status authority risk:** `attach_run_output` can mark an action as `completed` inside the shell. Phase 4 must prevent this from being confused with material execution/evidence closure.

## Required Next Slice

Phase 4B should add explicit pocket-runtime data types before changing route behavior:

```text
IngressState:
  Received
  Classified
  ClarificationRequired
  Rejected
  GhostRecorded
  CandidateProposed

CandidateKind:
  StrongCandidate
  OperationalCandidate
  ClarificationRequired
  Rejected
  GhostRecord

AdmissionState:
  NotAdmitted
  AdmissibilityPending
  AdmittedToIr
  Validated
  Planned
  Lowered
  StoppedBeforeDispatch
```

The next implementation must preserve:

```text
raw natural language -> no IR directly
candidate classification -> no execution authority
shell/audit note -> no evidence closure
lowering, if reached later -> stop before dispatch
```

## Later Status - Phase 4B

Phase 4B added explicit pocket-runtime data types in `crates/minilab-api/src/agent_runtime.rs`:

- `IngressState`
- `CandidateKind`
- `AdmissionState`
- `NaturalLanguageOrigin`
- `PocketRuntimeStateRecord`

This is type surface only. It does not mount the Agent Runtime route, change classifier behavior, admit natural language to IR, lower candidates, dispatch, mutate providers/databases, or write canonical evidence.

Receipts for the scoped slice:

- `cargo test -p minilab-api agent_runtime` passed: 7 Agent Runtime tests
- `cargo test --workspace` passed: 243 tests
- `cargo clippy --workspace --all-targets -- -D warnings` passed
- forbidden token scan in `crates/minilab-api/src/agent_runtime.rs` found execution tokens only inside the negative serialization test assertions

## Ghosts

- Phase 4B type surface exists, but classifier integration, candidate admission, and lowering gates are not implemented by this audit.
- The main API route mount for Agent Runtime remains unresolved.
- Existing shell completion statuses may confuse future callers until typed pocket-runtime state is introduced.
- Evidence closure for natural-language-originated programs remains outside this audit.
- This audit used source inspection only; no runtime HTTP probe was run.
