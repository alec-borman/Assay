use assay::bundle::Bundle;
use assay::report::builder::build;
use assay::runner::RunOutput;
use assay::spec::{Fixture, Objective, Spec};
use assay::witness::{Kind, Status, Witness, WitnessResult};
use std::collections::BTreeMap;
use std::path::Path;

fn make_spec() -> Spec {
    Spec {
        name: "test".to_string(),
        targets: vec!["src/lib.rs".to_string()],
        runner: "rust".to_string(),
        runner_args: Vec::new(),
        lang: "rust".to_string(),
        fixtures: Vec::<Fixture>::new(),
        witnesses: vec![
            Witness { name: "h1".to_string(), kind: Kind::Hard, body: "true".to_string() },
            Witness { name: "h2".to_string(), kind: Kind::Hard, body: "false".to_string() },
            Witness { name: "s1".to_string(), kind: Kind::Soft(1.0), body: "true".to_string() },
        ],
        objectives: Vec::<Objective>::new(),
    }
}

fn make_bundle() -> Bundle {
    let mut files = BTreeMap::new();
    files.insert("src/lib.rs".to_string(), "pub fn f() {}\n".to_string());
    Bundle {
        files,
        metadata: BTreeMap::new(),
        fingerprint: "sha256:abc".to_string(),
    }
}

fn make_run() -> RunOutput {
    RunOutput {
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        duration_ms: 42,
        per_witness: Vec::new(),
    }
}

fn results() -> Vec<WitnessResult> {
    vec![
        WitnessResult { name: "h1".to_string(), status: Status::Pass, detail: None, duration_ms: None },
        WitnessResult { name: "h2".to_string(), status: Status::Fail, detail: None, duration_ms: None },
        WitnessResult { name: "s1".to_string(), status: Status::Pass, detail: None, duration_ms: None },
    ]
}

#[test]
fn test_build_report_from_synthetic_inputs() {
    let spec = make_spec();
    let bundle = make_bundle();
    let run = make_run();
    let report = build(Path::new("test.assay"), &spec, &bundle, &run, results());

    assert_eq!(report.summary.total, 3);
    assert_eq!(report.summary.passed, 2);
    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.summary.hard_total, 2);
    assert_eq!(report.summary.hard_passed, 1);
    assert_eq!(report.summary.soft_total, 1);
    assert_eq!(report.summary.soft_passed, 1);
    assert!(!report.summary.satisfied, "h2 failed, so not satisfied");
    assert_eq!(report.summary.objective, 1.0);
    assert_eq!(report.summary.errored, 0);

    assert_eq!(report.spec.name, "test");
    assert_eq!(report.spec.runner, "rust");
    assert!(report.spec.fingerprint.starts_with("sha256:"));

    assert_eq!(report.bundle.file_count, 1);
    assert!(report.bundle.fingerprint.starts_with("sha256:"));

    assert!(report.report_fingerprint.starts_with("sha256:"));
}

#[test]
fn test_spec_fingerprint_is_deterministic() {
    let s1 = make_spec();
    let s2 = make_spec();
    assert_eq!(s1.fingerprint(), s2.fingerprint());
    assert!(s1.fingerprint().starts_with("sha256:"));
}

#[test]
fn test_satisfied_when_all_hard_pass() {
    let spec = make_spec();
    let bundle = make_bundle();
    let run = make_run();
    let all_pass = vec![
        WitnessResult { name: "h1".to_string(), status: Status::Pass, detail: None, duration_ms: None },
        WitnessResult { name: "h2".to_string(), status: Status::Pass, detail: None, duration_ms: None },
        WitnessResult { name: "s1".to_string(), status: Status::Pass, detail: None, duration_ms: None },
    ];
    let report = build(Path::new("test.assay"), &spec, &bundle, &run, all_pass);
    assert!(report.summary.satisfied);
}
