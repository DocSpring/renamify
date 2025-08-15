use anyhow::Result;
use renamify_core::apply_operation;

pub fn handle_apply(plan_id: Option<String>, commit: bool, force: bool) -> Result<()> {
    let result = apply_operation(None, plan_id, commit, force, None)?;
    println!("{result}");
    Ok(())
}
