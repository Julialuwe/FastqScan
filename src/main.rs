mod cli;
mod runner;
mod io_utils;
mod reporting;

use clap::Parser;
use cli::CliArgs;
use runner::{WorkflowRunner, StatisticType};
use io_utils::open_fastq;
use serde_json::Value;
use std::{io::BufReader, process};



fn main() {
    let args = CliArgs::parse();
    let mut readers = Vec::new();

    // Open File 1 
    let reader1 = match open_fastq(&args.read1) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("Fehler beim Öffnen von Datei 1 ({}): {}", args.read1.display(), e);
            process::exit(1);
        }
    };
    readers.push(reader1);

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
        readers.push(reader2);
    }

    let selected = vec![
    StatisticType::ReadQuality,
    StatisticType::BaseQualityPos,
    StatisticType::BaseComposition,
    StatisticType::GcContentPos,
    StatisticType::GcContentRead,
    StatisticType::ReadLength,
    ];

    for reader in readers {

        
        //let mut runner = WorkflowRunner::with_default_statistics();
        let mut runner = WorkflowRunner::from_selected_statistics(&selected);

        runner.process(reader);
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

        //println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        reporting::print_combined_table(&Value::Object(json_output.clone()));
        reporting::print_length_distribution(&Value::Object(json_output.clone()));

        println!("Parsing of File done!");
    }


    

    //Loop for multiple files
    //testing json output 
    
}
