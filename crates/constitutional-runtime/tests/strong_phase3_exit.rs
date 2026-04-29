//! Phase 3 exit harness: Strong Grammar is compiler-only ingress.
//!
//! This test proves the Phase 3 boundary:
//!
//! ```text
//! Strong JSON -> Strong AST -> IrGraph -> admissibility -> lowering artifact
//! ```
//!
//! It intentionally stops before dispatcher execution and evidence closure.

use constitutional_runtime::{
    compile_strong_program_to_ir_graph, parse_strong_json, validate_admissibility,
    AdmissibilityContext, CapabilityManifest, IRPrimitive, Lowerer, MinilabRuntimeLowerer,
    OperationalCommand, PolicyClass, PrimitiveName, StrongStep,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const FULL_V0_STRONG_PROGRAM: &str = r#"{
  "kind": "Pipeline",
  "steps": [
    {
      "kind": "Emit",
      "event": "demo.started"
    },
    {
      "kind": "Confirm",
      "subject": "demo.confirmed"
    },
    {
      "kind": "Execute",
      "action": "demo.run",
      "params": {
        "correlation_id": "phase3-exit"
      }
    },
    {
      "kind": "OnSuccess",
      "step": {
        "kind": "Emit",
        "event": "demo.succeeded"
      }
    },
    {
      "kind": "OnFailure",
      "step": {
        "kind": "Confirm",
        "subject": "demo.failed"
      }
    },
    {
      "kind": "SystemReview",
      "target": "lab8gb",
      "pipeline": ["Collect", "Compress", "Classify", "Prioritize"]
    },
    {
      "kind": "DriftReview",
      "target": "lab8gb",
      "mode": "dry_run"
    }
  ]
}"#;

#[test]
fn phase3_exit_full_v0_strong_program_reaches_lowering_and_stops_before_dispatch() {
    let program = parse_strong_json(FULL_V0_STRONG_PROGRAM).expect("full v0 Strong JSON parses");
    assert!(matches!(program.root, StrongStep::Pipeline { .. }));

    let graph =
        compile_strong_program_to_ir_graph(&program).expect("full v0 Strong AST compiles to IR");
    assert_eq!(graph.len(), 10, "all v0 constructs must produce IR nodes");
    assert!(
        graph
            .nodes
            .iter()
            .any(|n| matches!(n.body, IRPrimitive::Route { .. })),
        "OnSuccess/OnFailure must preserve branch structure as Route IR"
    );
    assert!(
        graph
            .nodes
            .iter()
            .any(|n| matches!(n.body, IRPrimitive::Execute { .. })),
        "Execute and DriftReview must compile to Execute IR artifacts"
    );

    let manifests = [phase3_manifest()];
    let ctx = phase3_admissibility_ctx();
    let lowerer = MinilabRuntimeLowerer;
    let mut commands: Vec<OperationalCommand> = Vec::new();

    for node in &graph.nodes {
        validate_admissibility(node, &manifests, &ctx)
            .unwrap_or_else(|e| panic!("{} must pass admissibility: {e}", node.id));
        let (_plan, command) = lowerer
            .lower(node)
            .unwrap_or_else(|e| panic!("{} must lower to command artifact: {e}", node.id));
        commands.push(command);
    }

    assert_eq!(commands.len(), graph.nodes.len());
    assert!(
        commands
            .iter()
            .all(|command| !command.namespace.is_empty() && !command.verb.is_empty()),
        "lowering produces inert command artifacts for the existing runtime path"
    );
    assert!(
        commands
            .iter()
            .any(|command| command.namespace == "flow" && command.verb == "drift_review"),
        "DriftReview lowers through existing machinery, not through a Strong-side executor"
    );

    // Stop here. No dispatcher is constructed or called in this harness.
}

#[test]
fn phase3_exit_invalid_strong_input_rejects_before_ir_admission() {
    let parsed = parse_strong_json(r#"{ "kind": "ToolCall", "name": "send" }"#);

    assert!(parsed.is_err());
}

#[test]
fn phase3_exit_compile_strong_production_surface_has_no_direct_execution_path() {
    let strong_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("compile")
        .join("strong");
    let forbidden = [
        "Dispatcher",
        "EvidenceStore",
        "execute_compiled_plan",
        "provider",
        "database",
        "rusqlite",
        "reqwest",
        "tokio::process",
        "std::process",
        "Command",
    ];

    for file in ["mod.rs", "ast.rs", "parser.rs", "compiler.rs"] {
        let source = fs::read_to_string(strong_dir.join(file)).unwrap();
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(&source);
        for (line_no, line) in production_source.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("*") {
                continue;
            }
            for token in forbidden {
                assert!(
                    !line.contains(token),
                    "compile/strong/{file}:{} exposes forbidden execution token {token}",
                    line_no + 1
                );
            }
        }
    }
}

fn phase3_admissibility_ctx() -> AdmissibilityContext {
    AdmissibilityContext {
        policy_class: PolicyClass::C,
        runtime_permitted: true,
        at_execution_boundary: true,
        require_evidence_closure: true,
    }
}

fn phase3_manifest() -> CapabilityManifest {
    CapabilityManifest {
        substrate_id: "phase3-exit-harness".into(),
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
