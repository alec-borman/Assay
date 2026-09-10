#!/bin/bash
cat << 'INNER' > src/main.rs
use clap::{Parser, Subcommand};
use anyhow::Result;
use assay::*;

#[derive(Parser)]
#[command(name = "assay")]
#[command(version = "2.0.0")]
#[command(about = "A Universal Verifier and Oracle Protocol", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify a bundle against a spec. Emit a report.
    Verify {
        spec: String,
        #[arg(long)]
        bundle: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        frozen: bool,
    },
    /// Same as verify, but always exit 0.
    Report {
        spec: String,
        #[arg(long)]
        bundle: String,
        #[arg(long)]
        out: Option<String>,
        #[arg(long)]
        frozen: bool,
    },
    /// Compare two reports.
    Diff {
        report_a: String,
        report_b: String,
    },
    /// Invoke the oracle. Emit a directive.
    Oracle {
        spec: String,
        #[arg(long)]
        report: String,
        #[arg(long)]
        provider: Option<String>,
    },
    /// Run the loop.
    Loop {
        spec: String,
        #[arg(long)]
        bundle: Option<String>,
    },
    /// Scaffold a new project.
    Init {
        dir: Option<String>,
    },
    /// Canonicalize a spec.
    Fmt {
        spec: String,
    },
    /// Print JSON schemas for reports and directives.
    Schema,
    /// Print version and exit.
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Verify { spec, bundle, out, frozen } => {
            use std::path::Path;
            let report = assay::verify::verify(
                Path::new(spec),
                Path::new(bundle),
            )?;

            let json = if *frozen {
                // Frozen reports strip the two nondeterministic fields.
                // For phase 1 we emit the same JSON; --frozen is a hint
                // for future caching, not a different serialization.
                report.canonical_json_with_fingerprint()
            } else {
                serde_json::to_string_pretty(&report)
                    .expect("Report must serialize")
            };

            match out {
                Some(path) => {
                    std::fs::write(path, &json)?;
                    eprintln!("wrote report to {}", path);
                }
                None => {
                    println!("{}", json);
                }
            }

            if report.summary.satisfied {
                return Ok(());
            } else {
                std::process::exit(1);
            }
        }
        Commands::Report { .. } => {
            println!("Generating report...");
        }
        Commands::Diff { .. } => {
            println!("Diffing reports...");
        }
        Commands::Oracle { .. } => {
            println!("Invoking oracle...");
        }
        Commands::Loop { .. } => {
            println!("Starting loop...");
        }
        Commands::Init { .. } => {
            println!("Initializing...");
        }
        Commands::Fmt { .. } => {
            println!("Formatting spec...");
        }
        Commands::Schema => {
            println!("Outputting schema...");
        }
        Commands::Version => {
            println!("assay {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
    }
    Ok(())
}
INNER
