# Constitutional Runtime - Final Work Plan

**Starting archive:** `/Users/ubl-ops/Downloads/constitutional-runtime.zip`
**Context archive:** `/Users/ubl-ops/Downloads/new-runtime.zip`
**Evidence status:** archive/source inspection completed on 2026-04-29.
**Build/test status:** Phase 0 baseline was later verified in this workspace. Phase 3 compiler-only exit receipts are also recorded below; do not treat these as Phase 4 implementation evidence.

## Archive Decision

The plan starts from `constitutional-runtime.zip`, not the nested runtime inside `new-runtime.zip`.

Evidence:

- `new-runtime.zip` contains a broader workspace bundle: `places.minilab.work/`, `bundle-minilab/`, extracted notes, and a nested `constitutional-runtime/`.
- The nested `new-runtime/constitutional-runtime/` copy is older and smaller. It has no files that are absent from `constitutional-runtime.zip`.
- `constitutional-runtime.zip` contains the newer runtime baseline expected by this plan, including:
  - `crates/constitutional-runtime/src/act_identity.rs`
  - `crates/constitutional-runtime/src/failure.rs`
  - `crates/constitutional-runtime/src/idempotency.rs`
  - `crates/constitutional-runtime/src/operational_grammar.rs`
  - `crates/constitutional-runtime/src/planning_compiler.rs`
  - `crates/constitutional-runtime/src/plan_executor.rs`
  - `crates/minilab-api/src/agent_runtime.rs`
  - `crates/minilab-store/src/real_dispatcher.rs`
  - `docs/PLAN.md`
- The outer `places.minilab.work/` app in `new-runtime.zip` remains useful context for the later registry/operator surface, but it is not the Rust runtime baseline.

## North Star

The runtime stops "trying" when the constitutional path becomes a monopoly over material action.

Every material act must pass through:

```text
intent / ingress
-> canonical act identity
-> IR graph
-> structural validation
-> policy validation
-> capability jurisdiction
-> authority / canon check
-> deterministic planning / lowering
-> dispatcher handoff
-> slice execution
-> evidence closure
-> receipt / correlation reconstruction
```

No side doors. No LLM behind the dispatcher. No handler directly mutating provider or database state for material acts. No provider success counted as constitutional closure unless evidence closure succeeds.

## Phase 0 - Verify the Baseline

Before adding anything, make the selected archive trustworthy.

Run from an extracted copy of `constitutional-runtime.zip`:

```bash
cargo test --workspace
cargo test --workspace --features sqlite-evidence
cargo test --workspace --features supabase-evidence
cargo clippy --workspace --all-targets -- -D warnings
```

Also:

- Write ADR 10 for the landed runtime consolidation.
- Audit orphan modules: `ingress.rs`, `refs.rs`, `policy.rs`.
- Record test count.
- Record feature matrix.
- Record failures as contracts with receipts, not vibes.

Exit condition:

- Tests and clippy are green, or failures are explicitly filed with receipts.
- Baseline receipt exists.

## Phase 1 - Reorganize Without Changing Behavior

Make the repo shape enforce the mental model.

Move `crates/constitutional-runtime/src/` into:

```text
ir/
compile/
execute/
contracts/
evidence/
```

Move `crates/minilab-store/src/` into:

```text
slices/
dispatch/
legacy/
persistence/
```

The important move is `legacy/`. Legacy code may coexist during transition, but it must be named as legacy and put on a retirement track. New work must not land there.

Exit condition:

- Same test count as baseline.
- No semantic changes.
- Public re-exports preserved.
- New work cannot hide inside legacy files.

## Phase 2 - Prove the Third Slice: `install.reconcile`

`outbound.send` proves a business material act. `host.pair` proves an infrastructure/platform material act. `install.reconcile` proves state convergence.

This matters because reconcile asks harder questions:

- What is desired state?
- What is observed state?
- What is drift?
- What is partial convergence?
- What can retry?
- What must escalate?
- What evidence proves reconciliation rather than mere execution?

Do:

- Add migration `011_install_reconcile_slice.sql` if needed.
- Add `submit_install_reconcile(client, input) -> InstallReconcileOutcome`.
- Add lowerer branch for canonical `install.reconcile`.
- Add `POST /installations/:id/reconcile`.
- Add `docs/integration/reconcile-anatomy.md`.

Evidence kinds:

