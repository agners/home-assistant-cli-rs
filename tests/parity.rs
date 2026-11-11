//! Go-vs-Rust parity checks, wrapped as (ignored by default) cargo tests:
//!
//!     cargo test --test parity -- --ignored
//!
//! They need `python3` and the Go reference binary (built automatically from
//! the Go source tree next to this repository when `go` is installed). See
//! the README for how to run the scripts directly for a single group.

use std::process::Command;

fn run(script: &str, args: &[&str]) {
    let status = Command::new("python3")
        .arg(format!("{}/tests/{script}", env!("CARGO_MANIFEST_DIR")))
        .args(args)
        .arg("--no-build")
        .env("HA_RUST_BIN", env!("CARGO_BIN_EXE_ha"))
        .status()
        .expect("python3 is required for the parity tests");
    assert!(status.success(), "{script} {args:?} reported differences");
}

#[test]
#[ignore = "needs python3 and the Go reference binary"]
fn parity_cases() {
    run("parity.py", &[]);
}

#[test]
#[ignore = "needs python3 and the Go reference binary"]
fn tree_diff_of_ported_groups() {
    for group in ["dns", "info", "completion"] {
        run("tree_diff.py", &[group]);
    }
}
