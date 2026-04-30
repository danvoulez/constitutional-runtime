use std::sync::Arc;

use axum::{body::Body, http::Request};
use http_body_util::BodyExt as _;
use minilab_api::{
    build_app, AgentRuntimeIngressFinalState, AgentRuntimeIngressReport, AgentRuntimeService,
    ApiConfig, AppState, CandidateKind,
};
use minilab_store::StoreClient;
use serde_json::json;
use tower::ServiceExt;

fn test_state() -> AppState {
    AppState {
        store: StoreClient::new("http://127.0.0.1:9", "service-key"),
        agent_runtime: AgentRuntimeService::new().expect("agent runtime service"),
        config: Arc::new(ApiConfig {
            bind_addr: "127.0.0.1:3000".parse().unwrap(),
            public_base_url: Some("https://api.minilab.example".into()),
            request_timeout: std::time::Duration::from_secs(5),
            twilio_max_body_bytes: 262_144,
            sendgrid_max_body_bytes: 1024 * 1024,
            twilio_auth_token: None,
            sendgrid_parse_public_key: None,
        }),
    }
}

async fn post_message(text: &str) -> AgentRuntimeIngressReport {
    let app = build_app(test_state());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/agent-runtime/places/chatgpt_workspace/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "text": text,
                        "app_id": "phase4_exit_harness"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request should complete");

    assert!(response.status().is_success());
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("report should deserialize")
}

#[tokio::test]
async fn phase4_exit_empty_message_is_received_and_rejected_without_execution() {
    let report = post_message(" ").await;

    assert_eq!(report.final_state, AgentRuntimeIngressFinalState::Rejected);
    assert_eq!(report.ir_status, "not_produced");
    assert_eq!(report.dispatch_boundary_status, "stopped_before_dispatch");
    assert!(report.ghosts.iter().any(|ghost| ghost == "missing_intent"));
}

#[tokio::test]
async fn phase4_exit_operational_candidate_reaches_ir_readiness_only() {
    let report = post_message("reconcile installation 8d4df830-4d1f-4a9f-bc17-7de11580d8f3").await;

    assert_eq!(
        report.final_state,
        AgentRuntimeIngressFinalState::ReadyForIr
    );
    assert_eq!(report.ir_status, "ready_for_ir");
    assert_eq!(report.dispatch_boundary_status, "stopped_before_dispatch");
    assert_eq!(report.next_required_action, "submit_candidate_to_ir_gate");
    assert_eq!(
        report.candidate_classification.candidate_kind,
        CandidateKind::OperationalCandidate
    );
}

#[tokio::test]
async fn phase4_exit_strong_candidate_reaches_ir_readiness_only() {
    let report = post_message(r#"{ "kind": "Emit", "event": "demo.started" }"#).await;

    assert_eq!(
        report.final_state,
        AgentRuntimeIngressFinalState::ReadyForIr
    );
    assert_eq!(report.ir_status, "ready_for_ir");
    assert_eq!(report.dispatch_boundary_status, "stopped_before_dispatch");
    assert_eq!(report.next_required_action, "submit_candidate_to_ir_gate");
}

#[tokio::test]
async fn phase4_exit_reconstructs_report_by_message_id() {
    let app = build_app(test_state());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/agent-runtime/places/chatgpt_workspace/messages")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "text": "reconcile installation 8d4df830-4d1f-4a9f-bc17-7de11580d8f3"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .expect("request should complete");

    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes();
    let report: AgentRuntimeIngressReport =
        serde_json::from_slice(&bytes).expect("report should deserialize");

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/agent-runtime/reports/{}", report.message_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request should complete");
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body should collect")
        .to_bytes();
    let reconstructed: AgentRuntimeIngressReport =
        serde_json::from_slice(&bytes).expect("report should deserialize");

    assert_eq!(reconstructed.message_id, report.message_id);
    assert_eq!(reconstructed.correlation_id, report.correlation_id);
    assert_eq!(
        reconstructed.dispatch_boundary_status,
        "stopped_before_dispatch"
    );
}
