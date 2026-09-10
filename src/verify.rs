use crate::bundle;
use crate::report::{self, Report};
use crate::runner::{NodeRunner, PythonRunner, Runner, RustRunner, ShellRunner};
use crate::spec;
use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tempfile::TempDir;

/// Verify a spec against a bundle. This is the entry point the
/// CLI calls. It:
///
///   1. Parses the spec.
///   2. Parses the bundle.
///   3. Selects a runner based on spec.runner.
///   4. Creates a temp directory.
///   5. Calls runner.prepare to extract files and inject tests.
///   6. Calls runner.invoke to run the tests.
///   7. Calls runner.parse to extract per-witness results.
///   8. Builds the Report.
///   9. Cleans up the temp directory.
///
/// Returns the Report on success. The caller inspects
/// report.summary.satisfied to determine the exit code.
pub fn verify(spec_path: &Path, bundle_path: &Path) -> Result<Report> {
    let parsed_spec = spec::parse_file(spec_path)
        .with_context(|| format!("parsing spec {}", spec_path.display()))?;

    let parsed_bundle = bundle::parse_file(bundle_path)
        .with_context(|| format!("parsing bundle {}", bundle_path.display()))?;

    let runner = select_runner(&parsed_spec.runner)?;

    let tmp = TempDir::new()
        .context("creating temporary directory")?;

    runner
        .prepare(tmp.path(), &parsed_spec, &parsed_bundle, &parsed_spec.witnesses)
        .context("preparing runner in temporary directory")?;

    let run_output = runner
        .invoke(tmp.path(), &parsed_spec.runner_args)
        .context("invoking runner")?;

    let witness_results = runner.parse(&run_output, &parsed_spec.witnesses);

    let report = report::builder::build(
        spec_path,
        &parsed_spec,
        &parsed_bundle,
        &run_output,
        witness_results,
    );

    // tmp is dropped here and the directory is deleted.
    Ok(report)
}

fn select_runner(name: &str) -> Result<Box<dyn Runner>> {
    match name {
        "rust" => Ok(Box::new(RustRunner::new())),
        "python" => Ok(Box::new(PythonRunner)),
        "node" => Ok(Box::new(NodeRunner)),
        "shell" => Ok(Box::new(ShellRunner)),
        other => Err(anyhow!(
            "unknown runner '{}'; expected one of rust, python, node, shell",
            other
        )),
    }
}
