//! High-level operations that correspond to CLI commands
//!
//! These modules contain the core business logic for each renamify operation,
//! separated from CLI concerns like argument parsing and output formatting.

pub mod apply;
pub mod history;
pub mod plan;
pub mod rename;
pub mod status;
pub mod undo;

// Re-export the main operation functions for easy access
pub use apply::apply_operation;
pub use history::history_operation;
pub use plan::plan_operation;
pub use rename::rename_operation;
pub use status::status_operation;
pub use undo::{redo_operation, undo_operation};