- `install.reconcile.planned`
- `install.reconcile.step.applied`
- `install.reconcile.reconciled`
- `install.reconcile.failed`

Tests:

- Happy path.
- Sub-step failure.
- Idempotent re-run.
- Partial convergence.

Exit condition:

- Reconcile reconstructs by `correlation_id`.
- Partial convergence and failure phases are explicit.
- Reconcile is no longer future work in slice-pattern docs.

## Phase 3 - Land Strong Grammar as Compiler, Not Executor

Strong Grammar must not become another oracle surface.

Rule:

```text
Strong Grammar does not execute.
Strong Grammar compiles to IR.
IR is admitted or rejected.
```

Do:

- Add `compile/strong/ast.rs`.
- Add `compile/strong/parser.rs`.
- Add `compile/strong/compiler.rs`.
- Start with JSON-form Strong Grammar, not textual syntax.

Initial constructs:

- `Pipeline`
- `OnSuccess`
- `OnFailure`
- `Emit`
- `Confirm`
- `Execute`
- `SystemReview`
- `DriftReview`

Exit condition:

- Strong programs compile to IR graphs.
- They pass admissibility.
- They produce plans.
- They execute only through dispatcher/evidence machinery.
- They cannot call tools directly.

## Phase 4 - Close Natural-Language Ingress Through Agent Runtime

Humans should speak naturally. The runtime should receive disciplined candidates.

Phase 4 introduces a different ingress solution than Phase 3.

Phase 3 law:

```text
Strong Grammar compiles.
```

Phase 4 law:

```text
Natural language enters a pocket pipeline runtime.
```

Strong Grammar is compiler input.
Natural language is ingress material.

Natural language must not compile directly into material action. A human message may be ambiguous, incomplete, social, exploratory, joking, approving, objecting, or requesting. It is not itself an act.

The Agent Runtime therefore needs a pocket pipeline runtime: a small constitutional state machine that governs language before it can become IR.

Hard invariant:

```text
Natural language never executes.
```

Natural language may produce:

```text
intent record
candidate
clarification request
rejection
ghost record
```

Natural language may not directly produce:

```text
dispatch
provider mutation
database mutation
evidence closure
direct runtime action
```

The Phase 4 path is:

```text
natural-language message
-> agent.message.received
-> pocket pipeline runtime
   -> interpret intent
   -> classify candidate
   -> record ghosts
   -> determine confirmation/admissibility needs
-> agent.candidate.classified
-> strong_candidate | operational_candidate | clarification_required | rejected
-> IR only if admissible
-> validation/admissibility
-> planning/lowering
-> dispatch/evidence machinery only outside ingress
```

Candidate types:

```text
strong_candidate
operational_candidate
clarification_required
rejected
ghost_record
```

Suggested pocket pipeline runtime states:

```text
Received
Interpreted
CandidateClassified
GhostChecked
CandidateProposed
AdmissibilityPending
AdmittedToIR
Planned
RequiresConfirmation
ReadyForDispatch
Rejected
ClarificationRequired
```

Boundary statements:

```text
The message is not the act.
The candidate is not the act.
The IR is not the act.
Only the constitutional pipeline may produce material action.
```

Do:

- Add or complete `POST /api/agent-runtime/places/:place_id/messages`.
- Ensure the Rust API path does not emit or mutate directly.

Evidence kinds:

- `agent.message.received`
- `agent.candidate.classified`
- `agent.pipeline.admitted`
- `agent.response.emitted`

Mandatory test:

- No agent-runtime output may skip IR, validation, planning, or evidence.

Exit condition:

- A natural-language message produces a reconstructable `ExecutionReport`.
- No agent-runtime path emits directly.

### Phase 4 LLM Roles

Phase 4 officially introduces multiple LLM language roles, but not multiple sovereign agents.

These are authority profiles, not execution authorities.

```text
Maestro LLM
Operator / Translator LLM
Ingress Classifier LLM
```

The architecture may implement these as one model with different prompts, separate models, or swappable modules. The requirement is not three vendors or three processes. The requirement is three constrained authority profiles.

#### Maestro LLM

The Maestro holds the score.

Responsibilities:

```text
preserve doctrine
detect drift
coordinate Operators
enforce Ghost Record discipline
prevent fake closure
summarize receipts back to the Architect
```

Authority:

```text
may govern language
may reject closure
may coordinate
may not execute material acts
may not sign consequence
```

#### Operator / Translator LLM

