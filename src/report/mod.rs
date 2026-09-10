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
