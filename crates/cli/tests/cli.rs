use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_symbiont"))
        .args(args)
        .output()
        .unwrap()
}

fn example(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
        .display()
        .to_string()
}

#[test]
fn preview_handles_met_not_met_and_indeterminate() {
    for (file, expected) in [
        ("market-yes.json", "met"),
        ("market-no.json", "not_met"),
        ("market-unresolved.json", "indeterminate"),
    ] {
        let out = cli(&["evaluate", &example(file)]);
        assert!(out.status.success());
        assert_eq!(
            serde_json::from_slice::<Value>(&out.stdout).unwrap()["outcome"],
            expected
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains("Preview only"));
    }
}

#[test]
fn expectation_is_saved_without_overwriting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("expected.json");
    let output = path.to_str().unwrap();
    assert!(
        cli(&["prepare", &example("participant-survey.json"), output])
            .status
            .success()
    );
    let before = fs::read(&path).unwrap();
    assert!(!cli(&["prepare", &example("market-yes.json"), output])
        .status
        .success());
    assert_eq!(before, fs::read(&path).unwrap());
    let value: Value = serde_json::from_slice(&before).unwrap();
    assert!(value.get("evidence").is_none());
    assert_eq!(value["evidence_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn malformed_missing_and_oversized_inputs_fail_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    let name = path.to_str().unwrap();
    assert!(!cli(&["evaluate", name]).status.success());
    for bytes in [
        b"{broken".to_vec(),
        vec![b' '; symbiont_core::MAX_INPUT_BYTES + 1],
    ] {
        fs::write(&path, bytes).unwrap();
        let out = cli(&["evaluate", name]);
        assert!(!out.status.success());
        assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
    }
}

#[cfg(not(feature = "zk"))]
#[test]
fn lightweight_build_cannot_emit_or_verify_fake_proofs() {
    for args in [
        vec!["prove", "anything", "receipt.json"],
        vec!["verify", "receipt.json", "--expect", "expected.json"],
    ] {
        let out = cli(&args);
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--features zk"));
    }
}
