mod cli;
mod runner;
mod io_utils;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use cli::CliArgs;
use runner::{FastqRecord, ReadQualityStatistic, WorkflowRunner};
use io_utils::open_fastq;
use std::process;
use std::any::Any;

fn main() {
    let args = CliArgs::parse();

    // Open File 1 
    let reader1 = match open_fastq(&args.read1) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Fehler beim Öffnen von Datei 1 ({}): {}", args.read1.display(), e);
            process::exit(1);
        }
    };

    // init Runner and register statistics 
    let mut runner = WorkflowRunner::new();
    runner.statistics.push(Box::new(ReadQualityStatistic::default()));

    // Processing File 1 
    runner.process(reader1);

    // Optional: Processing File 2 
    if let Some(read2_path) = args.read2 {
        let reader2 = match open_fastq(&read2_path) {
            Ok(reader) => reader,
            Err(e) => {
                eprintln!("Error opening File 2 
                 ({}): {}", read2_path.display(), e);
                process::exit(1);
            }
        };
        runner.process(reader2);
    }

    // show Statistic-Results (single- and paired-end)
    let stats = runner.finalize();
    for stat in stats {
        if let Some(read_q) = stat.as_any().downcast_ref::<ReadQualityStatistic>() {
            let avg = read_q.total_quality / read_q.read_count as f64;
            println!("Average Read-Quality: {:.2}", avg);
        }
    }

    println!("Parsing done!");
}
