#!/bin/bash
cat << 'INNER' > src/runner/rust.rs
use crate::bundle::Bundle;
use crate::runner::{RunOutput, Runner};
use crate::spec::Spec;
use crate::witness::{Status, Witness, WitnessResult};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub struct RustRunner;

impl RustRunner {
    pub fn new() -> Self { RustRunner }
}

impl Runner for RustRunner {
    fn name(&self) -> &'static str { "rust" }

    fn prepare(
        &self,
        dir: &Path,
        spec: &Spec,
        bundle: &Bundle,
        witnesses: &[Witness],
    ) -> Result<()> {
        if spec.targets.is_empty() {
            return Err(anyhow!("spec has no targets"));
        }
        if spec.targets.len() > 1 {
            return Err(anyhow!(
                "phase 1 runner supports exactly one target; spec has {}",
                spec.targets.len()
            ));
        }

        for (path, content) in &bundle.files {
            let p = dir.join(path);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&p, content)?;
        }

        let target = &spec.targets[0];
        let content = bundle.files.get(target).ok_or_else(|| {
            anyhow!("target '{}' not found in bundle", target)
        })?;

        // Inject the witness module at the end of the target.
        let injected = inject_witness_module(content, spec, witnesses);

        // Prepend #![allow(...)] attributes to silence warnings from
        // the witness module. Only if the file does not already start
        // with a crate-level attribute block.
        let final_content = wrap_with_crate_attrs(&injected);

        let target_path = dir.join(target);
        fs::write(&target_path, final_content)?;

        if !bundle.files.contains_key("Cargo.toml") {
            // Write Cargo.toml declaring a lib crate with the target path.
            let cargo_toml = format!(
                r#"[package]
name = "assay_temp_verify"
version = "0.0.0"
edition = "2021"
publish = false

[lib]
path = "{}"

[profile.test]
debug = false
"#,
                target
            );
            fs::write(dir.join("Cargo.toml"), cargo_toml)?;
        }

        Ok(())
    }

    fn invoke(&self, dir: &Path, args: &[String]) -> Result<RunOutput> {
        let start = Instant::now();
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        for a in args {
            cmd.arg(a);
        }
        cmd.current_dir(dir);

        let output = cmd
            .output()
            .with_context(|| "failed to spawn cargo (is it on PATH?)")?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(RunOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration_ms,
            per_witness: Vec::new(),
        })
    }

    fn parse(&self, output: &RunOutput, witnesses: &[Witness]) -> Vec<WitnessResult> {
        let mut combined = String::with_capacity(output.stdout.len() + output.stderr.len() + 1);
        combined.push_str(&output.stdout);
        combined.push('\n');
        combined.push_str(&output.stderr);

        let mut results = Vec::with_capacity(witnesses.len());

        for (idx, w) in witnesses.iter().enumerate() {
            let test_name = sanitize_test_name(idx, &w.name);
            let full_path = format!("assay_tests::{}", test_name);

            let status = find_test_status(&combined, &full_path);

            let detail = if matches!(status, Status::Fail) {
                find_panic_detail(&combined, &full_path)
            } else if matches!(status, Status::Error) {
                Some("witness did not appear in test output".to_string())
            } else {
                None
            };

            results.push(WitnessResult {
                name: w.name.clone(),
                status,
                detail,
                duration_ms: None,
            });
        }

        results
    }
}

// ---------- helpers ----------

fn inject_witness_module(content: &str, spec: &Spec, witnesses: &[Witness]) -> String {
    let mut out = String::with_capacity(content.len() + 1024);

    // Strip any previously injected block. Idempotent.
    const MARKER: &str = "// ===== ASSAY GENERATED TESTS =====";
    if let Some(pos) = content.find(MARKER) {
        out.push_str(content[..pos].trim_end());
        out.push('\n');
    } else {
        out.push_str(content);
        if !content.ends_with('\n') {
            out.push('\n');
        }
    }

    out.push('\n');
    out.push_str(MARKER);
    out.push('\n');
    out.push_str("#[cfg(test)]\n");
    out.push_str("mod assay_tests {\n");
    out.push_str("    #![allow(unused_imports, dead_code, unused_variables, unused_mut)]\n");
    out.push_str("    use super::*;\n\n");

    // Fixtures concatenated in order.
    for f in &spec.fixtures {
        out.push_str("    // fixture: ");
        out.push_str(&f.name);
        out.push('\n');
        for line in f.body.lines() {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }

    // One test per witness.
    for (idx, w) in witnesses.iter().enumerate() {
        let name = sanitize_test_name(idx, &w.name);
        out.push_str("    #[test]\n");
        out.push_str("    fn ");
        out.push_str(&name);
        out.push_str("() {\n");
        out.push_str("        assert!(\n");
        out.push_str("            { ");
        out.push_str(&w.body);
        out.push_str(" },\n");
        out.push_str(&format!(
            "            \"assay witness '{}' failed\"\n",
            escape_for_rust_string(&w.name)
        ));
        out.push_str("        );\n");
        out.push_str("    }\n\n");
    }

    out.push_str("}\n");
    out
}

fn wrap_with_crate_attrs(content: &str) -> String {
    // If the file already starts with an inner attribute block, leave
    // it alone. Otherwise prepend allow attributes.
    let trimmed = content.trim_start();
    if trimmed.starts_with("#![") {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len() + 64);
    out.push_str("#![allow(dead_code, unused_imports, unused_variables, unused_mut)]\n");
    out.push_str(content);
    out
}

fn sanitize_test_name(idx: usize, name: &str) -> String {
    let mut s = String::with_capacity(name.len() + 8);
    s.push_str("witness_");
    s.push_str(&idx.to_string());
    s.push('_');
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if c == '_' || c == '-' || c == ' ' {
            s.push('_');
        }
    }
    // Collapse consecutive underscores.
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c == '_' {
            if !prev_underscore {
                out.push(c);
            }
            prev_underscore = true;
        } else {
            out.push(c);
            prev_underscore = false;
        }
    }
    out.trim_end_matches('_').to_string()
}

fn find_test_status(combined: &str, full_path: &str) -> Status {
    // Match lines like:
    //   test assay_tests::witness_0_empty_input ... ok
    //   test assay_tests::witness_1_foo ... FAILED
    let needle = format!("test {} ...", full_path);
    for line in combined.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&needle) {
            let tail = trimmed[needle.len()..].trim();
            if tail.starts_with("ok") {
                return Status::Pass;
            }
            if tail.starts_with("FAILED") {
                return Status::Fail;
            }
            if tail.starts_with("ignored") {
                return Status::Skip;
            }
        }
    }
    Status::Error
}

fn find_panic_detail(combined: &str, full_path: &str) -> Option<String> {
    // Cargo test emits failing test details in a section like:
    //   ---- assay_tests::witness_1_foo stdout ----
    //   thread 'assay_tests::witness_1_foo' panicked at ...
    //   <assertion message>
    //
    // We capture everything between the header and the next blank
    // line, up to 20 lines.
    let header = format!("---- {} stdout ----", full_path);
    let mut capturing = false;    let mut lines: Vec<&str> = Vec::new();
    for line in combined.lines() {
        if capturing {
            if line.trim().is_empty() {
                if !lines.is_empty() {
                    break;
                }
                continue;
            }
            lines.push(line);
            if lines.len() >= 20 {
                break;
            }
        } else if line.trim() == header {
            capturing = true;
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn escape_for_rust_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}
INNER
bash patch_rust.sh