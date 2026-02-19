/// Integration tests that run full Lyra programs end-to-end via both backends.

use std::process::Command;

fn lyra_bin() -> String {
    // Build path to cargo binary
    let output = Command::new("cargo")
        .args(["build", "--quiet"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo build failed");
    assert!(output.status.success(), "cargo build failed: {}", String::from_utf8_lossy(&output.stderr));

    // Path to the built binary
    format!("{}/target/debug/lyra", env!("CARGO_MANIFEST_DIR"))
}

fn run_lyra(file: &str, vm: bool) -> (String, String, bool) {
    let bin = lyra_bin();
    let mut cmd = Command::new(&bin);
    cmd.arg(file);
    if vm {
        cmd.arg("--vm");
    }
    let output = cmd.output().expect("failed to run lyra");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

fn example_path(name: &str) -> String {
    format!("{}/examples/{}", env!("CARGO_MANIFEST_DIR"), name)
}

// ── Showcase example ──

#[test]
fn showcase_tree_walker() {
    let (stdout, stderr, success) = run_lyra(&example_path("showcase.lyra"), false);
    assert!(success, "showcase.lyra failed (tree-walker):\n{}", stderr);
    assert!(stdout.contains("Hello, Lyra!"));
    assert!(stdout.contains("10! = 3628800"));
    assert!(stdout.contains("Sum of squares 1-10: 385"));
}

#[test]
fn showcase_vm() {
    let (stdout, stderr, success) = run_lyra(&example_path("showcase.lyra"), true);
    assert!(success, "showcase.lyra failed (VM):\n{}", stderr);
    assert!(stdout.contains("Hello, Lyra!"));
    assert!(stdout.contains("10! = 3628800"));
    assert!(stdout.contains("Sum of squares 1-10: 385"));
}

// ── Interpolation example ──

#[test]
fn interpolation_tree_walker() {
    let (stdout, stderr, success) = run_lyra(&example_path("interpolation.lyra"), false);
    assert!(success, "interpolation.lyra failed (tree-walker):\n{}", stderr);
    assert!(stdout.contains("Hello, World!"));
    assert!(stdout.contains("5! = 120"));
}

#[test]
fn interpolation_vm() {
    let (stdout, stderr, success) = run_lyra(&example_path("interpolation.lyra"), true);
    assert!(success, "interpolation.lyra failed (VM):\n{}", stderr);
    assert!(stdout.contains("Hello, World!"));
    assert!(stdout.contains("5! = 120"));
}

// ── Records example ──

#[test]
fn records_tree_walker() {
    let (stdout, stderr, success) = run_lyra(&example_path("records.lyra"), false);
    assert!(success, "records.lyra failed (tree-walker):\n{}", stderr);
    assert!(stdout.contains("Name: Alice"));
    assert!(stdout.contains("Bob is a Engineer"));
}

#[test]
fn records_vm() {
    let (stdout, stderr, success) = run_lyra(&example_path("records.lyra"), true);
    assert!(success, "records.lyra failed (VM):\n{}", stderr);
    assert!(stdout.contains("Name: Alice"));
    assert!(stdout.contains("Bob is a Engineer"));
}

// ── Pipes example ──

#[test]
fn pipes_tree_walker() {
    let path = example_path("pipes.lyra");
    if std::path::Path::new(&path).exists() {
        let (_, stderr, success) = run_lyra(&path, false);
        assert!(success, "pipes.lyra failed (tree-walker):\n{}", stderr);
    }
}

#[test]
fn pipes_vm() {
    let path = example_path("pipes.lyra");
    if std::path::Path::new(&path).exists() {
        let (_, stderr, success) = run_lyra(&path, true);
        assert!(success, "pipes.lyra failed (VM):\n{}", stderr);
    }
}

// ── Module import example ──

#[test]
fn modules_tree_walker() {
    let path = example_path("modules/main.lyra");
    if std::path::Path::new(&path).exists() {
        let (_, stderr, success) = run_lyra(&path, false);
        assert!(success, "modules/main.lyra failed (tree-walker):\n{}", stderr);
    }
}

#[test]
fn modules_vm() {
    let path = example_path("modules/main.lyra");
    if std::path::Path::new(&path).exists() {
        let (_, stderr, success) = run_lyra(&path, true);
        assert!(success, "modules/main.lyra failed (VM):\n{}", stderr);
    }
}

// ── VM benchmark example ──

#[test]
fn vm_bench_runs() {
    let path = example_path("vm_bench.lyra");
    if std::path::Path::new(&path).exists() {
        let (stdout, stderr, success) = run_lyra(&path, true);
        assert!(success, "vm_bench.lyra failed:\n{}", stderr);
        assert!(stdout.contains("fib(30)"));
    }
}

// ── Error cases ──

#[test]
fn type_error_shown() {
    // Create a temporary file with a type error
    let dir = std::env::temp_dir();
    let path = dir.join("lyra_test_type_error.lyra");
    std::fs::write(&path, "1 + \"hello\"").unwrap();
    let (_, stderr, success) = run_lyra(path.to_str().unwrap(), false);
    assert!(!success, "should have failed with a type error");
    assert!(!stderr.is_empty(), "stderr should contain error message");
    std::fs::remove_file(&path).ok();
}

#[test]
fn undefined_variable_suggests() {
    let dir = std::env::temp_dir();
    let path = dir.join("lyra_test_suggest.lyra");
    std::fs::write(&path, "to_strng(42)").unwrap();
    let (_, stderr, _) = run_lyra(path.to_str().unwrap(), false);
    assert!(stderr.contains("to_string") || stderr.contains("did you mean"),
        "should suggest 'to_string', got: {}", stderr);
    std::fs::remove_file(&path).ok();
}

// ── Both backends agree on output ──

#[test]
fn both_backends_agree_on_showcase() {
    let (stdout_tw, _, success_tw) = run_lyra(&example_path("showcase.lyra"), false);
    let (stdout_vm, _, success_vm) = run_lyra(&example_path("showcase.lyra"), true);
    assert!(success_tw && success_vm, "both backends should succeed");
    assert_eq!(stdout_tw, stdout_vm, "tree-walker and VM should produce identical output");
}

#[test]
fn both_backends_agree_on_interpolation() {
    let (stdout_tw, _, success_tw) = run_lyra(&example_path("interpolation.lyra"), false);
    let (stdout_vm, _, success_vm) = run_lyra(&example_path("interpolation.lyra"), true);
    assert!(success_tw && success_vm, "both backends should succeed");
    assert_eq!(stdout_tw, stdout_vm, "tree-walker and VM should produce identical output");
}

#[test]
fn both_backends_agree_on_records() {
    let (stdout_tw, _, success_tw) = run_lyra(&example_path("records.lyra"), false);
    let (stdout_vm, _, success_vm) = run_lyra(&example_path("records.lyra"), true);
    assert!(success_tw && success_vm, "both backends should succeed");
    assert_eq!(stdout_tw, stdout_vm, "tree-walker and VM should produce identical output");
}
