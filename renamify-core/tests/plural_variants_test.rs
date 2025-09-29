use renamify_core::{scan_repository_multi, PlanOptions};
use std::fs;
use tempfile::TempDir;

fn write_sample_files(root: &std::path::Path) {
    let ts_path = root.join("web/src/api/schemas/getAdminDeployRequestsParams.ts");
    fs::create_dir_all(ts_path.parent().unwrap()).unwrap();
    fs::write(
        &ts_path,
        r#"import type { DeployRequest, DeployRequestList } from './types';

export const listAdminDeployRequests = (
  params: GetAdminDeployRequestsParams,
): Promise<DeployRequestList> =>
  unwrap(gateway.getAdminDeployRequests(params));

export const approveAdminDeployRequest = (
  id: string,
  payload?: DeployRequest,
): Promise<DeployRequest> =>
  unwrap(gateway.postAdminDeployRequestsIdApprove(id, payload ?? {}));
"#,
    )
    .unwrap();

    let go_path = root.join("internal/gateway/db/deploy_requests.go");
    fs::create_dir_all(go_path.parent().unwrap()).unwrap();
    fs::write(&go_path, "package db\n\n// deploy_requests placeholder\n").unwrap();
}

#[test]
fn test_plural_variants_enabled_updates_singular_forms() {
    let temp_dir = TempDir::new().unwrap();
    write_sample_files(temp_dir.path());

    let plan_out = temp_dir.path().join(".renamify/plan.json");
    let options = PlanOptions {
        plan_out: plan_out.clone(),
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "DeployRequests",
        "DeployApprovalRequests",
        &options,
    )
    .unwrap();
    let rename = plan
        .paths
        .iter()
        .find(|r| {
            r.path
                .ends_with("web/src/api/schemas/getAdminDeployRequestsParams.ts")
        })
        .expect("expected params file to be scheduled for rename");

    assert!(
        rename
            .new_path
            .ends_with("web/src/api/schemas/getAdminDeployApprovalRequestsParams.ts"),
        "file rename should preserve camel case with capital D",
    );

    assert!(
        plan.matches.iter().any(|m| {
            m.line_after
                .as_deref()
                .is_some_and(|line| line.contains("DeployApprovalRequestList"))
        }),
        "should rewrite DeployRequestList line when plural support enabled",
    );
    assert!(
        plan.matches
            .iter()
            .any(|m| m.content == "DeployRequest" && m.replace == "DeployApprovalRequest"),
        "should rewrite DeployRequest singular form when plural support enabled",
    );
}

#[test]
fn test_plural_variants_can_be_disabled() {
    let temp_dir = TempDir::new().unwrap();
    write_sample_files(temp_dir.path());

    let plan_out = temp_dir.path().join(".renamify/plan.json");
    let options = PlanOptions {
        plan_out,
        enable_plural_variants: false,
        ..Default::default()
    };

    let plan = scan_repository_multi(
        &[temp_dir.path().to_path_buf()],
        "DeployRequests",
        "DeployApprovalRequests",
        &options,
    )
    .unwrap();
    assert!(plan.paths.iter().any(|r| r
        .path
        .ends_with("web/src/api/schemas/getAdminDeployRequestsParams.ts")));

    assert!(
        !plan
            .matches
            .iter()
            .any(|m| m.content == "DeployRequestList"),
        "DeployRequestList should remain untouched when plural variants disabled",
    );
    assert!(
        !plan.matches.iter().any(|m| m.content == "DeployRequest"),
        "DeployRequest singular forms should remain untouched when plural variants disabled",
    );
}
