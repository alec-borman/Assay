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
        Commands::Verify { spec, bundle, out: _, frozen: _ } => {
            println!("Verifying {} with bundle {}", spec, bundle);
            // Implementation of verify goes here
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
