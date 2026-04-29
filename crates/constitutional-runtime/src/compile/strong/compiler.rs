//! Strong AST to Constitutional IR graph compiler.
//!
//! This compiler is pure: it allocates IR nodes and edges only. It does not
//! validate, lower, dispatch, write evidence, or mutate external state.

use serde_json::{Map, Value};
use std::fmt;

use super::ast::{StrongProgram, StrongStep};
use crate::act_identity::{CanonicalActionId, IdentityError};
use crate::ir::{ActionKind, IRPrimitive, InferSurface, IrNode, Kind, Role, Schema};
use crate::planning_compiler::{Edge, IrGraph};
use crate::refs::{DataRef, NodeId, PolicyId, SurfaceRef, TargetRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StrongCompileError {
    EmptyPipeline { path: String },
    NestedPipelineAsOperation { path: String },
    UnknownSystemReviewStep { path: String, step: String },
    InvalidExecuteAction { path: String, error: IdentityError },
}

impl fmt::Display for StrongCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrongCompileError::EmptyPipeline { path } => {
                write!(f, "empty Strong pipeline at {path}")
            }
            StrongCompileError::NestedPipelineAsOperation { path } => {
                write!(f, "pipeline cannot be used as a single operation at {path}")
            }
            StrongCompileError::UnknownSystemReviewStep { path, step } => {
                write!(f, "unknown SystemReview pipeline step {step:?} at {path}")
            }
            StrongCompileError::InvalidExecuteAction { path, error } => {
                write!(f, "invalid Execute.action at {path}: {error}")
            }
        }
    }
}

impl std::error::Error for StrongCompileError {}

pub fn compile_strong_program_to_ir_graph(
    program: &StrongProgram,
) -> Result<IrGraph, StrongCompileError> {
    let mut graph = IrGraph::default();
    compile_step_into_graph(&program.root, "$", None, &mut graph)?;
    Ok(graph)
}

fn compile_step_into_graph(
    step: &StrongStep,
    path: &str,
    parent: Option<NodeId>,
    graph: &mut IrGraph,
) -> Result<(), StrongCompileError> {
    match step {
        StrongStep::Pipeline { steps } => {
            if steps.is_empty() {
                return Err(StrongCompileError::EmptyPipeline { path: path.into() });
            }
            for (i, child) in steps.iter().enumerate() {
                compile_step_into_graph(child, &format!("{path}.steps[{i}]"), None, graph)?;
            }
            Ok(())
        }
        StrongStep::SystemReview {
            target,
            pipeline,
            on_success,
        } => {
            if pipeline.is_empty() {
                return Err(StrongCompileError::EmptyPipeline {
                    path: format!("{path}.pipeline"),
                });
            }
            for (i, item) in pipeline.iter().enumerate() {
                let primitive =
                    system_review_step_to_ir(target, item, &format!("{path}.pipeline[{i}]"))?;
                push_node(primitive, parent.clone(), graph);
            }
            if let Some(step) = on_success {
                compile_step_into_graph(step, &format!("{path}.on_success"), None, graph)?;
            }
            Ok(())
        }
        StrongStep::DriftReview {
            target,
            mode,
            on_failure,
        } => {
            let mut params = Map::new();
            params.insert("target".into(), Value::String(target.clone()));
            if let Some(mode) = mode {
                params.insert("mode".into(), Value::String(mode.clone()));
            }
            push_node(
                execute_primitive("flow.drift_review", params, path)?,
                parent.clone(),
                graph,
            );
            if let Some(step) = on_failure {
                compile_step_into_graph(step, &format!("{path}.on_failure"), None, graph)?;
            }
            Ok(())
        }
        other => {
            let primitive = compile_single_operation(other, path)?;
            push_node(primitive, parent, graph);
            Ok(())
        }
    }
}

