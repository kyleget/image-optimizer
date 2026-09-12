use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use image_optimizer::{InspectRequest, inspect};

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
    let cli = Cli::parse();
    let Command::Inspect {
        input,
        candidate_dir,
    } = cli.command;

    match inspect(InspectRequest {
        input,
        candidate_dir,
    }) {
        Ok(report) => {
            serde_json::to_writer(std::io::stdout().lock(), &report)
                .expect("serializing an inspection report cannot fail");
            println!();
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("image-optimizer: {error}");
            ExitCode::from(1)
        }
    }
}
