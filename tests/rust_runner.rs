use assay::bundle::Bundle;
use assay::runner::RustRunner;
use assay::runner::Runner;
use assay::spec::Spec;
use assay::witness::{Kind, Status, Witness};
use std::collections::BTreeMap;
use tempfile::TempDir;

fn spec_with(target: &str, witnesses: Vec<Witness>) -> Spec {
    Spec {
        name: "test".to_string(),
        targets: vec![target.to_string()],
        runner: "rust".to_string(),
        runner_args: Vec::new(),
        lang: "rust".to_string(),
        fixtures: Vec::new(),
        witnesses,
        objectives: Vec::new(),
    }
}

fn bundle_with(files: &[(&str, &str)]) -> Bundle {
    let mut map = BTreeMap::new();
    for (path, content) in files {
        map.insert(path.to_string(), content.to_string());
    }
    Bundle {
        files: map,
        metadata: BTreeMap::new(),
        fingerprint: "sha256:test".to_string(),
    }
}

fn hard_witness(name: &str, body: &str) -> Witness {
    Witness {
        name: name.to_string(),
        kind: Kind::Hard,
        body: body.to_string(),
    }
}

#[test]
fn test_rust_runner_passing_witness() {
    let stub = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";
    let spec = spec_with(
        "src/lib.rs",
        vec![hard_witness("adds two numbers", "add(2, 3) == 5")],
    );
    let bundle = bundle_with(&[("src/lib.rs", stub)]);

    let tmp = TempDir::new().unwrap();
    let runner = RustRunner::new();
    runner.prepare(tmp.path(), &spec, &bundle, &spec.witnesses).unwrap();

    let out = runner.invoke(tmp.path(), &["--quiet".to_string()]).unwrap();
    let results = runner.parse(&out, &spec.witnesses);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, Status::Pass);
    assert_eq!(results[0].name, "adds two numbers");
}

#[test]
fn test_rust_runner_failing_witness() {
    let stub = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
    let spec = spec_with(
        "src/lib.rs",
        vec![hard_witness("adds two numbers", "add(2, 3) == 5")],
    );
    let bundle = bundle_with(&[("src/lib.rs", stub)]);

    let tmp = TempDir::new().unwrap();
    let runner = RustRunner::new();
    runner.prepare(tmp.path(), &spec, &bundle, &spec.witnesses).unwrap();

    let out = runner.invoke(tmp.path(), &["--quiet".to_string()]).unwrap();
    let results = runner.parse(&out, &spec.witnesses);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, Status::Fail);
}

#[test]
fn test_rust_runner_multiple_witnesses() {
    let stub = "pub fn inc(x: i32) -> i32 { x + 1 }\npub fn dec(x: i32) -> i32 { x - 1 }\n";
    let spec = spec_with(
        "src/lib.rs",
        vec![
            hard_witness("inc works", "inc(5) == 6"),
            hard_witness("dec works", "dec(5) == 4"),
            hard_witness("inc and dec cancel", "inc(dec(10)) == 10"),
        ],
    );
    let bundle = bundle_with(&[("src/lib.rs", stub)]);

    let tmp = TempDir::new().unwrap();
    let runner = RustRunner::new();
    runner.prepare(tmp.path(), &spec, &bundle, &spec.witnesses).unwrap();

    let out = runner.invoke(tmp.path(), &["--quiet".to_string()]).unwrap();
    let results = runner.parse(&out, &spec.witnesses);

    assert_eq!(results.len(), 3);
    for r in &results {
        assert_eq!(r.status, Status::Pass, "witness '{}' should pass", r.name);
    }
}

#[test]
fn test_rust_runner_missing_target_errors() {
    let spec = spec_with("src/lib.rs", vec![hard_witness("x", "true")]);
    let bundle = bundle_with(&[("src/other.rs", "pub fn f() {}\n")]);

    let tmp = TempDir::new().unwrap();
    let runner = RustRunner::new();
    let res = runner.prepare(tmp.path(), &spec, &bundle, &spec.witnesses);
    assert!(res.is_err());
}
