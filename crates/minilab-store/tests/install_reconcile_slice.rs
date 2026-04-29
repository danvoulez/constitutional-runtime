//! Integration tests for the constitutional `install.reconcile` vertical.
//!
//! This is the first Reconcile-shaped slice: planned drift, zero or more
//! applied steps, then reconciled or failed closure under one correlation.

use minilab_core::evidence::EvidenceKind;
use minilab_core::SimMode;
use minilab_store::{
    submit_install_reconcile, InstallReconcileInput, InstallReconcileOutcome, StoreClient,
};
use serde_json::{json, Value};
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Seed {
    installation_id: Uuid,
    host_id: Uuid,
    desired_manifest: Value,
    observed_manifest: Value,
    prior_ledger_rows: Vec<Value>,
}

impl Seed {
    fn fresh(desired_manifest: Value, observed_manifest: Value) -> Self {
        Self {
            installation_id: Uuid::new_v4(),
            host_id: Uuid::new_v4(),
            desired_manifest,
            observed_manifest,
            prior_ledger_rows: vec![],
        }
    }
}

async fn mount_seed(server: &MockServer, seed: &Seed) {
    Mock::given(method("GET"))
        .and(path("/rest/v1/installation"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "host_id": seed.host_id,
            "desired_manifest": seed.desired_manifest,
            "observed_manifest": seed.observed_manifest,
            "canon_version": "canon-v1",
            "elastic_version": "elastic-v1",
        }])))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path("/rest/v1/evidence_ledger"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&seed.prior_ledger_rows))
        .mount(server)
        .await;

    Mock::given(method("POST"))
        .and(path("/rest/v1/evidence_ledger"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({})))
        .mount(server)
        .await;
}

fn client_for(server: &MockServer) -> StoreClient {
    StoreClient::with_mode(server.uri(), "service-key", SimMode::Simulation)
}

async fn evidence_posts(server: &MockServer) -> Vec<Value> {
    let received = server.received_requests().await.unwrap_or_default();
    received
        .into_iter()
        .filter(|r| {
            r.method == wiremock::http::Method::POST && r.url.path() == "/rest/v1/evidence_ledger"
        })
        .filter_map(|r| serde_json::from_slice::<Value>(&r.body).ok())
        .collect()
}

fn kinds_in_order(rows: &[Value]) -> Vec<String> {
    rows.iter()
        .filter_map(|r| r["kind"].as_str().map(str::to_owned))
        .collect()
}

fn input(installation_id: Uuid, correlation_id: Uuid) -> InstallReconcileInput {
    InstallReconcileInput {
        installation_id,
        correlation_id,
    }
}

#[tokio::test]
async fn happy_path_plans_applies_and_reconciles() {
    let server = MockServer::start().await;
    let seed = Seed::fresh(
        json!({ "services": { "api": "v2", "worker": "v1" } }),
        json!({ "services": { "api": "v1", "worker": "v1" } }),
    );
    mount_seed(&server, &seed).await;

    let correlation_id = Uuid::new_v4();
    let outcome = submit_install_reconcile(
        &client_for(&server),
        input(seed.installation_id, correlation_id),
    )
    .await
    .expect("reconcile happy path must not error");

    match outcome {
        InstallReconcileOutcome::Reconciled {
            applied_steps,
            idempotent,
            ..
        } => {
            assert_eq!(applied_steps, 1);
            assert!(!idempotent);
        }
        other => panic!("expected Reconciled, got {other:?}"),
    }

    let rows = evidence_posts(&server).await;
    assert_eq!(
        kinds_in_order(&rows),
        vec![
            EvidenceKind::INSTALL_RECONCILE_PLANNED.to_string(),
            EvidenceKind::INSTALL_RECONCILE_STEP_APPLIED.to_string(),
            EvidenceKind::INSTALL_RECONCILE_RECONCILED.to_string(),
        ]
    );

    for row in &rows {
        assert_eq!(
            row["correlation_id"].as_str(),
            Some(correlation_id.to_string().as_str())
        );
        assert_eq!(row["sim_mode"].as_str(), Some("simulation"));
    }

    let step = rows
        .iter()
        .find(|row| row["kind"] == EvidenceKind::INSTALL_RECONCILE_STEP_APPLIED)
        .unwrap();
    assert_eq!(step["payload"]["service"].as_str(), Some("api"));
    assert_eq!(step["payload"]["from_version"].as_str(), Some("v1"));
    assert_eq!(step["payload"]["to_version"].as_str(), Some("v2"));
}

