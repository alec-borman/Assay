use assay::spec::parser::parse_file;
use assay::witness::Kind;
use assay::spec::ObjectiveKind;
use std::io::Write;
use tempfile::NamedTempFile;

fn parse_str(content: &str) -> anyhow::Result<assay::spec::Spec> {
    let mut file = NamedTempFile::new()?;
    write!(file, "{}", content)?;
    parse_file(file.path())
}

#[test]
fn test_w3_minimal_valid_spec() {
    let content = r#"
spec "min"
target "src/lib.rs"
runner rust
lang rust

witness "w" hard {
    true
}
"#;
    let spec = parse_str(content).unwrap();
    assert_eq!(spec.name, "min");
    assert_eq!(spec.targets, vec!["src/lib.rs"]);
    assert_eq!(spec.runner, "rust");
    assert!(spec.runner_args.is_empty());
    assert_eq!(spec.lang, "rust");
    assert!(spec.fixtures.is_empty());
    assert_eq!(spec.witnesses.len(), 1);
    assert_eq!(spec.witnesses[0].name, "w");
    assert_eq!(spec.witnesses[0].kind, Kind::Hard);
    assert_eq!(spec.witnesses[0].body, "true");
}

#[test]
fn test_w4_missing_target() {
    let content = r#"
spec "bad"
runner rust
lang rust
witness "w" hard { true }
"#;
    let res = parse_str(content);
    assert!(res.is_err());
}

#[test]
fn test_w5_comments() {
    let content = r#"
# this is a comment

spec "c"
# another comment

target "src/lib.rs"
runner rust
lang rust

# comment inside
witness "w" hard {
    # this is a rust comment inside the witness
    true
}
"#;
    let spec = parse_str(content).unwrap();
    assert_eq!(spec.name, "c");
    assert_eq!(spec.witnesses.len(), 1);
    assert_eq!(spec.witnesses[0].name, "w");
    assert_eq!(spec.witnesses[0].body, "# this is a rust comment inside the witness\ntrue");
}

#[test]
fn test_w6_multiline_body() {
    let content = r#"
spec "m"
target "src/lib.rs"
runner rust
lang rust
witness "m" hard {
    let x = 1;
    let y = 2;
    x + y == 3
}
"#;
    let spec = parse_str(content).unwrap();
    assert_eq!(spec.witnesses[0].body, "let x = 1;\nlet y = 2;\nx + y == 3");
}

#[test]
fn test_w7_soft_weight() {
    let content = r#"
spec "s"
target "src/lib.rs"
runner rust
lang rust
witness "s" soft weight 0.5 {
    true
}
"#;
    let spec = parse_str(content).unwrap();
    assert_eq!(spec.witnesses[0].kind, Kind::Soft(0.5));
}

#[test]
fn test_objectives() {
    let content = r#"
spec "obj"
target "src/lib.rs"
runner rust
lang rust
objective {
    minimises "lines_of_code" target 200
    maximises "test_coverage"
}
"#;
    let spec = parse_str(content).unwrap();
    assert_eq!(spec.objectives.len(), 2);
    assert_eq!(spec.objectives[0].kind, ObjectiveKind::Minimise);
    assert_eq!(spec.objectives[0].name, "lines_of_code");
    assert_eq!(spec.objectives[0].target, Some(200.0));
    assert_eq!(spec.objectives[1].kind, ObjectiveKind::Maximise);
    assert_eq!(spec.objectives[1].name, "test_coverage");
    assert_eq!(spec.objectives[1].target, None);
}
