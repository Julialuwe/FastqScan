mod runner;
mod cli;

use clap::Parser;
use cli::CliArgs;

fn main() {
    let args = CliArgs::parse();

    println!("Read 1 Pfad: {:?}", args.read1);
    if let Some(read2) = args.read2 {
        println!("Read 2 Pfad: {:?}", read2);
    } else {
        println!("Nur eine Datei angegeben (Single-End)");
    }
}