The Operator is the Translator.

Responsibilities:

```text
translate human meaning into LogLine, Strong candidate, probe, or Ghost Record
draft Strong JSON
propose minimum viable probes
name ghosts and scope boundaries
summarize receipts back to the human
```

Authority:

```text
may propose candidates
may write probes
may draft Strong programs
may not dispatch
may not mutate state directly
may not call provider/database as consequence
may not declare closure without receipt
```

#### Ingress Classifier LLM

The Ingress Classifier is the little LLM inside the pocket pipeline runtime.

Its job is small:

```text
message -> candidate type
```

Candidate types:

```text
strong_candidate
operational_candidate
clarification_required
rejected
ghost_record
```

Authority:

```text
may classify
may normalize
may ask for missing fields
may not plan broadly
may not execute
may not decide closure
```

Core law:

```text
Multiple LLM roles may exist, but no LLM is the Runtime.
```

LLMs may classify, translate, propose, draft, critique, summarize, and explain.

LLMs may not execute, dispatch, mutate provider/database state, declare verified without receipt, or sign consequence they cannot bear.

### Pocket Runtimes For Language Roles

Pocket runtimes are attached to language roles, not only endpoints.

Any LLM role that moves meaning toward action must carry a pocket runtime appropriate to its authority.

The sovereign Constitutional Runtime remains external. Pocket runtimes govern language-state transitions only.

#### Ingress Pocket Runtime

Attached to natural-language entry.

Prevents:

```text
message -> action
```

Forces:

```text
message
-> intent record
-> candidate
-> ghost check
-> admissibility needs
-> IR only if admitted
```

#### Operator Pocket Runtime

Attached to the Operator / Translator.

Prevents:

```text
translation -> truth
candidate -> admitted act
probe proposal -> evidence
confidence -> closure
```

Forces:

```text
received intent
-> parsed meaning
-> candidate drafted
-> ghosts recorded
-> probe proposed
-> scope bound
-> ready for gate | blocked | needs clarification
```

#### Maestro Pocket Runtime

Attached to the Maestro.

Prevents:

```text
mode drift
fake closure
pressure collapse
tool-land swallowing the constitution
```

Forces:

```text
mode classification
state separation
receipt discipline
ghost visibility
desk clean handoff
```

Final Phase 4 pocket-runtime law:

```text
Every LLM that moves meaning toward action must carry a pocket runtime.
```

The Constitutional Runtime remains sovereign.
Pocket runtimes only govern language-state transitions.

## Phase 5 - Wire Business Canon and Elastic Config as Authority

The repo has `minilab-core::business_canon` and `minilab-core::elastic_config`, but they are not yet runtime authority. That is one of the clearest "trying" signals.

Do:

- Add `BusinessCanonContext` to admissibility or adjacent authority validation.
- Load `ElasticOperatingConfig` as runtime envelope defaults.
- Replace hardcoded eligibility and premium stubs with canon/config-backed evaluation.
- Write ADR 11: Business Canon as Constitutional Authority.

Evidence kinds:

- `canon.invariant.evaluated`
- `elastic.config.resolved`

Exit condition:

- `submit_outbound_send` consults canon.
- Config changes affect live admissibility without code change.
- Every live `outbound.send` writes canon/config evidence.

## Phase 6 - Convert `reply.received` Into a Constitutional Slice

`reply.rs` is the largest non-slice runtime surface. It is the biggest remaining ordinary-software-shaped path.

Do:

- Create `slices/reply_received/`.
- Normalize inbound Twilio/SendGrid payloads before admission.
- Move old `reply.rs` to `legacy/reply_legacy.rs`.
- Feature-flag the legacy reply path off in production.

Stations:

- `reply.provider_verified`
- `reply.contact_resolved`
- `reply.thread_routed`
- `reply.received.admitted`
- execution boundary: persist plus downstream notification
- closed evidence

Tests:

- Happy path.
- Signature rejection.
- Unknown-contact path.
- Duplicate-message idempotency.

Exit condition:

- Every inbound reply goes through the slice pipeline.
- Legacy reply path is off in production.

## Phase 7 - Real Executors and Production Evidence

Honest stubs are fine for anatomy. They are not final contact with reality.

Do:

- Real ed25519 executor for `host.pair`.
- Real SendGrid/Twilio executors for `outbound.send`.
- Capability bindings reference implementation anchors or simulation stub IDs.
- Production mode rejects simulation-only stubs.

