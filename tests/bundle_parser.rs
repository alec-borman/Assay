use assay::bundle::parse_file;
use std::io::Write;
use tempfile::NamedTempFile;

fn write_temp(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{}", content).unwrap();
    f
}

const MINIMAL: &str = r#"<file_summary>
<purpose>test</purpose>
</file_summary>

<directory_structure>
src/lib.rs
</directory_structure>

<files>
<file path="src/lib.rs">
<![CDATA[
pub fn f() -> i32 { 1 }
]]>
</file>
</files>"#;

#[test]
fn test_w8_bundle_parser_repomix() {
    let f = write_temp(MINIMAL);
    let bundle = parse_file(f.path()).unwrap();
    assert!(bundle.files.contains_key("src/lib.rs"));
    assert_eq!(bundle.files["src/lib.rs"], "pub fn f() -> i32 { 1 }");
}

#[test]
fn test_w9_bundle_fingerprint_deterministic() {
    let f = write_temp(MINIMAL);
    let b1 = parse_file(f.path()).unwrap();
    let b2 = parse_file(f.path()).unwrap();
    assert_eq!(b1.fingerprint, b2.fingerprint);
    assert!(b1.fingerprint.starts_with("sha256:"));
}

#[test]
fn test_w10_bundle_fingerprint_order_independent() {
    let a = r#"<file_summary/>
<directory_structure>x</directory_structure>
<files>
<file path="a.rs"><![CDATA[aa]]></file>
<file path="b.rs"><![CDATA[bb]]></file>
</files>"#;

    let b = r#"<file_summary/>
<directory_structure>x</directory_structure>
<files>
<file path="b.rs"><![CDATA[bb]]></file>
<file path="a.rs"><![CDATA[aa]]></file>
</files>"#;

    let fa = write_temp(a);
    let fb = write_temp(b);
    let ba = parse_file(fa.path()).unwrap();
    let bb = parse_file(fb.path()).unwrap();
    assert_eq!(ba.fingerprint, bb.fingerprint);
}

#[test]
fn test_w13_bundle_parser_handles_terminator_inside_cdata() {
    // One file's content contains the literal "</file>".
    // The parser must not truncate the block there.
    let xml = r#"<file_summary/>
<directory_structure>x</directory_structure>
<files>
<file path="a.rs"><![CDATA[fn f() { let s = "</file>"; }]]></file>
<file path="b.rs"><![CDATA[hello]]></file>
</files>"#;
    let f = write_temp(xml);
    let bundle = parse_file(f.path()).unwrap();
    assert_eq!(bundle.files.len(), 2);
    assert!(bundle.files["a.rs"].contains("</file>"),
        "file a must retain the literal terminator string");
    assert_eq!(bundle.files["b.rs"], "hello");
}
