# FastqScan

Fast and safe Q/C for FASTQ files.

## Usage

FastqScan is a command line (CLI) application for performing quality control of FASTQ files.

The application takes either a single FASTQ file for (single-end) sequencing
or two (paired-end) FASTQ files, and reports the following Q/C metrics:

* average base quality (Phred)
* average quality of all reads
* average proportions of `{A, C, G, T, N}` for each read position
* average G/C content per read position
* average G/C content per read
* distribution of lengths of the individual reads

The metrics are reported to STDOUT in a JSON format.

## Examples

Summarize single-end sequencing:

```shell
cargo run -- -1 data/example.R1.fastq.gz
```

Summarize paired-end sequencing:

```shell
cargo run -- -1 data/example.R1.fastq.gz -2 data/example.R2.fastq.gz
```



------------


FastqScan --> minimalistische Bioinformatik-Anwendung, die FASTQ-Dateien analysiert und verschiedene Qualitätsmetriken berechnet. Ziel des Projekts ist es, Rust durch praktische Anwendung zu lernen: Modularisierung, Traits, Fehlerbehandlung, Tests und JSON-Ausgabe.

Struktur:
* main --> Einstiegspunkt, verarbeitet CLI und steuert Ablauf
* cli --> Kommandozeilenparser mit clap
* io_utils --> Dateiöffnung mit .gz erkennung und unzip falls nötig
* runner --> war vorgegeben, Fastq-Parsing und Statistik (Traitbasiert)


Rust-Book 
* Kommandozeilenargumente mit clap 
--> https://doc.rust-lang.org/book/ch12-01-accepting-command-line-arguments.html

* BuffReader anstatt strings
--> https://doc.rust-lang.org/book/ch08-02-strings.html#reading-lines-with-bufreader

* Modulare Struktur
--> https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

* Traits/Trait-Objekte + Dynamische Dispatch
--> https://doc.rust-lang.org/book/ch10-02-traits.html
--> https://doc.rust-lang.org/book/ch18-02-trait-objects.html

* Errorhandling
--> https://doc.rust-lang.org/book/ch09-00-error-handling.html

* Testen
--> https://doc.rust-lang.org/book/ch11-01-writing-tests.html

* JSON
--> https://docs.rs/json/latest/json/

