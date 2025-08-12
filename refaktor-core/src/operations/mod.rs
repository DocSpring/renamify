//! High-level operations that correspond to CLI commands
//! 
//! These modules contain the core business logic for each refaktor operation,
//! separated from CLI concerns like argument parsing and output formatting.

pub mod apply;
pub mod plan;
pub mod rename;
pub mod undo;

// Re-export the main operation functions for easy access
pub use apply::apply_operation;
pub use plan::{plan_operation, plan_operation_with_dry_run};
pub use rename::rename_operation;
pub use undo::{undo_operation, redo_operation};