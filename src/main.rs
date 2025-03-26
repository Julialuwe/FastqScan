mod cli;
mod runner;
mod io_utils;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use cli::CliArgs;
use runner::{FastqRecord, ReadQualityStatistic, WorkflowRunner, BaseQualityPosStatistic, BaseCompositionStatistic};
use io_utils::open_fastq;
use std::process;
use std::any::Any;
use serde_json::json;

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
    runner.statistics.push(Box::new(BaseQualityPosStatistic::default()));
    runner.statistics.push(Box::new(BaseCompositionStatistic::default()));


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
    let mut json_output = serde_json::Map::new();

    for stat in stats {
        let value = stat.report_json();
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                json_output.insert(k.clone(), v.clone());
            }
        }
    }

    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());


    println!("Parsing done!");
}
