mod cli;
mod runner;
mod io_utils;
mod reporting;

use clap::Parser;
use cli::CliArgs;
use runner::WorkflowRunner;
use io_utils::open_fastq;
use serde_json::Value;
use std::process;


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
    let mut runner = WorkflowRunner::with_default_statistics();


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

    for wrapper in stats {
        let value = wrapper.reporter.report_json();
        if let Some(obj) = value.as_object() {
            for (k, v) in obj {
                json_output.insert(k.clone(), v.clone());
            }
        }
    }

    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());


    //still missing: json output into file
    reporting::print_combined_table(&Value::Object(json_output));


    println!("Parsing done!");
}
