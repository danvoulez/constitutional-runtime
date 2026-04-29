//! Strong Grammar compiler surface.
//!
//! Strong Grammar is a language boundary, not an execution boundary:
//!
//! ```text
//! Strong JSON -> Strong AST -> IR graph
//! ```
//!
//! This module intentionally stops at [`IrGraph`]. Validation, planning,
//! lowering, dispatch, and evidence closure remain owned by the existing
//! runtime pipeline.

pub mod ast;
pub mod compiler;
pub mod parser;

pub use ast::{StrongProgram, StrongStep};
pub use compiler::{compile_strong_program_to_ir_graph, StrongCompileError};
pub use parser::{parse_strong_json, StrongParseError};
