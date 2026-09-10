pub mod builder;
pub mod fingerprint;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub assay_version: String,
    pub spec: SpecInfo,
    pub bundle: BundleInfo,
    pub run: RunInfo,
    pub witnesses: Vec<crate::witness::WitnessResult>,
    pub summary: Summary,
    pub report_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecInfo {
    pub name: String,
    pub path: String,
    pub fingerprint: String,
    pub targets: Vec<String>,
    pub runner: String,
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleInfo {
    pub fingerprint: String,
    pub file_count: usize,
    pub byte_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunInfo {
    pub timestamp_ms: u64,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errored: usize,
    pub skipped: usize,
    pub hard_total: usize,
    pub hard_passed: usize,
    pub soft_total: usize,
    pub soft_passed: usize,
    pub satisfied: bool,
    pub objective: f64,
}

impl Report {
    /// Construct a minimal Report for testing. All fields are
    /// set to placeholders except the two fingerprints.
    pub fn empty_for_test(spec_fp: &str, bundle_fp: &str) -> Self {
        Report {
            assay_version: "2.0.0".to_string(),
            spec: SpecInfo {
                name: "test".to_string(),
                path: "test.assay".to_string(),
                fingerprint: spec_fp.to_string(),
                targets: Vec::new(),
                runner: "rust".to_string(),
                lang: "rust".to_string(),
            },
            bundle: BundleInfo {
                fingerprint: bundle_fp.to_string(),
                file_count: 0,
                byte_count: 0,
            },
            run: RunInfo {
                timestamp_ms: 0,
                duration_ms: 0,
                exit_code: 0,
                cached: false,
            },
            witnesses: Vec::new(),
            summary: Summary {
                total: 0,
                passed: 0,
                failed: 0,
                errored: 0,
                skipped: 0,
                hard_total: 0,
                hard_passed: 0,
                soft_total: 0,
                soft_passed: 0,
                satisfied: false,
                objective: 0.0,
            },
            report_fingerprint: String::new(),
        }
    }

    /// Produce the canonical JSON representation of this report,
    /// excluding the report_fingerprint field. Keys are sorted
    /// alphabetically. No insignificant whitespace. This string
    /// is the input to the SHA-256 fingerprint.
    /// Note: floats serialize with default precision. Six-decimal formatting
    /// as mentioned in SPEC.md §7.3 is deferred.
    pub fn canonical_json(&self) -> String {
        let mut value = serde_json::to_value(self)
            .expect("Report must be serializable to Value");
        if let Some(obj) = value.as_object_mut() {
            obj.remove("report_fingerprint");
        }
        serde_json::to_string(&value)
            .expect("Value must be serializable to String")
    }

    /// Canonical JSON including the report_fingerprint field.
    /// Used for the --frozen output path.
    pub fn canonical_json_with_fingerprint(&self) -> String {
        serde_json::to_string(self)
            .expect("Report must serialize to String")
    }

    /// Compute the fingerprint of this report's canonical body.
    /// Does not mutate self. Caller assigns the result to the
    /// report_fingerprint field before emitting the report.
    pub fn compute_fingerprint(&self) -> String {
        crate::report::fingerprint::compute(self)
    }
}
