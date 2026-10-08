//! File helpers: transparent gzip by extension, JSON stats output.

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

pub fn open(path: &Path) -> io::Result<Box<dyn BufRead>> {
    let file = File::open(path)?;
    if path.extension().is_some_and(|e| e == "gz") {
        Ok(Box::new(BufReader::with_capacity(
            1 << 20,
            MultiGzDecoder::new(file),
        )))
    } else {
        Ok(Box::new(BufReader::with_capacity(1 << 20, file)))
    }
}

pub fn create(path: &Path) -> io::Result<Box<dyn Write>> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    let file = BufWriter::new(File::create(path)?);
    if path.extension().is_some_and(|e| e == "gz") {
        // Gzip header mtime stays 0, so output is byte-identical across runs.
        Ok(Box::new(GzEncoder::new(file, Compression::default())))
    } else {
        Ok(Box::new(file))
    }
}

pub fn write_json(path: &Path, value: &impl serde::Serialize) -> io::Result<()> {
    let mut out = create(path)?;
    serde_json::to_writer_pretty(&mut out, value)?;
    writeln!(out)?;
    out.flush()
}
