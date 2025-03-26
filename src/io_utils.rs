use std::fs::File;
use std::io::{self, BufReader, Read};
use flate2::read::GzDecoder;
use std::path::Path;

// Opens File and detects if gzip Type
pub fn open_fastq<P: AsRef<Path>>(path: P) -> io::Result<BufReader<Box<dyn Read>>> {
    let path_ref = path.as_ref();
    let file = File::open(path_ref)?;

    if let Some(ext) = path_ref.extension() {
        if ext == "gz" {
            let decoder = GzDecoder::new(file);
            return Ok(BufReader::new(Box::new(decoder)));
        }
    }

    Ok(BufReader::new(Box::new(file)))
}
