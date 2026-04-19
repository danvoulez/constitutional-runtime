//! Executive lowering: IR to operational grammar with explicit plans and errors.

use crate::capability::PrimitiveName;
use crate::evidence::EvidenceContract;
use crate::ir::{IRPrimitive, IrNode};
use crate::refs::NodeId;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use thiserror::Error;

/// Where this command is meant to run (operational surface).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTarget {
    MinilabOperationalGrammar,
    Mcp,
    Shell,
    Cloud,
}

/// **Canonical** materialization of intent for execution: `namespace`, `verb`, and ordered args.
///
/// Every path that reaches a host or tool should use this single type — including the decision
/// pipeline after [`crate::decision::compile_node`] (still produced only by
/// [`MinilabRuntimeLowerer`], not a parallel command model).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OperationalCommand {
    pub namespace: String,
    pub verb: String,
    pub args: BTreeMap<String, Value>,
    pub target_runtime: RuntimeTarget,
}

/// Risk and closure metadata produced alongside a command.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoweringPlan {
    pub requires_confirmation: bool,
    pub estimated_latency_ms: Option<u64>,
    pub evidence: EvidenceContract,
}

#[derive(Debug, Error)]
pub enum LoweringError {
    #[error("IR `{0}` cannot be lowered to operational grammar on this boundary: {1}")]
    NotLowerableToOperationalGrammar(NodeId, String),
}

pub trait Lowerer {
    fn lower(&self, node: &IrNode) -> Result<(LoweringPlan, OperationalCommand), LoweringError>;
}

/// Minilab boundary compiler: pure translator, not a planner. Does **not** lower [`IRPrimitive::Decide`].
#[derive(Clone, Debug, Default)]
pub struct MinilabRuntimeLowerer;

impl MinilabRuntimeLowerer {
    /// Serialize args deterministically as `k=v` lines for low-entropy grammars.
    pub fn render_pairs(args: &BTreeMap<String, Value>) -> Vec<String> {
        let mut out = Vec::new();
        for (k, v) in args {
            let rendered = Self::render_value(v);
            let needs_quotes = rendered.contains(' ') || rendered.contains('\t');
            let val = if needs_quotes {
                format!("\"{}\"", rendered.replace('\"', "\\\""))
            } else {
                rendered
            };
            out.push(format!("{k}={val}"));
        }
        out
    }

