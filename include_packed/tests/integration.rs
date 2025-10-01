// in tests/integration.rs

use std::process::Command;
use which::which;

#[test]
fn run_test_project() {
    // 1. Locate the `cargo` binary on the system's PATH.
    let cargo = which("cargo").expect("cargo not found in PATH");

    // 2. Execute `cargo run` within the `test-project` directory.
    // This command triggers the test project's build script and then runs its main binary.
    let output = Command::new(cargo)
        .arg("run")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_project"))
        .output()
        .expect("Failed to execute test project");

    // 3. Assert that the command executed successfully.
    // If it failed, print the stdout and stderr for easy debugging.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Test project failed to run:\n--- stdout\n{}\n--- stderr\n{}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );

    // 4. Assert that the program's output contains the expected text.
    // This confirms that the file was correctly included, decompressed, and printed.
    let stdout = String::from_utf8(output.stdout).expect("non UTF-8 output from test project");
    assert!(
        stdout.contains("Contents of file.txt"),
        "stdout did not contain expected content: {stdout}"
    );
    assert!(
        stdout.contains("Decompressed data matches original."),
        "stdout did not contain success message: {stdout}"
    );
}
