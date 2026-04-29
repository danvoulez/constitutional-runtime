//! Constitutional `install.reconcile` orchestrator.
//!
//! First Reconcile-shaped slice. Unlike `outbound.send` and `host.pair`, this
//! slice is not a single admissibility-governed act followed by one execution
//! boundary. It converges desired state against observed state:
//!
//! ```text
//!   install.reconcile.planned
//!     -> install.reconcile.step.applied*
//!     -> install.reconcile.reconciled | install.reconcile.failed
//! ```
//!
//! The substrate is deliberately narrow. `installation.desired_manifest` and
//! `installation.observed_manifest` are JSON objects with a top-level
//! `services` object. Each service maps to either a version string or an
//! object with a `version` string and optional `"simulate": "fail"` for
//! deterministic failure tests.

use minilab_core::evidence::EvidenceKind;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::client::{StoreClient, StoreError};
use crate::evidence::insert_ledger_row;

#[derive(Debug, Clone)]
pub struct InstallReconcileInput {
    pub installation_id: Uuid,
    pub correlation_id: Uuid,
}

#[derive(Debug, Clone)]
pub enum InstallReconcileOutcome {
    Reconciled {
        desired_hash: String,
        applied_steps: usize,
        idempotent: bool,
    },
    Failed {
        desired_hash: Option<String>,
        reason_code: String,
        reason_detail: String,
        phase: &'static str,
        applied_steps: usize,
        remaining_steps: usize,
        partial_convergence: bool,
    },
}

impl InstallReconcileOutcome {
    pub fn was_reconciled(&self) -> bool {
        matches!(self, Self::Reconciled { .. })
    }
}

pub async fn submit_install_reconcile(
    client: &StoreClient,
    input: InstallReconcileInput,
) -> Result<InstallReconcileOutcome, StoreError> {
    let installation = match fetch_installation(client, input.installation_id).await? {
        Some(row) => row,
        None => {
            return fail(
                client,
                &input,
                None,
                "install_not_found",
                &format!("installation {} has no row", input.installation_id),
                "pre_admission",
                0,
                0,
            )
            .await;
        }
    };

    let desired_hash = manifest_hash(&installation.desired_manifest);
    let steps = match diff_steps(
        &installation.desired_manifest,
        &installation.observed_manifest,
    ) {
        Ok(steps) => steps,
        Err(detail) => {
            return fail(
                client,
                &input,
                Some(desired_hash),
                "desired_manifest_invalid",
                &detail,
                "planning",
                0,
                0,
            )
            .await;
        }
    };

    record_planned(client, &input, &installation, &desired_hash, &steps).await?;

    if prior_reconciled_exists(client, input.installation_id, &desired_hash).await? {
        return record_reconciled(client, &input, &installation, &desired_hash, 0, true).await;
    }

    if steps.is_empty() {
        return record_reconciled(client, &input, &installation, &desired_hash, 0, false).await;
    }

    let mut applied_steps = 0usize;
    for (idx, step) in steps.iter().enumerate() {
        if step.simulate_failure {
            return fail(
                client,
                &input,
                Some(desired_hash),
                "step_apply_failed",
                &format!(
                    "service {} refused convergence from {:?} to {}",
                    step.service, step.observed_version, step.desired_version
                ),
                "execution",
                applied_steps,
                steps.len() - idx,
            )
            .await;
        }

        insert_ledger_row(
            client,
            EvidenceKind::INSTALL_RECONCILE_STEP_APPLIED,
            json!({
                "installation_id": input.installation_id,
                "host_id": installation.host_id,
                "service": step.service,
                "from_version": step.observed_version,
                "to_version": step.desired_version,
                "desired_hash": desired_hash,
                "step_index": idx,
                "correlation_id": input.correlation_id,
            }),
            input.correlation_id,
            Some(format!(
                "install.reconcile.step.applied:{}:{}:{}",
                input.installation_id, desired_hash, step.service
            )),
        )
        .await?;
        applied_steps += 1;
    }

    record_reconciled(
        client,
        &input,
        &installation,
        &desired_hash,
        applied_steps,
        false,
    )
    .await
}

#[derive(Debug, Clone)]
struct InstallationRow {
    host_id: Option<Uuid>,
    desired_manifest: Value,
    observed_manifest: Value,
    canon_version: String,
    elastic_version: String,
}

#[derive(Debug, Clone)]
struct ReconcileStep {
    service: String,
    observed_version: Option<String>,
    desired_version: String,
    simulate_failure: bool,
}