    fn render_value(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            _ => v.to_string(),
        }
    }

    fn lower_primitive(
        &self,
        node_id: &NodeId,
        p: &IRPrimitive,
    ) -> Result<(LoweringPlan, OperationalCommand), LoweringError> {
        match p {
            IRPrimitive::Decide { .. } => Err(LoweringError::NotLowerableToOperationalGrammar(
                node_id.clone(),
                "Decide must lower through a higher-order planner or flow compiler".into(),
            )),

            // --- Intelligence loop ---
            IRPrimitive::Observe { target, scope } => {
                let mut args = BTreeMap::new();
                args.insert("target".into(), json!(target.0.clone()));
                args.insert("scope".into(), json!(scope.clone()));
                Ok((
                    plan(false, Some(50), vec!["host.snapshot"]),
                    cmd("host", "inspect", args),
                ))
            }
            IRPrimitive::Collect { kind, target, window } => {
                let mut args = BTreeMap::new();
                args.insert("kind".into(), json!(kind.0.clone()));
                args.insert("target".into(), json!(target.0.clone()));
                args.insert("window".into(), json!(window.0.clone()));
                Ok((
                    plan(false, Some(120), vec!["events.slice"]),
                    cmd("events", "collect", args),
                ))
            }
            IRPrimitive::Fetch { kind, id } => {
                let mut args = BTreeMap::new();
                args.insert("kind".into(), json!(kind.0.clone()));
                args.insert("id".into(), json!(id.clone()));
                Ok((
                    plan(false, Some(40), vec!["events.record"]),
                    cmd("events", "fetch", args),
                ))
            }
            IRPrimitive::Compress {
                kind,
                input_ref,
                infer_surface,
            } => {
                let mut args = BTreeMap::new();
                args.insert("kind".into(), json!(kind.0.clone()));
                args.insert("input_ref".into(), json!(input_ref.0.clone()));
                args.insert("infer_surface".into(), json!(infer_surface));
                Ok((
                    plan(false, Some(300), vec!["intel.compress"]),
                    cmd("intel", "compress", args),
                ))
            }
            IRPrimitive::Classify {
                kind,
                input_ref,
                schema,
            } => {
                let mut args = BTreeMap::new();
                args.insert("kind".into(), json!(kind.0.clone()));
                args.insert("input_ref".into(), json!(input_ref.0.clone()));
                args.insert("schema".into(), json!(schema.0.clone()));
                Ok((
                    plan(false, Some(250), vec!["intel.labels"]),
                    cmd("intel", "classify", args),
                ))
            }
            IRPrimitive::Prioritize {
                kind,
                input_ref,
                policy,
            } => {
                let mut args = BTreeMap::new();
                args.insert("kind".into(), json!(kind.0.clone()));
                args.insert("input_ref".into(), json!(input_ref.0.clone()));
                args.insert("policy".into(), json!(policy.0.clone()));
                Ok((
                    plan(false, Some(180), vec!["intel.rank"]),
                    cmd("intel", "prioritize", args),
                ))
            }
            IRPrimitive::Compare { kind, left, right } => {
                let mut args = BTreeMap::new();
                args.insert("kind".into(), json!(kind.0.clone()));
                args.insert("left".into(), json!(left.0.clone()));
                args.insert("right".into(), json!(right.0.clone()));
                Ok((
                    plan(false, Some(150), vec!["intel.diff"]),
                    cmd("intel", "compare", args),
                ))
            }

            // --- Bounded routing / scheduling (no nested lowering) ---
            IRPrimitive::Route { operation, surface } => {
                let routed = PrimitiveName::from_primitive(operation);
                let mut args = BTreeMap::new();
                args.insert("surface".into(), json!(surface.0.clone()));
                args.insert("routed_primitive".into(), json!(routed.to_string()));
                Ok((
                    plan(false, Some(20), vec!["routing.decision"]),
                    cmd("routing", "surface", args),
                ))
            }
            IRPrimitive::Schedule { action, trigger } => {
                let action_prim = PrimitiveName::from_primitive(action);
                let mut args = BTreeMap::new();
                args.insert("trigger".into(), json!(trigger.0.clone()));
                args.insert("action_primitive".into(), json!(action_prim.to_string()));
                Ok((
                    plan(false, Some(25), vec!["runtime.schedule"]),
                    cmd("runtime", "schedule", args),
                ))
            }

            // --- Governance ---
            IRPrimitive::Confirm { action, role } => {
                let inner = PrimitiveName::from_primitive(action);
                let mut args = BTreeMap::new();
                args.insert("role".into(), json!(role.0.clone()));
                args.insert("inner_primitive".into(), json!(inner.to_string()));
                args.insert(
                    "checkpoint_ref".into(),
                    json!(format!("{}:{}", node_id.0, inner)),
                );
                Ok((
                    plan(true, Some(10), vec!["checkpoint.open", "authority.envelope"]),
                    cmd("checkpoint", "await", args),
                ))
            }
            IRPrimitive::Persist { data, durability } => {
                let mut args = BTreeMap::new();
                args.insert("data_ref".into(), json!(data.0.clone()));
                args.insert("durability".into(), json!(durability));
                Ok((
                    plan(false, Some(80), vec!["store.write"]),
                    cmd("store", "write_intent", args),
                ))
            }

            // --- Action / effect ---
            IRPrimitive::Execute { action, params } => {
                let mut args: BTreeMap<String, Value> = BTreeMap::new();
                for (k, v) in params {
                    args.insert(k.clone(), v.clone());
                }
                let (ns, verb) = match action {
                    crate::ir::ActionKind::HostReconcile => ("host", "reconcile"),
                    crate::ir::ActionKind::Named(n) => {
                        let parts: Vec<&str> = n.splitn(2, '.').collect();
                        if parts.len() == 2 {
                            (parts[0], parts[1])
                        } else {
                            ("cmd", n.as_str())
                        }
                    }
                    crate::ir::ActionKind::Custom(c) => ("cmd", c.as_str()),
                };
                Ok((
                    plan(
                        matches!(action, crate::ir::ActionKind::HostReconcile),
                        Some(200),
                        vec!["exec.result"],
                    ),
                    cmd(ns, verb, args),
                ))
            }
            IRPrimitive::Reconcile { target, desired, mode } => {
                let mut args = BTreeMap::new();
                args.insert("target".into(), json!(target.0.clone()));
                args.insert("desired_ref".into(), json!(desired.0.clone()));
                args.insert("mode".into(), json!(mode));
                Ok((
                    plan(true, Some(500), vec!["reconcile.diff", "exec.result"]),
                    cmd("host", "reconcile", args),
                ))
            }
            IRPrimitive::Emit { surface, payload } => {
                let mut args = BTreeMap::new();
                args.insert("surface".into(), json!(surface.0.clone()));
                args.insert("payload_ref".into(), json!(payload.0.clone()));
                Ok((plan(false, Some(30), vec!["emit.ack"]), cmd("place", "emit", args)))
            }
            IRPrimitive::Cancel { id } => {
                let mut args = BTreeMap::new();
                args.insert("id".into(), json!(id.clone()));
                Ok((
                    plan(true, Some(20), vec!["cancel.ack"]),
                    cmd("work", "cancel", args),
                ))
            }
        }
    }
}

