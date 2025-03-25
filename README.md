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
