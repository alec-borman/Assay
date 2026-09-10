use crate::bundle::Bundle;
use crate::report::{
    BundleInfo, Report, RunInfo, SpecInfo, Summary,
};
use crate::runner::RunOutput;
use crate::spec::Spec;
use crate::witness::{Status, WitnessResult};
use std::path::Path;

/// Build a Report from the outputs of one verification run.
///
/// The caller is responsible for having already called
/// runner.prepare, runner.invoke, and runner.parse. This
/// function combines their results into a Report.
pub fn build(
    spec_path: &Path,
    spec: &Spec,
    bundle: &Bundle,
    run: &RunOutput,
    witness_results: Vec<WitnessResult>,
) -> Report {
    let spec_info = SpecInfo {
        name: spec.name.clone(),
        path: spec_path.to_string_lossy().into_owned(),
        fingerprint: spec.fingerprint(),
        targets: spec.targets.clone(),
        runner: spec.runner.clone(),
        lang: spec.lang.clone(),
    };

    let byte_count: usize = bundle.files.values().map(|c| c.len()).sum();
    let bundle_info = BundleInfo {
        fingerprint: bundle.fingerprint.clone(),
        file_count: bundle.files.len(),
        byte_count,
    };

    let run_info = RunInfo {
        timestamp_ms: now_ms(),
        duration_ms: run.duration_ms,
        exit_code: run.exit_code,
        cached: false,
    };

    let summary = summarize(spec, &witness_results);

    let mut report = Report {
        assay_version: env!("CARGO_PKG_VERSION").to_string(),
        spec: spec_info,
        bundle: bundle_info,
        run: run_info,
        witnesses: witness_results,
        summary,
        report_fingerprint: String::new(),
    };

    report.report_fingerprint = report.compute_fingerprint();
    report
}

fn summarize(spec: &Spec, results: &[WitnessResult]) -> Summary {
    let total = results.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut errored = 0;
    let mut skipped = 0;

    for r in results {
        match r.status {
            Status::Pass => passed += 1,
            Status::Fail => failed += 1,
            Status::Skip => skipped += 1,
            Status::Error => errored += 1,
        }
    }

    // Walk witnesses and results in lockstep to classify by kind.
    let mut hard_total = 0;
    let mut hard_passed = 0;
    let mut soft_total = 0;
    let mut soft_passed = 0;
    let mut soft_weight_sum = 0.0_f64;
    let mut soft_weight_passed = 0.0_f64;

    for (w, r) in spec.witnesses.iter().zip(results.iter()) {
        match w.kind {
            crate::witness::Kind::Hard => {
                hard_total += 1;
                if matches!(r.status, Status::Pass) {
                    hard_passed += 1;
                }
            }
            crate::witness::Kind::Soft(weight) => {
                soft_total += 1;
                soft_weight_sum += weight;
                if matches!(r.status, Status::Pass) {
                    soft_passed += 1;
                    soft_weight_passed += weight;
                }
            }
        }
    }

    // The objective is 1.0 when there are no soft witnesses, as the empty sum
    // over an empty set is trivially satisfied.
    let objective = if soft_weight_sum > 0.0 {
        soft_weight_passed / soft_weight_sum
    } else {
        1.0
    };

    let satisfied = hard_total == hard_passed && errored == 0;

    Summary {
        total,
        passed,
        failed,
        errored,
        skipped,
        hard_total,
        hard_passed,
        soft_total,
        soft_passed,
        satisfied,
        objective,
    }
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
