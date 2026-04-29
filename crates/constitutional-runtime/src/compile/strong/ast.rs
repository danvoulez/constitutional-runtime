//! AST for JSON-form Strong Grammar.
//!
//! The v0 surface is deliberately JSON-only. Textual syntax is a later
//! surface and must compile through this same AST before reaching IR.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Root Strong Grammar program.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StrongProgram {
    pub root: StrongStep,
}

/// JSON-form Strong Grammar constructs admitted in Phase 3.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "PascalCase")]
pub enum StrongStep {
    Pipeline {
        steps: Vec<StrongStep>,
    },
    OnSuccess {
        step: Box<StrongStep>,
    },
    OnFailure {
        step: Box<StrongStep>,
    },
    Emit {
        event: String,
    },
    Confirm {
        subject: String,
        #[serde(default = "default_confirm_role")]
        role: String,
    },
    Execute {
        action: String,
        #[serde(default)]
        params: Map<String, Value>,
    },
    SystemReview {
        target: String,
        pipeline: Vec<String>,
        #[serde(default)]
        on_success: Option<Box<StrongStep>>,
    },
    DriftReview {
        target: String,
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        on_failure: Option<Box<StrongStep>>,
    },
}

fn default_confirm_role() -> String {
    "operator".into()
}
