use std::io::{self, BufRead};

use serde_json::json;
use std::rc::Rc;
use std::cell::RefCell;


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FastqRecord {
    seq: Vec<u8>,
    qual: Vec<u8>,
}

/* 
trait Report {

    fn report(&self);
}

trait Accumulate {
    fn accumulate(&self);
}

struct A;

impl Report for A {
    fn report(&self) {
        todo!()
    }
}
impl Accumulate for A {
    fn accumulate(&self) {
        todo!()
    }
}

fn outer () {
    let a = A;

    let bla = Box::new(a);
    bla.report();
    bla.accumulate();
    let after = inner(bla);
    after.report();
    after.accumulate();
}

fn inner(val: Box<dyn Report>) -> Box<dyn Report> {
    val
}


wrapper structs:
- allow sharing single instance of a statistic for Statistics and Report
- without duplicating the underlying data

- Statistic::process` requires mutable access  
- `Report::report_json` requires shared (immutable) access
--> `Rc<RefCell<T>>` to enable interior mutability and shared ownership
        - `Rc` (Reference Counted) enables multiple owners of the same data
        - `RefCell` allows mutable access even through a shared reference (`&self`) at runtime
        - checked safely by Rust’s borrow rules

--> `RcStatistic<T>` wraps shared statistic for usage as a `Statistic`
        - calling `process()` via `borrow_mut()`

--> `RcReporter<T>` wraps the same shared statistic so it can be used as a `Report`,
        -calling `report_json()` via `borrow()`.

This  allows WorkflowRunner to treat `Statistic` and `Report` as separate
traits while internally using only one instance of each concrete statistic type.
*/ 

pub struct RcStatistic<T: Statistic>(Rc<RefCell<T>>);
impl<T: Statistic> Statistic for RcStatistic<T> {
    fn process(&mut self, record: &FastqRecord) {
        self.0.borrow_mut().process(record);
    }
}

pub struct RcReporter<T: Report>(Rc<RefCell<T>>);
impl<T: Report> Report for RcReporter<T> {
    fn report_json(&self) -> serde_json::Value {
        self.0.borrow().report_json()
    }
}


pub struct StatisticWrapper {
    pub statistic: Box<dyn Statistic>,
    pub reporter: Box<dyn Report>,
}

pub trait Statistic {
    /* Statistics:
     * average base quality (Phred)
     * average quality of all reads
     * average proportions of `{A, C, G, T, N}` for each read position
     * ...
     */

    fn process(&mut self, record: &FastqRecord);
    // TODO - find a way to represent the results.
    // Let's try to identify the shared parts of *any* statistic
    // and report these in some fashion.
    // fn report(self) -> ?
}

pub trait Report {
    fn report_json(&self) -> serde_json::Value;
}


/// Conputes distribution of lengths of the individual reads (similar to BaseCompositionStatistics)

pub struct BaseCompositionPerRead {
    total_counts: [f64; 5],
    read_count: usize,
}

impl Default for BaseCompositionPerRead {
    fn default() -> Self {
        Self {
            total_counts: [0.0; 5],
            read_count: 0, 
        }
    }
} 

impl Statistic for BaseCompositionPerRead {
    fn process(&mut self, record: &FastqRecord) {
        let mut counts = [0usize; 5];
        for &base in &record.seq {
            let idx = match base {
                b'A' => 0,
                b'C' => 1,
                b'G' => 2,
                b'T' => 3,
                _ => 4, // N or other
            };
            counts[idx] += 1;
        }

        let len = record.seq.len();
        if len > 0 {
            for i in 0..5 {
                self.total_counts[i] += counts[i] as f64 / len as f64;
            }
            self.read_count += 1;
        }
    }
}

impl Report for BaseCompositionPerRead {
    fn report_json(&self) -> serde_json::Value {
        let bases = ["A", "C", "G", "T", "N"];
        let mut composition = serde_json::Map::new();

        for (i, &base) in bases.iter().enumerate() {
            let avg = if self.read_count > 0 {
                self.total_counts[i] / self.read_count as f64
            } else {
                0.0
            };
            composition.insert(base.to_string(), json!(avg));
        }

        json!({
            "average_base_composition_per_read": composition
        })
    }
}