fn compile_single_operation(
    step: &StrongStep,
    path: &str,
) -> Result<IRPrimitive, StrongCompileError> {
    match step {
        StrongStep::Emit { event } => Ok(IRPrimitive::Emit {
            surface: SurfaceRef(event.clone()),
            payload: DataRef(format!("strong.emit:{event}")),
        }),
        StrongStep::Confirm { subject, role } => Ok(IRPrimitive::Confirm {
            action: Box::new(IRPrimitive::Emit {
                surface: SurfaceRef(subject.clone()),
                payload: DataRef(format!("strong.confirm:{subject}")),
            }),
            role: Role(role.clone()),
        }),
        StrongStep::Execute { action, params } => execute_primitive(action, params.clone(), path),
        StrongStep::OnSuccess { step } => Ok(IRPrimitive::Route {
            operation: Box::new(compile_single_operation(step, &format!("{path}.step"))?),
            surface: SurfaceRef("strong.on_success".into()),
        }),
        StrongStep::OnFailure { step } => Ok(IRPrimitive::Route {
            operation: Box::new(compile_single_operation(step, &format!("{path}.step"))?),
            surface: SurfaceRef("strong.on_failure".into()),
        }),
        StrongStep::Pipeline { .. }
        | StrongStep::SystemReview { .. }
        | StrongStep::DriftReview { .. } => {
            Err(StrongCompileError::NestedPipelineAsOperation { path: path.into() })
        }
    }
}

fn execute_primitive(
    action: &str,
    params: Map<String, Value>,
    path: &str,
) -> Result<IRPrimitive, StrongCompileError> {
    let id = CanonicalActionId::parse(action).map_err(|error| {
        StrongCompileError::InvalidExecuteAction {
            path: path.into(),
            error,
        }
    })?;
    Ok(IRPrimitive::Execute {
        action: ActionKind::Canonical(id),
        params,
    })
}

fn system_review_step_to_ir(
    target: &str,
    step: &str,
    path: &str,
) -> Result<IRPrimitive, StrongCompileError> {
    match step {
        "Collect" => Ok(IRPrimitive::Collect {
            kind: Kind("system.review".into()),
            target: TargetRef(target.into()),
            window: crate::ir::Window("latest".into()),
        }),
        "Compress" => Ok(IRPrimitive::Compress {
            kind: Kind("system.review".into()),
            input_ref: DataRef(format!("strong.system_review:{target}:collect")),
            infer_surface: InferSurface::Local,
        }),
        "Classify" => Ok(IRPrimitive::Classify {
            kind: Kind("system.review".into()),
            input_ref: DataRef(format!("strong.system_review:{target}:compress")),
            schema: Schema("system.review".into()),
            infer_surface: InferSurface::Local,
        }),
        "Prioritize" => Ok(IRPrimitive::Prioritize {
            kind: Kind("system.review".into()),
            input_ref: DataRef(format!("strong.system_review:{target}:classify")),
            policy: PolicyId("system.review".into()),
            infer_surface: InferSurface::Local,
        }),
        other => Err(StrongCompileError::UnknownSystemReviewStep {
            path: path.into(),
            step: other.into(),
        }),
    }
}

