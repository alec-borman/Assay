use assay::report::Report;

#[test]
fn test_w11_report_fingerprint_deterministic() {
    let r = Report::empty_for_test("specfp", "bundfp");
    let s1 = r.canonical_json();
    let s2 = r.canonical_json();
    assert_eq!(s1, s2);
}

#[test]
fn test_w12_report_canonical_sorts_keys() {
    let r = Report::empty_for_test("s", "b");
    let j = r.canonical_json();
    let pos_a = j.find("assay_version").expect("missing assay_version");
    let pos_b = j.find("\"bundle\"").expect("missing bundle");
    let pos_s = j.find("\"spec\"").expect("missing spec");
    assert!(pos_a < pos_b, "assay_version must precede bundle");
    assert!(pos_b < pos_s, "bundle must precede spec");
}

#[test]
fn test_compute_fingerprint_deterministic() {
    let r = Report::empty_for_test("s", "b");
    let f1 = r.compute_fingerprint();
    let f2 = r.compute_fingerprint();
    assert_eq!(f1, f2);
    assert!(f1.starts_with("sha256:"));
    assert_eq!(f1.len(), "sha256:".len() + 64);
}

#[test]
fn test_canonical_json_excludes_fingerprint_field() {
    let r = Report::empty_for_test("s", "b");
    let j = r.canonical_json();
    assert!(
        !j.contains("report_fingerprint"),
        "canonical_json must exclude report_fingerprint"
    );
}
