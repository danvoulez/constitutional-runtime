//! Constitutional runtime core types for Minilab-style systems.
//!
//! Execution is not sovereign: material actions must be semantically admissible,
//! policy-permitted, capability-realizable, and evidentially accountable.
//!
//! See `docs/runtime/constitutional-runtime.md` for the full definition.

pub mod compile;
pub mod contracts;
pub mod evidence;
pub mod execute;
pub mod ir;

pub mod act_identity {
    pub use crate::ir::action_identity::*;
}
pub mod capability {
    pub use crate::contracts::capability::*;
}
pub mod decision {
    pub use crate::compile::decision::*;
}
#[cfg(feature = "sqlite-evidence")]
pub mod evidence_sqlite {
    pub use crate::evidence::sqlite::*;
}
#[cfg(feature = "supabase-evidence")]
pub mod evidence_supabase {
    pub use crate::evidence::supabase::*;
}
pub mod failure {
    pub use crate::contracts::failure::*;
}
pub mod idempotency {
    pub use crate::contracts::idempotency::*;
}
pub mod ingress {
    pub use crate::contracts::ingress::*;
}
pub mod lowering {
    pub use crate::compile::lowering::*;
}
pub mod operational_grammar {
    pub use crate::compile::operational_grammar::*;
}
pub mod plan_executor {
    pub use crate::execute::plan_executor::*;
}
pub mod planning_compiler {
    pub use crate::compile::planning_compiler::*;
}
pub mod policy {
    pub use crate::contracts::policy::*;
}
pub mod refs {
    pub use crate::ir::refs::*;
}
pub mod validation {
    pub use crate::contracts::validation::*;
}

pub use act_identity::{CanonicalActionId, IdentityError};
pub use capability::{
    primitive_kind, CapabilityBinding, CapabilityManifest, CostEnvelope, EvidenceGuarantee,
    GuaranteeEnvelope, LatencyEnvelope, PrimitiveName,
};
pub use compile::strong::{
    compile_strong_program_to_ir_graph, parse_strong_json, StrongCompileError, StrongParseError,
    StrongProgram, StrongStep,
};
pub use decision::{
    assert_decide_free, compile_flow, compile_node, contains_decide, lower_compiled_flow,
    materialize_primitive, resolve_lower_one, DecideResolver, PlannerError, PlannerLoweringError,
};
pub use evidence::{
    close_execution_evidence, EvidenceContract, EvidenceRecord, EvidenceStore, EvidenceStoreError,
    FailureToClose, FileEvidenceStore,
};
pub use failure::{FailurePhase, FailureStage, PolicyClassTag, RuntimeFailure};
pub use idempotency::{IdempotencyClass, IdempotencyContract, ReplayStance};
pub use ingress::IngressMode;
pub use ir::{
    ActionKind, DurabilityClass, IRPrimitive, InferSurface, IrNode, Kind, ReconcileMode, Role,
    Schema, Trigger, Window,
};
pub use lowering::{
    Lowerer, LoweringError, LoweringPlan, MinilabRuntimeLowerer, OperationalCommand, RuntimeTarget,
};
pub use operational_grammar::{
    parse_line, parse_program, ArgValue, IrLoweringError, OperationalEntry, OperationalLine,
    OperationalProgram, ParseError, ParseErrorKind,
};
pub use plan_executor::{
    execute_compiled_plan, execute_compiled_plan_async, AsyncDispatcher, DispatchOutcome,
    Dispatcher, ExecutionReport, NodeExecutionResult, NodeOutcome,
};
pub use planning_compiler::{
    compile_program_to_ir_graph, plan_operational_program, CompileError, CompiledOperationalPlan,
    Edge, IrGraph, NodePlan, PlanError,
};
pub use policy::PolicyClass;
pub use refs::{DataRef, NodeId, PolicyId, SurfaceRef, TargetRef};
pub use validation::{
    check_capability, validate_admissibility, validate_capability, validate_policy,
    validate_structure, AdmissibilityContext, AdmissibleNode, ValidationError,
    MAX_ROUTE_NESTING_DEPTH,
};

#[cfg(feature = "sqlite-evidence")]
pub use evidence_sqlite::SqliteEvidenceStore;
#[cfg(feature = "supabase-evidence")]
pub use evidence_supabase::SupabaseRestEvidenceStore;
