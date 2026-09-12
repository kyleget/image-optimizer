use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, error::ErrorKind};
use image_optimizer::{InspectReport, InspectRequest, inspect, invalid_invocation_report};

#[derive(Debug, Parser)]
#[command(name = "image-optimizer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Inspect {
        input: PathBuf,
        #[arg(long)]
        candidate_dir: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) {
                error.print().expect("writing CLI help cannot fail");
                return ExitCode::SUCCESS;
            }
            write_report(&invalid_invocation_report());
            eprint!("{error}");
            return ExitCode::from(2);
        }
    };
    let Command::Inspect {
        input,
        candidate_dir,
    } = cli.command;

    let report = inspect(InspectRequest {
        input,
        candidate_dir,
    });
    write_report(&report);
    if let Some(diagnostic) = report.diagnostic() {
        eprintln!("image-optimizer: {diagnostic}");
    }
    if report.has_failures() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn write_report(report: &InspectReport) {
    serde_json::to_writer(std::io::stdout().lock(), report)
        .expect("serializing an inspection report cannot fail");
    println!();
}
