//! Constitutional runtime core types for Minilab-style systems.
//!
//! Execution is not sovereign: material actions must be semantically admissible,
//! policy-permitted, capability-realizable, and evidentially accountable.
//!
//! See `docs/constitutional-runtime.md` for the full definition.

pub mod capability;
pub mod decision;
pub mod evidence;
#[cfg(feature = "sqlite-evidence")]
pub mod evidence_sqlite;
#[cfg(feature = "supabase-evidence")]
pub mod evidence_supabase;
pub mod ingress;
pub mod ir;
pub mod lowering;
pub mod policy;
pub mod refs;
pub mod validation;

pub use capability::{primitive_kind, CapabilityManifest, PrimitiveName};
pub use decision::{
    assert_decide_free, compile_flow, compile_node, contains_decide, lower_compiled_flow,
    materialize_primitive, resolve_lower_one, DecideResolver, PlannerError, PlannerLoweringError,
};
pub use evidence::{
    close_execution_evidence, EvidenceContract, EvidenceRecord, EvidenceStore, EvidenceStoreError,
    FailureToClose, FileEvidenceStore,
};
pub use ingress::IngressMode;
pub use ir::{
    ActionKind, DurabilityClass, InferSurface, IRPrimitive, IrNode, Kind, ReconcileMode, Role, Schema,
    Trigger, Window,
};
pub use lowering::{
    Lowerer, LoweringError, LoweringPlan, MinilabRuntimeLowerer, OperationalCommand, RuntimeTarget,
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
