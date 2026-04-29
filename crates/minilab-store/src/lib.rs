pub mod dispatch;
pub mod evidence;
pub mod legacy;
pub mod persistence;
pub mod slices;

pub mod campaign {
    pub use crate::legacy::campaign::*;
}
pub mod client {
    pub use crate::persistence::client::*;
}
pub mod dispatcher {
    pub use crate::dispatch::*;
}
pub mod eligibility {
    pub use crate::slices::outbound_send::eligibility::*;
}
pub mod host_pair {
    pub use crate::slices::host_pair::*;
}
pub mod install_reconcile {
    pub use crate::slices::install_reconcile::*;
}
pub mod optout_gate {
    pub use crate::slices::outbound_send::optout_gate::*;
}
pub mod outbound {
    pub use crate::legacy::outbound::*;
}
pub mod outbound_orchestrator {
    pub use crate::slices::outbound_send::orchestrator::*;
}
pub mod outreach {
    pub use crate::legacy::outreach::*;
}
pub mod policy {
    pub use crate::slices::outbound_send::policy::*;
}
pub mod premium {
    pub use crate::slices::outbound_send::premium::*;
}
pub mod real_dispatcher {
    pub use crate::dispatch::real::*;
}
pub mod reply {
    pub use crate::legacy::reply::*;
}
pub mod scoring {
    pub use crate::legacy::scoring::*;
}
pub mod store {
    pub use crate::persistence::store::*;
}
pub mod webhook {
    pub use crate::persistence::webhook::*;
}

pub use client::{StoreClient, StoreError};
pub use dispatcher::{
    dispatch_operational_command, lower_and_dispatch_execute, lower_execute_action, DispatchOutcome,
};
pub use host_pair::{submit_host_pair, HostPairInput, HostPairOutcome};
pub use install_reconcile::{
    submit_install_reconcile, InstallReconcileInput, InstallReconcileOutcome,
};
pub use outbound_orchestrator::{submit_outbound_send, OutboundSendInput, OutboundSendOutcome};
pub use store::{ingest_signal, IngestInput, IngestOutput};
