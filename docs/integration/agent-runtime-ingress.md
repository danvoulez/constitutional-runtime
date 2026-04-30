# Agent Runtime Ingress

**Status:** Phase 4 scoped implementation memo
**Scope:** natural-language ingress only

## Law

Natural language never executes.

Natural language enters a pocket pipeline runtime. Pocket runtimes govern language-state transitions only. The Constitutional Runtime remains sovereign.

The message is not the act. The candidate is not the act. The IR is not the act. Only the constitutional pipeline may produce material action.

No LLM sits behind the dispatcher.

## Implemented Path

The Phase 4 ingress path is:

```text
natural-language message
-> agent.message.received
-> classify_natural_language_candidate
-> PocketRuntimeStateRecord
-> evaluate_ingress_admission
-> AgentRuntimeIngressReport
-> stop before dispatch
```

The mounted HTTP path is:

```text
POST /api/agent-runtime/places/{place_id}/messages
GET  /api/agent-runtime/reports/{message_id_or_correlation_id}
```

The endpoint returns a reconstructable ingress report. It does not return proof of material execution.

## Candidate Outcomes

The classifier may produce:

- `StrongCandidate`
- `OperationalCandidate`
- `ClarificationRequired`
- `Rejected`
- `GhostRecord`

The admission gate may move valid Strong or operational candidates to `AdmissibilityPending`, meaning ready for the later IR gate. It does not itself compile, validate, lower, dispatch, mutate providers/databases, or write canonical evidence.

## Report Shape

`AgentRuntimeIngressReport` contains:

- `message_id`
- `correlation_id`
- `origin`
- received state
- candidate classification
- ghosts
- admission state
- IR readiness
- validation/admissibility status if reached
- planning/lowering status if reached
- dispatch boundary status
- ingress event references
- final state
- next required action

Current final states are intentionally ingress-local:

- `clarification_required`
- `rejected`
- `ghost_recorded`
- `ready_for_ir`
- `admissibility_pending`
- `planned_before_dispatch`
- `blocked`

Do not use `completed` here as constitutional closure.

## Event Names

Ingress reports may reference these event names:

- `agent.message.received`
- `agent.candidate.classified`
- `agent.pipeline.admitted`
- `agent.response.emitted`

In this implementation these are report/event references, not canonical evidence closure. `agent.response.emitted` means an ingress/report response, not a material act.

## Boundaries

Forbidden from Agent Runtime ingress:

- message -> action
- candidate -> dispatch
- LLM -> dispatcher
- classification -> tool call
- route -> provider/database mutation
- route -> shell/MCP command handler
- `completed` as constitutional closure without receipt and scope

The Operator is the Translator. The Ingress Classifier is a small classifier role for candidate classification only.
