#[test]
#[cfg(not(target_os = "windows"))]
fn export_bindings() {
    // This test exports TypeScript bindings when run
    // The bindings will be generated in the bindings/ directory

    // After ts-rs generates the bindings, convert them to ambient declarations
    let output = std::process::Command::new("./convert-ts-bindings-to-ambient.sh")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run convert-ts-bindings-to-ambient.sh");

    if !output.status.success() {
        panic!(
            "convert-ts-bindings-to-ambient.sh failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