/// Computes average G/C content per read position
pub struct GcContentPerRead {
    gc_percent: f64, 
    counts: usize,
}

impl Default for GcContentPerRead {
    fn default() -> Self {
        Self {
            gc_percent: 0.0,
            counts: 0, 
        }
    }
} 

impl Statistic for GcContentPerRead {
    fn process(&mut self, record: &FastqRecord) {
        let gc_count = record.seq.iter().filter(|&&b| b == b'G' || b == b'C').count();
        let len = record.seq.len();

        if len > 0 {
            let fraction = gc_count as f64 / len as f64;
            self.gc_percent += fraction;
            self.counts += 1;
        }
        
    }
}

impl Report for GcContentPerRead {
    fn report_json(&self) -> serde_json::Value {
        let average_gc = if self.counts > 0 {
            self.gc_percent / self.counts as f64
        } else {
            0.0
        };

        json!({
            "average_gc_content_per_read": average_gc
        })
    }
}


/// Computes average G/C content per read position
pub struct GcContentPerPosition {
    gc_counts: Vec<usize>, 
    total_counts: Vec<usize>,
}

impl Default for GcContentPerPosition {
    fn default() -> Self {
        Self {
            gc_counts: Vec::new(),
            total_counts: Vec::new(), 
        }
    }
} 
    

impl Statistic for GcContentPerPosition {
    fn process(&mut self, record: &FastqRecord) {
        let len = record.seq.len();

        if self.gc_counts.len() < len {
            self.gc_counts.resize(len, 0);
            self.total_counts.resize(len, 0);
        }

        for (i, &base) in record.seq.iter().enumerate() {
            if base == b'G' || base == b'C' {
                self.gc_counts[i] += 1;
            }
            self.total_counts[i] += 1;
        }
    }
}

impl Report for GcContentPerPosition {
    fn report_json(&self) -> serde_json::Value {
        let gc_per_position: Vec<f64> = self.gc_counts.iter()
            .zip(self.total_counts.iter())
            .map(|(&gc, &total)| {
                if total > 0 {
                    gc as f64 / total as f64
                } else {
                    0.0
                }
            })
            .collect();
    
        json!({
            "average_gc_content_per_position": gc_per_position
        })
    }
}

/// Computes average proportions of {A, C, G, T, N} for each read position
pub struct BaseCompositionStatistic {
    base_counts: Vec<[usize; 5]>, // A,C,G,T,N → 0–4
}

impl Default for BaseCompositionStatistic {
    fn default() -> Self {
        Self {
            base_counts: Vec::new(),
        }
    }
}

impl Statistic for BaseCompositionStatistic {
    fn process(&mut self, record: &FastqRecord) {
        let len = record.seq.len();
        if self.base_counts.len() < len {
            self.base_counts.resize(len, [0; 5]);
        }

        for (i, &base) in record.seq.iter().enumerate() {
            let idx = match base {
                b'A' => 0,
                b'C' => 1,
                b'G' => 2,
                b'T' => 3,
                _ => 4, // N or others
            };
            self.base_counts[i][idx] += 1;
        }
    }
}

impl Report for BaseCompositionStatistic {
    fn report_json(&self) -> serde_json::Value {
        let proportions: Vec<_> = self.base_counts.iter().map(|counts| {
            let total: usize = counts.iter().sum();
            if total == 0 {
                json!({"A":0.0,"C":0.0,"G":0.0,"T":0.0,"N":0.0})
            } else {
                json!({
                    "A": counts[0] as f64 / total as f64,
                    "C": counts[1] as f64 / total as f64,
                    "G": counts[2] as f64 / total as f64,
                    "T": counts[3] as f64 / total as f64,
                    "N": counts[4] as f64 / total as f64,
                })
            }
        }).collect();

        json!({
            "base_composition_per_position": proportions
        })
    }
}



/// Computes mean base quality for a position read.
pub struct BaseQualityPosStatistic {
    pub total_qualities: Vec<f64>,
    pub counts: Vec<usize>,
}

impl Default for BaseQualityPosStatistic {
    fn default() -> Self {
        Self {
            total_qualities: Vec::new(),
            counts: Vec::new(), 
        }
    }
} 

