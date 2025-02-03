use std::env;
use std::fs::File;
use std::io;
use rayon::prelude::*;
use ahash::AHashMap;
use memmap2::Mmap;
use memchr::memchr;

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

// Merge two hash maps
fn merge_maps(
    mut global: AHashMap<String, Stats>,
    local: AHashMap<String, Stats>,
) -> AHashMap<String, Stats> {
    for (station, stats) in local {
        global.entry(station).and_modify(|gstats| {
            if stats.min < gstats.min {
                gstats.min = stats.min;
            }
            if stats.max > gstats.max {
                gstats.max = stats.max;
            }
            gstats.sum += stats.sum;
            gstats.count += stats.count;
        }).or_insert(stats);
    }
    global
}

// Process a chunk using byte-level parsing with memchr for speed.
fn process_chunk(chunk: &'static str) -> AHashMap<String, Stats> {
    let bytes = chunk.as_bytes();
    let mut pos = 0;
    let len = bytes.len();
    let mut local_map: AHashMap<String, Stats> = AHashMap::new();
    while pos < len {
        // Find end of line
        let line_end = match memchr(b'\n', &bytes[pos..]) {
            Some(rel) => pos + rel,
            None => len,
        };
        if line_end > pos {
            // Find delimiter ';' in the line
            if let Some(delim_rel) = memchr(b';', &bytes[pos..line_end]) {
                let delim = pos + delim_rel;
                // Convert station and measurement to &str.
                // Safety: The file is valid UTF-8.
                let station = unsafe { std::str::from_utf8_unchecked(&bytes[pos..delim]) };
                let measurement = unsafe { std::str::from_utf8_unchecked(&bytes[delim + 1..line_end]) };
                if let Ok(value) = measurement.parse::<f64>() {
                    local_map.entry(station.to_owned())
                        .and_modify(|stats| {
                            if value < stats.min { stats.min = value; }
                            if value > stats.max { stats.max = value; }
                            stats.sum += value;
                            stats.count += 1;
                        })
                        .or_insert(Stats {
                            min: value,
                            max: value,
                            sum: value,
                            count: 1,
                        });
                }
            }
        }
        pos = line_end + 1;
    }
    local_map
}

fn main() -> io::Result<()> {
    // Open file and memory-map it.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <measurements.txt>", args[0]);
        std::process::exit(1);
    }
    let file = File::open(&args[1])?;
    let mmap = unsafe { Mmap::map(&file)? };
    // Leak mmap to get a 'static lifetime.
    let leaked_mmap: &'static Mmap = Box::leak(Box::new(mmap));
    // SAFETY: measurements.txt is assumed to be valid UTF-8.
    let content: &'static str = unsafe { std::str::from_utf8_unchecked(&leaked_mmap[..]) };
    let total_len = content.len();
    let num_workers = 8;
    let chunk_size = total_len / num_workers;
    let mut ranges = Vec::with_capacity(num_workers);
    let mut start = 0;
    for _ in 0..num_workers {
        let end = if start + chunk_size >= total_len {
            total_len
        } else {
            let mut pos = start + chunk_size;
            while pos < total_len && content.as_bytes()[pos] != b'\n' {
                pos += 1;
            }
            if pos < total_len { pos + 1 } else { total_len }
        };
        ranges.push((start, end));
        start = end;
    }
    // Process chunks in parallel using Rayon.
    let global_map: AHashMap<String, Stats> = ranges
        .into_par_iter()
        .map(|(s, e)| {
            let chunk = &content[s..e];
            process_chunk(chunk)
        })
        .reduce(|| AHashMap::new(), merge_maps);
    let mut stations: Vec<(String, Stats)> = global_map.into_iter().collect();
    stations.sort_by(|a, b| a.0.cmp(&b.0));
    for (station, stats) in stations {
        let mean = stats.sum / (stats.count as f64);
        println!("{};{:.1};{:.1};{:.1}", station, stats.min, mean, stats.max);
    }
    Ok(())
}