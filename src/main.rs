mod cli;
mod runner;

use std::fs::File;
use std::io::BufReader;
use clap::Parser;
use cli::CliArgs;
use runner::{WorkflowRunner, FastqRecord};

fn main() {
    let args = CliArgs::parse();

    let file = File::open(args.read1).expect("Fehler beim Öffnen der Datei");
    let reader = BufReader::new(file);

    let mut runner = WorkflowRunner {
        statistics: vec![], // Noch leer, aber Parser wird getestet
    };

    runner.process(reader);
    println!("Fertig mit Parsen.");
}

