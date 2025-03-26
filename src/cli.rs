use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path first FASTQ-File (Single-End or Read 1)
    #[arg(short = '1', long)]
    pub read1: PathBuf,

    /// Path second FASTQ-File (optional, Paired-End)
    #[arg(short = '2', long)]
    pub read2: Option<PathBuf>,
}