async fn fetch_installation(
    client: &StoreClient,
    installation_id: Uuid,
) -> Result<Option<InstallationRow>, StoreError> {
    let rows: Vec<Value> = client
        .http
        .get(format!(
            "{}?id=eq.{}&select=host_id,desired_manifest,observed_manifest,canon_version,elastic_version&limit=1",
            client.rest("installation"),
            installation_id
        ))
        .send()
        .await?
        .json::<Vec<Value>>()
        .await?;

    Ok(rows.into_iter().next().map(|row| InstallationRow {
        host_id: row["host_id"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok()),
        desired_manifest: row["desired_manifest"].clone(),
        observed_manifest: row["observed_manifest"].clone(),
        canon_version: row["canon_version"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
        elastic_version: row["elastic_version"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
    }))
}

fn diff_steps(desired: &Value, observed: &Value) -> Result<Vec<ReconcileStep>, String> {
    let desired_services = desired
        .get("services")
        .and_then(Value::as_object)
        .ok_or_else(|| "desired_manifest.services must be an object".to_string())?;
    if desired_services.is_empty() {
        return Err("desired_manifest.services must not be empty".into());
    }
    let observed_services = observed.get("services").and_then(Value::as_object);

    let mut steps = Vec::new();
    for (service, desired_value) in desired_services {
        let (desired_version, simulate_failure) = desired_service_version(desired_value)
            .ok_or_else(|| format!("desired service `{service}` must name a version"))?;
        let observed_version = observed_services
            .and_then(|services| services.get(service))
            .and_then(observed_service_version);
        if observed_version.as_deref() != Some(desired_version.as_str()) {
            steps.push(ReconcileStep {
                service: service.clone(),
                observed_version,
                desired_version,
                simulate_failure,
            });
        }
    }

    Ok(steps)
}

fn desired_service_version(value: &Value) -> Option<(String, bool)> {
    if let Some(version) = value.as_str() {
        return Some((version.to_owned(), false));
    }
    let obj = value.as_object()?;
    let version = obj.get("version")?.as_str()?.to_owned();
    let simulate_failure = obj.get("simulate").and_then(Value::as_str) == Some("fail");
    Some((version, simulate_failure))
}

fn observed_service_version(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned).or_else(|| {
        value
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_owned)
    })
}

fn manifest_hash(value: &Value) -> String {
    let digest = Sha256::digest(value.to_string().as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

async fn prior_reconciled_exists(
    client: &StoreClient,
    installation_id: Uuid,
    desired_hash: &str,
) -> Result<bool, StoreError> {
    let rows: Vec<Value> = client
        .http
        .get(format!(
            "{}?kind=eq.{}&select=payload&limit=50",
            client.rest("evidence_ledger"),
            EvidenceKind::INSTALL_RECONCILE_RECONCILED
        ))
        .send()
        .await?
        .json::<Vec<Value>>()
        .await?;

    Ok(rows.into_iter().any(|row| {
        let payload = &row["payload"];
        payload["installation_id"].as_str() == Some(installation_id.to_string().as_str())
            && payload["desired_hash"].as_str() == Some(desired_hash)
    }))
}

async fn record_planned(
    client: &StoreClient,
    input: &InstallReconcileInput,
    installation: &InstallationRow,
    desired_hash: &str,
    steps: &[ReconcileStep],
) -> Result<(), StoreError> {
    let planned_steps: Vec<Value> = steps
        .iter()
        .enumerate()
        .map(|(idx, step)| {
            json!({
                "step_index": idx,
                "service": step.service,
                "from_version": step.observed_version,
                "to_version": step.desired_version,
            })
        })
        .collect();

    insert_ledger_row(
        client,
        EvidenceKind::INSTALL_RECONCILE_PLANNED,
        json!({
            "installation_id": input.installation_id,
            "host_id": installation.host_id,
            "desired_hash": desired_hash,
            "planned_steps": planned_steps,
            "step_count": steps.len(),
            "canon_version": installation.canon_version,
            "elastic_version": installation.elastic_version,
            "correlation_id": input.correlation_id,
        }),
        input.correlation_id,
        Some(format!(
            "install.reconcile.planned:{}:{}:{}",
            input.installation_id, desired_hash, input.correlation_id
        )),
    )
    .await
}

async fn record_reconciled(
    client: &StoreClient,
    input: &InstallReconcileInput,
    installation: &InstallationRow,
    desired_hash: &str,
    applied_steps: usize,
    idempotent: bool,
) -> Result<InstallReconcileOutcome, StoreError> {
    insert_ledger_row(
        client,
        EvidenceKind::INSTALL_RECONCILE_RECONCILED,
        json!({
            "installation_id": input.installation_id,
            "host_id": installation.host_id,
            "desired_hash": desired_hash,
            "applied_steps": applied_steps,
            "idempotent": idempotent,
            "canon_version": installation.canon_version,
            "elastic_version": installation.elastic_version,
            "correlation_id": input.correlation_id,
        }),
        input.correlation_id,
        Some(format!(
            "install.reconcile.reconciled:{}:{}:{}",
            input.installation_id, desired_hash, input.correlation_id
        )),
    )
    .await?;

    Ok(InstallReconcileOutcome::Reconciled {
        desired_hash: desired_hash.to_owned(),
        applied_steps,
        idempotent,
    })
}

#[allow(clippy::too_many_arguments)]
async fn fail(
    client: &StoreClient,
    input: &InstallReconcileInput,
    desired_hash: Option<String>,
    reason_code: &str,
    reason_detail: &str,
    phase: &'static str,
    applied_steps: usize,
    remaining_steps: usize,
) -> Result<InstallReconcileOutcome, StoreError> {
    let partial_convergence = applied_steps > 0 && remaining_steps > 0;
    insert_ledger_row(
        client,
        EvidenceKind::INSTALL_RECONCILE_FAILED,
        json!({
            "installation_id": input.installation_id,
            "desired_hash": desired_hash,
            "reason_code": reason_code,
            "reason_detail": reason_detail,
            "phase": phase,
            "applied_steps": applied_steps,
            "remaining_steps": remaining_steps,
            "partial_convergence": partial_convergence,
            "correlation_id": input.correlation_id,
        }),
        input.correlation_id,
        Some(format!(
            "install.reconcile.failed:{}:{}:{}",
            input.installation_id, reason_code, input.correlation_id
        )),
    )
    .await?;

    Ok(InstallReconcileOutcome::Failed {
        desired_hash,
        reason_code: reason_code.to_owned(),
        reason_detail: reason_detail.to_owned(),
        phase,
        applied_steps,
        remaining_steps,
        partial_convergence,
    })
}
