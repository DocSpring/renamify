use anyhow::Result;
use refaktor_core::apply_operation;
use std::path::PathBuf;

pub fn handle_apply(
    plan_path: Option<PathBuf>,
    plan_id: Option<String>,
    commit: bool,
    force: bool,
) -> Result<()> {
    let result = apply_operation(plan_path, plan_id, commit, force, None)?;
    println!("{}", result);
    Ok(())
}
