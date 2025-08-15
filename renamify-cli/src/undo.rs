use anyhow::Result;
use renamify_core::undo_operation;

pub fn handle_undo(id: &str) -> Result<()> {
    let result = undo_operation(id, None)?;
    println!("{}", result);
    Ok(())
}