fn plan(
    requires_confirmation: bool,
    ms: Option<u64>,
    kinds: Vec<&'static str>,
) -> LoweringPlan {
    LoweringPlan {
        requires_confirmation,
        estimated_latency_ms: ms,
        evidence: EvidenceContract {
            required_kinds: kinds.iter().map(|s| (*s).to_string()).collect(),
        },
    }
}

fn cmd(ns: &str, verb: &str, args: BTreeMap<String, Value>) -> OperationalCommand {
    OperationalCommand {
        namespace: ns.to_string(),
        verb: verb.to_string(),
        args,
        target_runtime: RuntimeTarget::MinilabOperationalGrammar,
    }
}

impl Lowerer for MinilabRuntimeLowerer {
    fn lower(&self, node: &IrNode) -> Result<(LoweringPlan, OperationalCommand), LoweringError> {
        self.lower_primitive(&node.id, &node.body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ActionKind, InferSurface, IRPrimitive, Kind, Schema, Window};
    use crate::refs::{DataRef, NodeId, PolicyId, SurfaceRef, TargetRef};

    fn node(id: &str, body: IRPrimitive) -> IrNode {
        IrNode {
            id: NodeId(id.into()),
            body,
        }
    }

    #[test]
    fn decide_is_not_lowerable() {
        let n = node(
            "n1",
            IRPrimitive::Decide {
                context: DataRef("c".into()),
                policy: PolicyId("p".into()),
            },
        );
        assert!(MinilabRuntimeLowerer.lower(&n).is_err());
    }

    #[test]
    fn intelligence_lowering_roundtrip_tags() {
        let lowerer = MinilabRuntimeLowerer;
        let cases = vec![
            (
                IRPrimitive::Collect {
                    kind: Kind("events".into()),
                    target: TargetRef("lab8gb".into()),
                    window: Window("24h".into()),
                },
                ("events", "collect"),
            ),
            (
                IRPrimitive::Fetch {
                    kind: Kind("cmd".into()),
                    id: "c42".into(),
                },
                ("events", "fetch"),
            ),
            (
                IRPrimitive::Compress {
                    kind: Kind("logs".into()),
                    input_ref: DataRef("r1".into()),
                    infer_surface: InferSurface::Local,
                },
                ("intel", "compress"),
            ),
            (
                IRPrimitive::Classify {
                    kind: Kind("logs".into()),
                    input_ref: DataRef("r1".into()),
                    schema: Schema("failures".into()),
                },
                ("intel", "classify"),
            ),
            (
                IRPrimitive::Prioritize {
                    kind: Kind("tasks".into()),
                    input_ref: DataRef("r1".into()),
                    policy: PolicyId("attention".into()),
                },
                ("intel", "prioritize"),
            ),
            (
                IRPrimitive::Compare {
                    kind: Kind("state".into()),
                    left: DataRef("a".into()),
                    right: DataRef("b".into()),
                },
                ("intel", "compare"),
            ),
        ];
        for (prim, (ns, verb)) in cases {
            let (_, cmd) = lowerer.lower(&node("t", prim)).unwrap();
            assert_eq!(cmd.namespace, ns);
            assert_eq!(cmd.verb, verb);
        }
    }

    #[test]
    fn route_is_surface_only() {
        let inner = IRPrimitive::Observe {
            target: TargetRef("h".into()),
            scope: "s".into(),
        };
        let n = node(
            "r",
            IRPrimitive::Route {
                operation: Box::new(inner),
                surface: SurfaceRef("slack".into()),
            },
        );
        let (_, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert_eq!((cmd.namespace.as_str(), cmd.verb.as_str()), ("routing", "surface"));
        assert!(cmd.args.contains_key("routed_primitive"));
        assert!(!cmd.args.contains_key("target"));
    }

    #[test]
    fn schedule_is_bounded() {
        let n = node(
            "s",
            IRPrimitive::Schedule {
                action: Box::new(IRPrimitive::Emit {
                    surface: SurfaceRef("t".into()),
                    payload: DataRef("p".into()),
                }),
                trigger: crate::ir::Trigger("cron:0 * * * *".into()),
            },
        );
        let (_, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert_eq!((cmd.namespace.as_str(), cmd.verb.as_str()), ("runtime", "schedule"));
    }

    #[test]
    fn confirm_is_checkpoint() {
        let n = node(
            "c",
            IRPrimitive::Confirm {
                action: Box::new(IRPrimitive::Execute {
                    action: ActionKind::Named("host.restart".into()),
                    params: Default::default(),
                }),
                role: crate::ir::Role("admin".into()),
            },
        );
        let (plan, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert!(plan.requires_confirmation);
        assert_eq!((cmd.namespace.as_str(), cmd.verb.as_str()), ("checkpoint", "await"));
    }

    #[test]
    fn persist_is_store_intent() {
        let n = node(
            "p",
            IRPrimitive::Persist {
                data: DataRef("blob:1".into()),
                durability: crate::ir::DurabilityClass::Durable,
            },
        );
        let (_, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert_eq!((cmd.namespace.as_str(), cmd.verb.as_str()), ("store", "write_intent"));
    }

    /// Stable `namespace.verb` + deterministic `k=v` rendering (golden / regression).
    #[test]
    fn golden_collect_operational_shape() {
        let n = node(
            "g1",
            IRPrimitive::Collect {
                kind: Kind("events".into()),
                target: TargetRef("lab8gb".into()),
                window: Window("24h".into()),
            },
        );
        let (_, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert_eq!(cmd.namespace, "events");
        assert_eq!(cmd.verb, "collect");
        let pairs = MinilabRuntimeLowerer::render_pairs(&cmd.args);
        assert_eq!(
            pairs,
            vec![
                "kind=events".to_string(),
                "target=lab8gb".to_string(),
                "window=24h".to_string(),
            ]
        );
    }

    #[test]
    fn golden_routing_surface_only_args() {
        let n = node(
            "g2",
            IRPrimitive::Route {
                operation: Box::new(IRPrimitive::Collect {
                    kind: Kind("events".into()),
                    target: TargetRef("h".into()),
                    window: Window("1h".into()),
                }),
                surface: SurfaceRef("thread:9".into()),
            },
        );
        let (_, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert_eq!((cmd.namespace.as_str(), cmd.verb.as_str()), ("routing", "surface"));
        let pairs = MinilabRuntimeLowerer::render_pairs(&cmd.args);
        assert_eq!(
            pairs,
            vec![
                "routed_primitive=COLLECT".to_string(),
                "surface=thread:9".to_string(),
            ]
        );
    }

    #[test]
    fn golden_checkpoint_not_substrate_specific() {
        let n = node(
            "g3",
            IRPrimitive::Confirm {
                action: Box::new(IRPrimitive::Observe {
                    target: TargetRef("h".into()),
                    scope: "facts".into(),
                }),
                role: crate::ir::Role("admin".into()),
            },
        );
        let (_, cmd) = MinilabRuntimeLowerer.lower(&n).unwrap();
        assert_eq!((cmd.namespace.as_str(), cmd.verb.as_str()), ("checkpoint", "await"));
        assert!(cmd.args.contains_key("inner_primitive"));
        assert!(cmd.args.contains_key("role"));
        assert!(cmd.args.contains_key("checkpoint_ref"));
        assert!(!cmd.args.contains_key("postgres"));
    }
}
