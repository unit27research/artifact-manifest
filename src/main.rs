use anyhow::Result;
use clap::{Parser, Subcommand};
use evidence_packet::{CreatePacketOptions, create_packet, write_packet};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "evidence-packet")]
#[command(about = "Package local artifacts with claim scope, limitations, and hashes.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Create {
        artifact_dir: PathBuf,
        #[arg(long)]
        claim: String,
        #[arg(long)]
        scope: String,
        #[arg(long = "limitations")]
        limitations: Vec<String>,
        #[arg(long, default_value = ".")]
        output: PathBuf,
        #[arg(long)]
        allow_risky: bool,
        #[arg(long = "risk-reviewed")]
        risk_review_note: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            artifact_dir,
            claim,
            scope,
            limitations,
            output,
            allow_risky,
            risk_review_note,
        } => {
            let packet = create_packet(CreatePacketOptions {
                artifact_dir,
                declared_claim: claim,
                supported_scope: scope,
                limitations,
                allow_risky,
                risk_review_note,
            })?;
            write_packet(&packet, &output)?;
            println!(
                "Evidence packet written to {}",
                output.canonicalize().unwrap_or(output).display()
            );
        }
    }

    Ok(())
}
