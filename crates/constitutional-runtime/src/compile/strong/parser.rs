//! Parser for JSON-form Strong Grammar.

use serde_json::Value;
use std::fmt;

use super::ast::{StrongProgram, StrongStep};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StrongParseError {
    JsonSyntax(String),
    ExpectedObject { path: String },
    MissingKind { path: String },
    UnknownConstruct { path: String, kind: String },
    Shape { path: String, detail: String },
}

impl fmt::Display for StrongParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrongParseError::JsonSyntax(detail) => write!(f, "json syntax: {detail}"),
            StrongParseError::ExpectedObject { path } => {
                write!(f, "expected object at {path}")
            }
            StrongParseError::MissingKind { path } => write!(f, "missing kind at {path}"),
            StrongParseError::UnknownConstruct { path, kind } => {
                write!(f, "unknown Strong construct {kind:?} at {path}")
            }
            StrongParseError::Shape { path, detail } => write!(f, "shape at {path}: {detail}"),
        }
    }
}

impl std::error::Error for StrongParseError {}

pub fn parse_strong_json(input: &str) -> Result<StrongProgram, StrongParseError> {
    let value: Value =
        serde_json::from_str(input).map_err(|e| StrongParseError::JsonSyntax(e.to_string()))?;
    parse_strong_value(value)
}

pub fn parse_strong_value(value: Value) -> Result<StrongProgram, StrongParseError> {
    preflight_constructs(&value, "$")?;
    let root: StrongStep = serde_json::from_value(value).map_err(|e| StrongParseError::Shape {
        path: "$".into(),
        detail: e.to_string(),
    })?;
    Ok(StrongProgram { root })
}

fn preflight_constructs(value: &Value, path: &str) -> Result<(), StrongParseError> {
    let object = value
        .as_object()
        .ok_or_else(|| StrongParseError::ExpectedObject { path: path.into() })?;
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| StrongParseError::MissingKind { path: path.into() })?;

    match kind {
        "Pipeline" => {
            let steps = object
                .get("steps")
                .and_then(Value::as_array)
                .ok_or_else(|| StrongParseError::Shape {
                    path: path.into(),
                    detail: "Pipeline.steps must be an array".into(),
                })?;
            for (i, step) in steps.iter().enumerate() {
                preflight_constructs(step, &format!("{path}.steps[{i}]"))?;
            }
        }
        "OnSuccess" | "OnFailure" => {
            let step = object.get("step").ok_or_else(|| StrongParseError::Shape {
                path: path.into(),
                detail: format!("{kind}.step is required"),
            })?;
            preflight_constructs(step, &format!("{path}.step"))?;
        }
        "SystemReview" => {
            if let Some(step) = object.get("on_success") {
                preflight_constructs(step, &format!("{path}.on_success"))?;
            }
        }
        "DriftReview" => {
            if let Some(step) = object.get("on_failure") {
                preflight_constructs(step, &format!("{path}.on_failure"))?;
            }
        }
        "Emit" | "Confirm" | "Execute" => {}
        other => {
            return Err(StrongParseError::UnknownConstruct {
                path: path.into(),
                kind: other.into(),
            });
        }
    }

    Ok(())
}
