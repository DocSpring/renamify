use anyhow::Result;
use renamify_core::redo_operation;

pub fn handle_redo(id: &str) -> Result<()> {
    let result = redo_operation(id, None)?;
    println!("{}", result);
    Ok(())
}
