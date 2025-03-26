use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)] 
/* 
Usage: Reading command line arguments with clap
- clap similar to argparse in python
- Pathbuf similar to string but explicitly for paths
- similar to pathlib Path python
- derive automatically generates help and validations
    - Parser makes struct CLI-Argument-parser
    - Debug allows print statements
- pub struct --> wie öffentliche Klasse
- pflichteingabe -1
- option ist wie optional in python -2
*/ 
pub struct CliArgs {
    /// Path first FASTQ-File (Single-End or Read 1)
    #[arg(short = '1', long)]
    pub read1: PathBuf, //BathBuf better than string 

    /// Path second FASTQ-File (optional, Paired-End)
    #[arg(short = '2', long)]
    pub read2: Option<PathBuf>,
}