fn push_node(primitive: IRPrimitive, parent: Option<NodeId>, graph: &mut IrGraph) {
    let node_id = NodeId(format!("n{}", graph.nodes.len()));
    if let Some(parent) = parent {
        graph.edges.push(Edge {
            parent,
            child: node_id.clone(),
        });
    }
    graph.nodes.push(IrNode {
        id: node_id,
        body: primitive,
    });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::capability::{CapabilityManifest, PrimitiveName};
    use crate::compile::strong::{parse_strong_json, StrongParseError};
    use crate::lowering::{Lowerer, MinilabRuntimeLowerer, OperationalCommand};
    use crate::policy::PolicyClass;
    use crate::validation::{validate_admissibility, AdmissibilityContext};

    const MINIMAL_PIPELINE: &str = r#"{
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
    }"#;

    #[test]
    fn parses_minimal_strong_pipeline_from_json() {
        let program = parse_strong_json(MINIMAL_PIPELINE).unwrap();

        assert!(matches!(program.root, StrongStep::Pipeline { .. }));
    }

    #[test]
    fn parses_each_v0_construct_from_json() {
        for (name, json) in v0_construct_fixtures() {
            parse_strong_json(json).unwrap_or_else(|e| panic!("{name} must parse: {e}"));
        }
    }

    #[test]
    fn compiles_minimal_strong_pipeline_to_ir() {
        let program = parse_strong_json(MINIMAL_PIPELINE).unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();

        assert_eq!(graph.len(), 2);
        assert!(graph.edges.is_empty());
        assert!(matches!(graph.nodes[0].body, IRPrimitive::Emit { .. }));
        assert!(matches!(graph.nodes[1].body, IRPrimitive::Confirm { .. }));
        assert_eq!(graph.nodes[0].id.0, "n0");
        assert_eq!(graph.nodes[1].id.0, "n1");
    }

    #[test]
    fn each_v0_construct_compiles_to_ir_without_direct_execution() {
        for (name, json) in v0_construct_fixtures() {
            let program = parse_strong_json(json).unwrap();
            let graph = compile_strong_program_to_ir_graph(&program)
                .unwrap_or_else(|e| panic!("{name} must compile to IR: {e}"));

            assert!(
                !graph.is_empty(),
                "{name} must produce at least one IR node"
            );
            assert!(
                graph.nodes.iter().all(|node| !node.id.0.is_empty()),
                "{name} must produce stable node ids"
            );
        }
    }

    #[test]
    fn execute_compiles_to_canonical_execute_ir() {
        let program = parse_strong_json(
            r#"{
              "kind": "Execute",
              "action": "demo.run",
              "params": { "correlation_id": "c-1" }
            }"#,
        )
        .unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();

        assert_eq!(graph.len(), 1);
        match &graph.nodes[0].body {
            IRPrimitive::Execute { action, params } => {
                assert_eq!(params.get("correlation_id").unwrap(), "c-1");
                match action {
                    ActionKind::Canonical(id) => assert_eq!(id.dotted_str(), "demo.run"),
                    other => panic!("expected canonical action, got {other:?}"),
                }
            }
            other => panic!("expected Execute IR, got {other:?}"),
        }
    }

    #[test]
    fn branch_constructs_preserve_route_structure_in_ir() {
        for (json, expected_surface) in [
            (
                r#"{
                  "kind": "OnSuccess",
                  "step": { "kind": "Emit", "event": "demo.ok" }
                }"#,
                "strong.on_success",
            ),
            (
                r#"{
                  "kind": "OnFailure",
                  "step": { "kind": "Confirm", "subject": "demo.failed" }
                }"#,
                "strong.on_failure",
            ),
        ] {
            let program = parse_strong_json(json).unwrap();
            let graph = compile_strong_program_to_ir_graph(&program).unwrap();

            assert_eq!(graph.len(), 1);
            match &graph.nodes[0].body {
                IRPrimitive::Route { operation, surface } => {
                    assert_eq!(surface.0, expected_surface);
                    assert!(!matches!(**operation, IRPrimitive::Route { .. }));
                }
                other => panic!("expected Route IR, got {other:?}"),
            }
        }
    }

    #[test]
    fn system_review_compiles_to_review_pipeline_ir_without_runtime_call() {
        let program = parse_strong_json(
            r#"{
              "kind": "SystemReview",
              "target": "lab8gb",
              "pipeline": ["Collect", "Compress", "Classify", "Prioritize"]
            }"#,
        )
        .unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();

        assert_eq!(graph.len(), 4);
        assert!(matches!(graph.nodes[0].body, IRPrimitive::Collect { .. }));
        assert!(matches!(graph.nodes[1].body, IRPrimitive::Compress { .. }));
        assert!(matches!(graph.nodes[2].body, IRPrimitive::Classify { .. }));
        assert!(matches!(
            graph.nodes[3].body,
            IRPrimitive::Prioritize { .. }
        ));
    }

    #[test]
    fn drift_review_compiles_to_canonical_execute_ir_without_runtime_call() {
        let program = parse_strong_json(
            r#"{
              "kind": "DriftReview",
              "target": "lab8gb",
              "mode": "dry_run"
            }"#,
        )
        .unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();

        assert_eq!(graph.len(), 1);
        match &graph.nodes[0].body {
            IRPrimitive::Execute { action, params } => {
                assert_eq!(params.get("target").unwrap(), "lab8gb");
                assert_eq!(params.get("mode").unwrap(), "dry_run");
                match action {
                    ActionKind::Canonical(id) => {
                        assert_eq!(id.dotted_str(), "flow.drift_review");
                    }
                    other => panic!("expected canonical action, got {other:?}"),
                }
            }
            other => panic!("expected Execute IR, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unknown_strong_construct() {
        let err = parse_strong_json(r#"{ "kind": "ToolCall", "name": "send" }"#).unwrap_err();

        assert_eq!(
            err,
            StrongParseError::UnknownConstruct {
                path: "$".into(),
                kind: "ToolCall".into(),
            }
        );
    }

    #[test]
    fn invalid_execute_action_rejects_before_ir_admission() {
        let program = parse_strong_json(
            r#"{
              "kind": "Execute",
              "action": "not-canonical",
              "params": {}
            }"#,
        )
        .unwrap();
        let err = compile_strong_program_to_ir_graph(&program).unwrap_err();

        assert!(matches!(
            err,
            StrongCompileError::InvalidExecuteAction { .. }
        ));
    }

    #[test]
    fn strong_ir_is_accepted_by_existing_admissibility_validation() {
        let program = parse_strong_json(MINIMAL_PIPELINE).unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();
        let manifests = [manifest_for_strong_v0()];
        let ctx = admissibility_ctx();

        for node in &graph.nodes {
            validate_admissibility(node, &manifests, &ctx).unwrap();
        }
    }

    #[test]
    fn representative_strong_programs_pass_admissibility() {
        for (name, json) in representative_v0_programs() {
            let program = parse_strong_json(json).unwrap();
            let graph = compile_strong_program_to_ir_graph(&program).unwrap();
            let manifests = [manifest_for_strong_v0()];
            let ctx = admissibility_ctx();

            for node in &graph.nodes {
                validate_admissibility(node, &manifests, &ctx)
                    .unwrap_or_else(|e| panic!("{name} node {} must admit: {e}", node.id));
            }
        }
    }

    #[test]
    fn strong_ir_produces_lowered_representation_without_dispatch() {
        let program = parse_strong_json(MINIMAL_PIPELINE).unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();
        let commands = validate_and_lower_without_dispatch(&graph);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].namespace, "place");
        assert_eq!(commands[0].verb, "emit");
        assert_eq!(commands[1].namespace, "checkpoint");
        assert_eq!(commands[1].verb, "await");
    }

    #[test]
    fn representative_strong_programs_lower_through_existing_lowerer_without_dispatch() {
        for (name, json) in representative_v0_programs() {
            let program = parse_strong_json(json).unwrap();
            let graph = compile_strong_program_to_ir_graph(&program).unwrap();
            let commands = validate_and_lower_without_dispatch(&graph);

            assert_eq!(
                commands.len(),
                graph.nodes.len(),
                "{name} must lower one command artifact per IR node"
            );
            assert!(
                commands
                    .iter()
                    .all(|cmd| !cmd.namespace.is_empty() && !cmd.verb.is_empty()),
                "{name} must produce inspectable command artifacts"
            );
        }
    }

    #[test]
    fn strong_integration_stops_before_dispatcher_execution() {
        let program = parse_strong_json(MINIMAL_PIPELINE).unwrap();
        let graph = compile_strong_program_to_ir_graph(&program).unwrap();
        let commands = validate_and_lower_without_dispatch(&graph);

        assert!(commands.iter().all(|cmd| !cmd.namespace.is_empty()));
        assert!(commands.iter().all(|cmd| !cmd.verb.is_empty()));
        // The integration proof ends at lowered command artifacts. Execution
        // remains owned by the runtime dispatcher/evidence path outside
        // compile/strong.
        assert_eq!(commands.len(), graph.nodes.len());
    }

    #[test]
    fn invalid_strong_input_rejects_before_ir_admission() {
        let parsed = parse_strong_json(r#"{ "kind": "ToolCall", "name": "send" }"#);

        assert!(matches!(
            parsed,
            Err(StrongParseError::UnknownConstruct { .. })
        ));
    }

    #[test]
    fn strong_compiler_does_not_execute() {
        let strong_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("compile")
            .join("strong");
        let forbidden = [
            concat!("Dis", "patcher"),
            concat!("Evidence", "Store"),
            concat!("execute", "_compiled", "_plan"),
            concat!("pro", "vider"),
            concat!("data", "base"),
            concat!("rus", "qlite"),
            concat!("req", "west"),
            concat!("tokio", "::", "process"),
            concat!("std", "::", "process"),
        ];

        for file in ["mod.rs", "ast.rs", "parser.rs", "compiler.rs"] {
            let body = fs::read_to_string(strong_dir.join(file)).unwrap();
            for token in forbidden {
                assert!(
                    !body.contains(token),
                    "compile/strong/{file} must not introduce execution token {token}"
                );
            }
        }
    }

    fn validate_and_lower_without_dispatch(graph: &IrGraph) -> Vec<OperationalCommand> {
        let manifests = [manifest_for_strong_v0()];
        let ctx = admissibility_ctx();
        let lowerer = MinilabRuntimeLowerer;
        let mut commands = Vec::new();

        for node in &graph.nodes {
            validate_admissibility(node, &manifests, &ctx).unwrap();
            let (_plan, command) = lowerer.lower(node).unwrap();
            commands.push(command);
        }

        commands
    }

    fn v0_construct_fixtures() -> [(&'static str, &'static str); 8] {
        [
            (
                "Pipeline",
                r#"{
                  "kind": "Pipeline",
                  "steps": [{ "kind": "Emit", "event": "demo.started" }]
                }"#,
            ),
            ("Emit", r#"{ "kind": "Emit", "event": "demo.started" }"#),
            (
                "Confirm",
                r#"{ "kind": "Confirm", "subject": "demo.confirmed" }"#,
            ),
            (
                "Execute",
                r#"{ "kind": "Execute", "action": "demo.run", "params": { "id": "1" } }"#,
            ),
            (
                "OnSuccess",
                r#"{
                  "kind": "OnSuccess",
                  "step": { "kind": "Emit", "event": "demo.ok" }
                }"#,
            ),
            (
                "OnFailure",
                r#"{
                  "kind": "OnFailure",
                  "step": { "kind": "Confirm", "subject": "demo.failed" }
                }"#,
            ),
            (
                "SystemReview",
                r#"{
                  "kind": "SystemReview",
                  "target": "lab8gb",
                  "pipeline": ["Collect", "Compress", "Classify", "Prioritize"]
                }"#,
            ),
            (
                "DriftReview",
                r#"{ "kind": "DriftReview", "target": "lab8gb", "mode": "dry_run" }"#,
            ),
        ]
    }

    fn representative_v0_programs() -> [(&'static str, &'static str); 5] {
        [
            ("minimal pipeline", MINIMAL_PIPELINE),
            (
                "execute",
                r#"{ "kind": "Execute", "action": "demo.run", "params": { "id": "1" } }"#,
            ),
            (
                "branches",
                r#"{
                  "kind": "Pipeline",
                  "steps": [
                    {
                      "kind": "OnSuccess",
                      "step": { "kind": "Emit", "event": "demo.ok" }
                    },
                    {
                      "kind": "OnFailure",
                      "step": { "kind": "Confirm", "subject": "demo.failed" }
                    }
                  ]
                }"#,
            ),
            (
                "system review",
                r#"{
                  "kind": "SystemReview",
                  "target": "lab8gb",
                  "pipeline": ["Collect", "Compress", "Classify", "Prioritize"]
                }"#,
            ),
            (
                "drift review",
                r#"{ "kind": "DriftReview", "target": "lab8gb", "mode": "dry_run" }"#,
            ),
        ]
    }

    fn admissibility_ctx() -> AdmissibilityContext {
        AdmissibilityContext {
            policy_class: PolicyClass::C,
            runtime_permitted: true,
            at_execution_boundary: true,
            require_evidence_closure: true,
        }
    }

    fn manifest_for_strong_v0() -> CapabilityManifest {
        CapabilityManifest {
            substrate_id: "strong-integration-test".into(),
            substrate_version: "1".into(),
            supported_primitives: BTreeSet::from([
                PrimitiveName::Collect,
                PrimitiveName::Compress,
                PrimitiveName::Classify,
                PrimitiveName::Prioritize,
                PrimitiveName::Route,
                PrimitiveName::Execute,
                PrimitiveName::Emit,
                PrimitiveName::Confirm,
            ]),
            declared_guarantees: BTreeSet::from(["evidence.write".into()]),
            ..Default::default()
        }
    }
}
