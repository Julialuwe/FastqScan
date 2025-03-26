use std::io::{self, BufRead};

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

    // TODO - find a way to represent the results.
    // Let's try to identify the shared parts of *any* statistic
    // and report these in some fashion.
    // fn report(self) -> ?
}

/// Computes mean base quality for a position read.
pub struct BaseQualityPosStatistic {
    
}

impl Statistic for BaseQualityPosStatistic {
    fn process(&mut self, record: &FastqRecord) {
        todo!()
    }
}

/// Computes mean base quality for a read.
pub struct ReadQualityStatistic {

}

impl Statistic for ReadQualityStatistic {
    fn process(&mut self, record: &FastqRecord) {
        todo!()
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