#[tokio::test]
async fn sub_step_failure_closes_failed_without_applied_steps() {
    let server = MockServer::start().await;
    let seed = Seed::fresh(
        json!({ "services": { "api": { "version": "v2", "simulate": "fail" } } }),
        json!({ "services": { "api": "v1" } }),
    );
    mount_seed(&server, &seed).await;

    let outcome = submit_install_reconcile(
        &client_for(&server),
        input(seed.installation_id, Uuid::new_v4()),
    )
    .await
    .expect("step failure is a closed outcome, not a bubbled error");

    match outcome {
        InstallReconcileOutcome::Failed {
            reason_code,
            phase,
            applied_steps,
            remaining_steps,
            partial_convergence,
            ..
        } => {
            assert_eq!(reason_code, "step_apply_failed");
            assert_eq!(phase, "execution");
            assert_eq!(applied_steps, 0);
            assert_eq!(remaining_steps, 1);
            assert!(!partial_convergence);
        }
        other => panic!("expected Failed, got {other:?}"),
    }

    assert_eq!(
        kinds_in_order(&evidence_posts(&server).await),
        vec![
            EvidenceKind::INSTALL_RECONCILE_PLANNED.to_string(),
            EvidenceKind::INSTALL_RECONCILE_FAILED.to_string(),
        ]
    );
}

#[tokio::test]
async fn idempotent_rerun_plans_then_closes_without_reapplying_steps() {
    let server = MockServer::start().await;
    let mut seed = Seed::fresh(
        json!({ "services": { "api": "v2" } }),
        json!({ "services": { "api": "v1" } }),
    );
    seed.prior_ledger_rows = vec![json!({
        "payload": {
            "installation_id": seed.installation_id,
            "desired_hash": desired_hash(&seed.desired_manifest),
        }
    })];
    mount_seed(&server, &seed).await;

    let outcome = submit_install_reconcile(
        &client_for(&server),
        input(seed.installation_id, Uuid::new_v4()),
    )
    .await
    .expect("idempotent rerun must close");

    match outcome {
        InstallReconcileOutcome::Reconciled {
            applied_steps,
            idempotent,
            ..
        } => {
            assert_eq!(applied_steps, 0);
            assert!(idempotent);
        }
        other => panic!("expected Reconciled(idempotent), got {other:?}"),
    }

    assert_eq!(
        kinds_in_order(&evidence_posts(&server).await),
        vec![
            EvidenceKind::INSTALL_RECONCILE_PLANNED.to_string(),
            EvidenceKind::INSTALL_RECONCILE_RECONCILED.to_string(),
        ]
    );
}

#[tokio::test]
async fn partial_convergence_is_explicit_when_later_step_fails() {
    let server = MockServer::start().await;
    let seed = Seed::fresh(
        json!({
            "services": {
                "api": "v2",
                "worker": { "version": "v3", "simulate": "fail" }
            }
        }),
        json!({ "services": { "api": "v1", "worker": "v1" } }),
    );
    mount_seed(&server, &seed).await;

    let outcome = submit_install_reconcile(
        &client_for(&server),
        input(seed.installation_id, Uuid::new_v4()),
    )
    .await
    .expect("partial convergence failure must be closed");

    match outcome {
        InstallReconcileOutcome::Failed {
            applied_steps,
            remaining_steps,
            partial_convergence,
            ..
        } => {
            assert_eq!(applied_steps, 1);
            assert_eq!(remaining_steps, 1);
            assert!(partial_convergence);
        }
        other => panic!("expected Failed(partial), got {other:?}"),
    }

    let rows = evidence_posts(&server).await;
    assert_eq!(
        kinds_in_order(&rows),
        vec![
            EvidenceKind::INSTALL_RECONCILE_PLANNED.to_string(),
            EvidenceKind::INSTALL_RECONCILE_STEP_APPLIED.to_string(),
            EvidenceKind::INSTALL_RECONCILE_FAILED.to_string(),
        ]
    );
    let failed = rows
        .iter()
        .find(|row| row["kind"] == EvidenceKind::INSTALL_RECONCILE_FAILED)
        .unwrap();
    assert_eq!(
        failed["payload"]["partial_convergence"].as_bool(),
        Some(true)
    );
    assert_eq!(failed["payload"]["phase"].as_str(), Some("execution"));
}

fn desired_hash(value: &Value) -> String {
    use sha2::{Digest, Sha256};

    Sha256::digest(value.to_string().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
