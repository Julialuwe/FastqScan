use std::io::{self, BufRead};

use flate2::read;
use std::any::Any;
use serde_json::json;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FastqRecord {
    seq: Vec<u8>,
    qual: Vec<u8>,
}

pub trait Statistic {
    /* Statistics:
     * average base quality (Phred)
     * average quality of all reads
     * average proportions of `{A, C, G, T, N}` for each read position
     * ...
     */

    fn process(&mut self, record: &FastqRecord);
    fn as_any(&self) -> &dyn Any;
    fn report_json(&self) -> serde_json::Value;


    // TODO - find a way to represent the results.
    // Let's try to identify the shared parts of *any* statistic
    // and report these in some fashion.
    // fn report(self) -> ?
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


/*impl Statistic for BaseQualityPosStatistic {
    fn process(&mut self, record: &FastqRecord) {
        let len = record.qual.len();

        // Bei Bedarf Vektoren erweitern
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}*/


/// Computes mean base quality for a read.
pub struct ReadQualityStatistic {
    pub total_quality: f64,
    pub read_count: usize,
}

impl Default for  ReadQualityStatistic {
    fn default() -> Self {
        Self { 
            total_quality: 0.0, 
            read_count: 0, 
        }
    }
}

impl Statistic for ReadQualityStatistic {
    fn process(&mut self, record: &FastqRecord) {
        let read_quality: f64 = record.qual
            .iter()
            .map(|&q| (q - 33) as f64)
            .sum::<f64>() / record.qual.len() as f64;

        self.total_quality += read_quality;
        self.read_count += 1;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

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
    pub statistics: Vec<Box<dyn Statistic>>,
}

impl WorkflowRunner {
    /// Process the FASTQ file.
    ///
    /// Can return an I/O error or other errors (not in the signature at this point)
    pub fn new() -> Self {
        Self {
            statistics: Vec::new(),
        }
    }
    pub fn process<R>(&mut self, mut read: R)
    where
        R: BufRead,
    {
        let mut record = FastqRecord::default();

        while let Ok(()) = WorkflowRunner::parse_record(&mut read, &mut record) {
            for statistic in self.statistics.iter_mut() {
                statistic.process(&record);
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

        // Line 3 --> +
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

    pub fn finalize(self) -> Vec<Box<dyn Statistic>> {
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

  

}
