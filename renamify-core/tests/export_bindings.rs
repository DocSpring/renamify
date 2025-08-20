#[test]
fn export_bindings() {
    // This test exports TypeScript bindings when run
    // The bindings will be generated in the bindings/ directory

    // After ts-rs generates the bindings, convert them to ambient declarations
    let output = std::process::Command::new("node")
        .arg("convert-ts-bindings-to-ambient.js")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run convert-ts-bindings-to-ambient.js");

    if !output.status.success() {
        panic!(
            "convert-ts-bindings-to-ambient.js failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