impl Statistic for BaseQualityPosStatistic {
    fn process(&mut self, record: &FastqRecord) {
        let len = record.qual.len();

        if self.total_qualities.len() < len {
            self.total_qualities.resize(len, 0.0);
            self.counts.resize(len, 0);
        }

        for (i, &q) in record.qual.iter().enumerate() {
            let phred = (q - 33) as f64;
            self.total_qualities[i] += phred;
            self.counts[i] += 1;
        }
    }
}

impl Report for BaseQualityPosStatistic {
    fn report_json(&self) -> serde_json::Value {
        let averages: Vec<f64> = self.total_qualities
            .iter()
            .zip(self.counts.iter())
            .map(|(&sum, &count)| {
                if count > 0 {
                    sum / count as f64
                } else {
                    0.0
                }
            })
            .collect();

        serde_json::json!({
            "average_base_quality_per_position": averages
        })
    }
}

/// Computes mean base quality for a read.
#[derive(Default)]
pub struct ReadQualityStatistic {
    pub total_quality: f64,
    pub read_count: usize,
}

// impl Default for ReadQualityStatistic {
//     fn default() -> Self {
//         Self {
//             total_quality: 0.0,
//             read_count: 0, 
//         }
//     }
// } 
//derive

impl Statistic for ReadQualityStatistic {
    fn process(&mut self, record: &FastqRecord) {
        let read_quality: f64 = record.qual
            .iter()
            .map(|&q| (q - 33) as f64)
            .sum::<f64>() / record.qual.len() as f64;

        self.total_quality += read_quality;
        self.read_count += 1;
    }
}

impl Report for ReadQualityStatistic {
    fn report_json(&self) -> serde_json::Value {
        let average = if self.read_count > 0 {
            self.total_quality / self.read_count as f64
        } else {
            0.0
        };

        json!({
            "average_read_quality": average
        })
    }
}

pub struct WorkflowRunner {
    pub statistics: Vec<StatisticWrapper>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatisticType {
    ReadQuality,
    BaseQualityPos,
    BaseComposition,
    GcContentPos,
    GcContentRead,
    BaseCompositionRead,
}

impl WorkflowRunner { // Default ?
    /// Process the FASTQ file.
    ///
    /// Can return an I/O error or other errors (not in the signature at this point)
    pub fn new() -> Self {
        Self {
            statistics: Vec::new(),
        }
    }

    fn wrap<T: 'static + Statistic + Report>(instance: T) -> StatisticWrapper {
        let shared = Rc::new(RefCell::new(instance));
        StatisticWrapper {
            statistic: Box::new(RcStatistic(shared.clone())),
            reporter: Box::new(RcReporter(shared)),
        }
    }

    /// Creates a runner with only the selected statistics
    pub fn from_selected_statistics(selected: &[StatisticType]) -> Self {
        let mut runner = Self::new();

        runner.statistics = selected
            .iter()
            .map(|s| match s {
                StatisticType::ReadQuality => Self::wrap(ReadQualityStatistic::default()),
                StatisticType::BaseQualityPos => Self::wrap(BaseQualityPosStatistic::default()),
                StatisticType::BaseComposition => Self::wrap(BaseCompositionStatistic::default()),
                StatisticType::GcContentPos => Self::wrap(GcContentPerPosition::default()),
                StatisticType::GcContentRead => Self::wrap(GcContentPerRead::default()),
                StatisticType::BaseCompositionRead => Self::wrap(BaseCompositionPerRead::default()),
            })
            .collect();

        runner
    }

    pub fn with_default_statistics() -> Self {
        let mut runner = Self::new();

        runner.statistics = vec![
            WorkflowRunner::wrap(ReadQualityStatistic::default()),
            WorkflowRunner::wrap(BaseQualityPosStatistic::default()),
            WorkflowRunner::wrap(BaseCompositionStatistic::default()),
            WorkflowRunner::wrap(GcContentPerPosition::default()),
            WorkflowRunner::wrap(GcContentPerRead::default()),
            WorkflowRunner::wrap(BaseCompositionPerRead::default()),            
        ];

        runner
    }


    
    pub fn process<R>(&mut self, mut read: R)
    where
        R: BufRead,
    {
        let mut record = FastqRecord::default();

        while let Ok(()) = WorkflowRunner::parse_record(&mut read, &mut record) {
            for wrapper in self.statistics.iter_mut() {
                wrapper.statistic.process(&record);
            }
        }
    }

