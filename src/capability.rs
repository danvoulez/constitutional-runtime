//! Capability manifests: material jurisdiction (what a substrate can realize).

use crate::ir::IRPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Declared support for IR primitives on a specific substrate (runtime, MCP, shell, …).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct CapabilityManifest {
    pub substrate_id: String,
    pub substrate_version: String,
    /// Which IR primitives this substrate can execute or assist.
    pub supported_primitives: BTreeSet<PrimitiveName>,
    /// If empty, any `Kind` is accepted. If non-empty, `kind` on primitive must match.
    #[serde(default)]
    pub supported_kinds: BTreeSet<String>,
    /// Tags this substrate claims (e.g. `evidence.write`, `append_only`). Used for realizability checks.
    #[serde(default)]
    pub declared_guarantees: BTreeSet<String>,
}

/// Stable name for [`crate::ir::IRPrimitive`] variants (for manifest matching).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimitiveName {
    Observe,
    Collect,
    Fetch,
    Compress,
    Classify,
    Prioritize,
    Compare,
    Decide,
    Route,
    Schedule,
    Execute,
    Reconcile,
    Emit,
    Persist,
    Confirm,
    Cancel,
}

impl PrimitiveName {
    pub fn from_primitive(p: &IRPrimitive) -> Self {
        match p {
            IRPrimitive::Observe { .. } => PrimitiveName::Observe,
            IRPrimitive::Collect { .. } => PrimitiveName::Collect,
            IRPrimitive::Fetch { .. } => PrimitiveName::Fetch,
            IRPrimitive::Compress { .. } => PrimitiveName::Compress,
            IRPrimitive::Classify { .. } => PrimitiveName::Classify,
            IRPrimitive::Prioritize { .. } => PrimitiveName::Prioritize,
            IRPrimitive::Compare { .. } => PrimitiveName::Compare,
            IRPrimitive::Decide { .. } => PrimitiveName::Decide,
            IRPrimitive::Route { .. } => PrimitiveName::Route,
            IRPrimitive::Schedule { .. } => PrimitiveName::Schedule,
            IRPrimitive::Execute { .. } => PrimitiveName::Execute,
            IRPrimitive::Reconcile { .. } => PrimitiveName::Reconcile,
            IRPrimitive::Emit { .. } => PrimitiveName::Emit,
            IRPrimitive::Persist { .. } => PrimitiveName::Persist,
            IRPrimitive::Confirm { .. } => PrimitiveName::Confirm,
            IRPrimitive::Cancel { .. } => PrimitiveName::Cancel,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveName::Observe => "OBSERVE",
            PrimitiveName::Collect => "COLLECT",
            PrimitiveName::Fetch => "FETCH",
            PrimitiveName::Compress => "COMPRESS",
            PrimitiveName::Classify => "CLASSIFY",
            PrimitiveName::Prioritize => "PRIORITIZE",
            PrimitiveName::Compare => "COMPARE",
            PrimitiveName::Decide => "DECIDE",
            PrimitiveName::Route => "ROUTE",
            PrimitiveName::Schedule => "SCHEDULE",
            PrimitiveName::Execute => "EXECUTE",
            PrimitiveName::Reconcile => "RECONCILE",
            PrimitiveName::Emit => "EMIT",
            PrimitiveName::Persist => "PERSIST",
            PrimitiveName::Confirm => "CONFIRM",
            PrimitiveName::Cancel => "CANCEL",
        }
    }
}

impl std::fmt::Display for PrimitiveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Extracts the kind field when present for capability matching.
pub fn primitive_kind(p: &IRPrimitive) -> Option<&str> {
    match p {
        IRPrimitive::Collect { kind, .. }
        | IRPrimitive::Fetch { kind, .. }
        | IRPrimitive::Compress { kind, .. }
        | IRPrimitive::Classify { kind, .. }
        | IRPrimitive::Prioritize { kind, .. }
        | IRPrimitive::Compare { kind, .. } => Some(kind.0.as_str()),
        _ => None,
    }
}

impl CapabilityManifest {
    pub fn can_realize(&self, p: &IRPrimitive) -> bool {
        self.supported_primitives
            .contains(&PrimitiveName::from_primitive(p))
    }

    /// Kind constraint: empty `supported_kinds` means wildcard.
    pub fn kind_allowed(&self, p: &IRPrimitive) -> bool {
        if self.supported_kinds.is_empty() {
            return true;
        }
        match primitive_kind(p) {
            Some(k) => self.supported_kinds.contains(k),
            None => true,
        }
    }

    /// Evidence realizability: if any guarantee is required for execution, substrate must declare `evidence.write`.
    pub fn evidence_realizable(&self, require_evidence: bool) -> bool {
        if !require_evidence {
            return true;
        }
        self.declared_guarantees.contains("evidence.write")
    }
}
