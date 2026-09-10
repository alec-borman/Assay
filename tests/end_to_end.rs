use assay::verify::verify;
use std::io::Write;
use tempfile::TempDir;

/// Write a file at `dir/rel`, creating parent directories.
fn write_file(dir: &std::path::Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

/// Construct a bundle XML from a set of (path, content) pairs.
/// This is the minimal Repomix shape our parser accepts.
fn make_bundle_xml(files: &[(&str, &str)]) -> String {
    let mut out = String::new();
    out.push_str("<file_summary>\n<purpose>test</purpose>\n</file_summary>\n\n");
    out.push_str("<directory_structure>\n");
    for (p, _) in files {
        out.push_str(p);
        out.push('\n');
    }
    out.push_str("</directory_structure>\n\n<files>\n");
    for (p, c) in files {
        out.push_str(&format!(
            "<file path=\"{}\">\n<![CDATA[\n{}\n]]>\n</file>\n",
            p, c
        ));
    }
    out.push_str("</files>\n");
    out
}

#[test]
fn test_end_to_end_passing_spec() {
    let tmp = TempDir::new().unwrap();

    // The spec targets src/lib.rs and asserts a single fact.
    let spec_content = r#"
spec "e2e"
target "src/lib.rs"
runner rust
lang rust

witness "add works" hard {
    add(2, 3) == 5
}
"#;

    let lib_content = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

    let bundle_xml = make_bundle_xml(&[("src/lib.rs", lib_content)]);

    write_file(tmp.path(), "e2e.assay", spec_content);
    write_file(tmp.path(), "bundle.xml", &bundle_xml);

    let report = verify(
        &tmp.path().join("e2e.assay"),
        &tmp.path().join("bundle.xml"),
    )
    .expect("verify should succeed");

    assert!(report.summary.satisfied, "spec should be satisfied");
    assert_eq!(report.summary.hard_total, 1);
    assert_eq!(report.summary.hard_passed, 1);
    assert_eq!(report.summary.failed, 0);
    assert!(report.report_fingerprint.starts_with("sha256:"));
    assert!(report.spec.fingerprint.starts_with("sha256:"));
    assert!(report.bundle.fingerprint.starts_with("sha256:"));
}

#[test]
fn test_end_to_end_failing_spec() {
    let tmp = TempDir::new().unwrap();

    let spec_content = r#"
spec "e2e"
target "src/lib.rs"
runner rust
lang rust

witness "add works" hard {
    add(2, 3) == 5
}
"#;

    // The stub returns the wrong answer, so the witness fails.
    let lib_content = "pub fn add(a: i32, b: i32) -> i32 { a - b }\n";

    let bundle_xml = make_bundle_xml(&[("src/lib.rs", lib_content)]);

    write_file(tmp.path(), "e2e.assay", spec_content);
    write_file(tmp.path(), "bundle.xml", &bundle_xml);

    let report = verify(
        &tmp.path().join("e2e.assay"),
        &tmp.path().join("bundle.xml"),
    )
    .expect("verify should complete even on failure");

    assert!(!report.summary.satisfied, "spec should NOT be satisfied");
    assert_eq!(report.summary.hard_total, 1);
    assert_eq!(report.summary.hard_passed, 0);
    assert_eq!(report.summary.failed, 1);
}

#[test]
fn test_end_to_end_unknown_runner() {
    let tmp = TempDir::new().unwrap();

    let spec_content = r#"
spec "e2e"
target "src/lib.rs"
runner cobol
lang rust

witness "x" hard {
    true
}
"#;

    write_file(tmp.path(), "e2e.assay", spec_content);
    write_file(tmp.path(), "bundle.xml", &make_bundle_xml(&[("src/lib.rs", "")]));

    let result = verify(
        &tmp.path().join("e2e.assay"),
        &tmp.path().join("bundle.xml"),
    );

    assert!(result.is_err(), "unknown runner should error");
}
