mod cli;
mod runner;
mod io_utils;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use cli::CliArgs;
use runner::{WorkflowRunner, FastqRecord};
use io_utils::open_fastq;
use std::process;


fn main() {
    let args = CliArgs::parse();

    // Try Opening File or throw error
    let reader1 = match open_fastq(&args.read1) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Fehler beim Öffnen von Datei 1 ({}): {}", args.read1.display(), e);
            process::exit(1);
        }
    };

    let mut runner = WorkflowRunner::new(); // falls du den Konstruktor nutzt
    runner.process(reader1);

    // Optional: Second file 
    if let Some(read2_path) = args.read2 {
        let reader2 = match open_fastq(&read2_path) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Fehler beim Öffnen von Datei 2 ({}): {}", read2_path.display(), e);
                process::exit(1);
            }
        };
        runner.process(reader2);
    }

    println!("Parsing done!");
}

