//! Streaming the dump through parallel parsers into one aggregator.
//!
//! One reader thread decompresses and batches lines, `jobs` workers parse
//! JSON, and the calling thread folds facts into the [`Table`]. Folding is
//! order-independent (see `aggregate`), so the worker count never changes the
//! output.

use crate::aggregate::Table;
use crate::edition::{self, EditionFacts};
use flate2::read::MultiGzDecoder;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::thread;

const BATCH: usize = 4096;

pub struct Options {
    pub jobs: usize,
    /// Stop after this many dump lines (for sampling and smoke runs).
    pub limit: Option<u64>,
    /// Report progress to stderr every this many lines (0 = never).
    pub progress_every: u64,
}

fn open(path: &Path) -> io::Result<Box<dyn BufRead + Send>> {
    let file = File::open(path)?;
    let reader: Box<dyn Read + Send> = if path.extension().is_some_and(|e| e == "gz") {
        Box::new(MultiGzDecoder::new(BufReader::with_capacity(1 << 20, file)))
    } else {
        Box::new(file)
    };
    Ok(Box::new(BufReader::with_capacity(1 << 22, reader)))
}

pub fn build(input: &Path, opts: &Options) -> io::Result<Table> {
    let mut reader = open(input)?;
    let (line_tx, line_rx) = sync_channel::<Vec<String>>(opts.jobs * 4);
    let (fact_tx, fact_rx) = sync_channel::<(u64, Vec<EditionFacts>)>(opts.jobs * 4);
    let line_rx = Arc::new(Mutex::new(line_rx));

    let workers: Vec<_> = (0..opts.jobs)
        .map(|_| {
            let rx = Arc::clone(&line_rx);
            let tx = fact_tx.clone();
            thread::spawn(move || loop {
                let batch = match rx.lock().expect("line channel").recv() {
                    Ok(b) => b,
                    Err(_) => return,
                };
                let mut errors = 0u64;
                let facts: Vec<EditionFacts> = batch
                    .iter()
                    .filter_map(|l| {
                        edition::parse_line(l).unwrap_or_else(|_| {
                            errors += 1;
                            None
                        })
                    })
                    .collect();
                if tx.send((errors, facts)).is_err() {
                    return;
                }
            })
        })
        .collect();
    drop(fact_tx);

    let limit = opts.limit;
    let progress_every = opts.progress_every;
    let reader_thread = thread::spawn(move || -> io::Result<u64> {
        let mut lines = 0u64;
        let mut batch = Vec::with_capacity(BATCH);
        let mut buf = String::new();
        while limit.is_none_or(|l| lines < l) {
            buf.clear();
            if reader.read_line(&mut buf)? == 0 {
                break;
            }
            lines += 1;
            batch.push(buf.trim_end_matches(['\n', '\r']).to_string());
            if batch.len() == BATCH
                && line_tx
                    .send(std::mem::replace(&mut batch, Vec::with_capacity(BATCH)))
                    .is_err()
            {
                break;
            }
            if progress_every > 0 && lines.is_multiple_of(progress_every) {
                eprintln!("identifier-tables: {lines} lines read");
            }
        }
        if !batch.is_empty() {
            let _ = line_tx.send(batch);
        }
        Ok(lines)
    });

    let mut table = Table::new();
    for (errors, facts) in fact_rx {
        table.stats.parse_errors += errors;
        for f in facts {
            table.fold(f);
        }
    }
    for w in workers {
        w.join()
            .map_err(|_| io::Error::other("parser thread panicked"))?;
    }
    table.stats.dump_lines = reader_thread
        .join()
        .map_err(|_| io::Error::other("reader thread panicked"))??;
    Ok(table)
}