Crucial rule:

```text
provider success != constitutional closure
```

Provider success is one evidence item. The act closes only when the evidence contract closes.

Exit condition:

- Live `outbound.send` produces real `provider_message_id` and closed evidence.
- Live `host.pair` uses real cryptographic handshake.
- Production cannot run unanchored capability claims.

## Phase 8 - Registry / Operator Evidence Surface

The Shelf becomes real when humans and agents can point at receipts.

Do:

- Build evidence reconstruction endpoint by `correlation_id`.
- Build operator evidence viewer in `places.minilab.work`.
- Add retention/compaction policy.

The UI should show:

- Admission path.
- Policy class.
- Authority/canon result.
- Capability binding.
- Lowered command.
- Execution outcome.
- Evidence closure status.
- Unresolved or unverified remainder.

Exit condition:

- Humans and agents can point at receipts, not stories.
- Evidence graph becomes the product surface.
- Correlation reconstruction returns in under 500 ms.

## Phase 9 - Delete Legacy Substrate

This is the final becoming move.

Delete or permanently quarantine:

- `outreach.rs`
- `campaign.rs`
- `scoring.rs`
- legacy outbound helpers
- legacy reply path

Every removal gets an ADR documenting the replacement slice.

Add regression tests proving direct legacy-style mutation paths are unavailable.

Exit condition:

- New material behavior can only be implemented as a slice.
- Legacy substrate cannot grow.

## Non-Negotiable Invariants

1. No material act bypasses IR.
2. Strong Grammar compiles to IR; it never executes.
3. Agent Runtime classifies/proposes at ingress only; no LLM behind dispatcher.
4. `Decide` never reaches the executive lowerer unresolved.
5. PolicyClass D remains non-executable at the normal runtime boundary.
6. Capability without implementation anchor or valid sim stub is inadmissible.
7. Simulation stubs are rejected in Production.
8. Evidence closure failure means the act is not constitutionally closed.
9. Every live slice reconstructs by `correlation_id`.
10. Every failure has stage / phase / reason code.
11. Canon authority lives outside the substrate-neutral kernel.
12. UI consumes public API JSON, not internal Rust types.
13. Natural language never executes.
14. Natural language may only produce candidates, clarification requests, rejections, intent records, or ghost records.
15. Strong Grammar is compiler input; natural language is ingress material.
16. The Operator is the Translator.
17. The Ingress Classifier classifies messages; it does not translate full meaning or execute.
18. Every LLM role that moves meaning toward action carries a pocket runtime.
19. Pocket runtimes govern language-state transitions only; the Constitutional Runtime remains sovereign.
20. No LLM role sits behind the dispatcher.

## What Not To Do

- Do not add more manifesto instead of gates.
- Do not add a workflow engine.
- Do not let provider success equal constitutional success.
- Do not let legacy modules absorb new behavior.
- Do not add natural-language tool execution before Strong Grammar -> IR is landed.
- Do not split crates before Strong Grammar and Agent Runtime prove different release cadences.
- Do not make `minilab-core` part of the generic runtime kernel.
- Do not put an LLM behind the dispatcher.
- Do not let natural language compile directly into action.
- Do not let candidate classification become execution authority.
- Do not let the Operator's translation become truth without gate, probe, receipt, and scope.
- Do not let pocket runtimes mutate the world; they govern language state only.

## Final Outcome

The work is to turn the constitution from doctrine into monopoly.

The repo already has the right kernel shape:

```text
IR
validation
capability
policy
lowering
dispatch
slices
evidence
```

It is still not fully itself while these gaps remain:

- Legacy substrate still exists.
- Business Canon is not runtime authority yet.
- Elastic Config is not live authority yet.
- Honest stubs remain.
- `install.reconcile` is landed as a Reconcile-shaped slice, but real host-side install execution, rich manifest semantics, and canon/config authority remain open.
- Strong Grammar is closed only as compiler-only v0 JSON ingress through JSON -> AST -> IR -> admissibility -> lowering -> stop before dispatch. Strong-originated execution/evidence closure remains outside Phase 3.
- Agent Runtime ingress is not fully closed through the pocket pipeline runtime.
- `reply.received` is still not a constitutional slice.
- Receipt/registry/operator surface is not first-class.
- Real production executors are incomplete.

When the phases above are done, the system stops trying to be a constitutional runtime. It becomes one.