    // Read data for a complete FASTQ record from `read`.
    pub fn parse_record<R>(read: &mut R, record: &mut FastqRecord) -> io::Result<()>
    where
        R: BufRead,
    {
        let mut buffer = String::new();

        // Line 1 --> @SEQ_ID (ignoring for now)
        buffer.clear();
        if read.read_line(&mut buffer)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF before read ID"));
        }
        
        // Line 2 --> Sequence
        buffer.clear();
        if read.read_line(&mut buffer)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF before sequence"));
        }
        record.seq = buffer.trim_end().as_bytes().to_vec();

        // Line 3 --> + (ignoring for now)
        buffer.clear();
        if read.read_line(&mut buffer)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF before + line"));
        }

        // Line 4 --> Quality
        buffer.clear();
        if read.read_line(&mut buffer)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF before quality"));
        }
        record.qual = buffer.trim_end().as_bytes().to_vec();

        Ok(())
    }

    pub fn finalize(self) -> Vec<StatisticWrapper> {
        // Move out the statistics, effectively preventing the future use of the runner.
        self.statistics
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_quality_statistic_on_example() {
        let record = FastqRecord {
            seq: b"AGTC".to_vec(),
            qual: b"IIII".to_vec(), // 'I' = Phred 40
        };

        let mut stat = ReadQualityStatistic::default();
        stat.process(&record);

        assert_eq!(stat.read_count, 1);
        assert_eq!(stat.total_quality, 40.0);
    }

    #[test]
    fn test_parse_record_reads_correct_fields() {
        use std::io::BufReader;

        // Small FASTQ-Block
        let fastq_data = b"@SEQ_ID\nAGTC\n+\nIIII\n";
        let mut reader = BufReader::new(&fastq_data[..]);

        let mut record = FastqRecord::default();
        let result = WorkflowRunner::parse_record(&mut reader, &mut record);

        // Should be ok
        assert!(result.is_ok());

        // Sequence correct?
        assert_eq!(record.seq, b"AGTC");

        // Quality correct?
        assert_eq!(record.qual, b"IIII");
    }

    #[test]
    fn test_base_quality_pos_statistic() {
        let record = FastqRecord {
            seq: b"AGTC".to_vec(),
            qual: b"IIII".to_vec(), // 'I' = ASCII 73 -> Phred 40
        };

        let mut stat = BaseQualityPosStatistic::default();
        stat.process(&record);

        let expected = vec![40.0, 40.0, 40.0, 40.0];

        let json = stat.report_json();
        let result = json.get("average_base_quality_per_position").unwrap();
        let result_array = result.as_array().unwrap();

        for (i, val) in result_array.iter().enumerate() {
            let observed = val.as_f64().unwrap();
            assert!((observed - expected[i]).abs() < 1e-6, "Mismatch at position {}: got {}, expected {}", i, observed, expected[i]);
        }
    }

    #[test]
    fn test_base_composition_statistic() {
        let record = FastqRecord {
            seq: b"ACGT".to_vec(),
            qual: b"!!!!".to_vec(), // irrelevant hier
        };

        let mut stat = BaseCompositionStatistic::default();
        stat.process(&record);

        let json = stat.report_json();
        let result = json.get("base_composition_per_position").unwrap();
        let array = result.as_array().unwrap();

        let expected_bases = ["A", "C", "G", "T"];

        for (i, expected_base) in expected_bases.iter().enumerate() {
            let position_counts = &array[i];
            let freq = position_counts.get(*expected_base).unwrap().as_f64().unwrap();
            assert!((freq - 1.0).abs() < 1e-6, "Base {} at position {} was not 100%", expected_base, i);
        }

        // Check that other bases are 0.0
        for (i, expected_base) in expected_bases.iter().enumerate() {
            let position_counts = &array[i];
            for other_base in ["A", "C", "G", "T", "N"] {
                if other_base != *expected_base {
                    let freq = position_counts.get(other_base).unwrap().as_f64().unwrap();
                    assert!(freq.abs() < 1e-6, "Unexpected non-zero value for base {} at position {}", other_base, i);
                }
            }
        }
    }

    

}
