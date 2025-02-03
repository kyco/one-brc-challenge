use std::env;
use std::fs::File;
use std::io;
use rayon::prelude::*;
use ahash::AHashMap;
use memmap2::Mmap;
use memchr::{memchr, memchr_iter};
use lexical_core;

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

fn merge_maps(
    mut global: AHashMap<&'static str, Stats>,
    local: AHashMap<&'static str, Stats>,
) -> AHashMap<&'static str, Stats> {
    for (station, stats) in local {
        global.entry(station).and_modify(|gstats| {
            if stats.min < gstats.min { gstats.min = stats.min; }
            if stats.max > gstats.max { gstats.max = stats.max; }
            gstats.sum += stats.sum;
            gstats.count += stats.count;
        }).or_insert(stats);
    }
    global
}

// Process a chunk using memchr_iter for line splitting and lexical_core for f64 parsing.
fn process_chunk(chunk: &'static str) -> AHashMap<&'static str, Stats> {
    let bytes = chunk.as_bytes();
    let mut local_map: AHashMap<&'static str, Stats> = AHashMap::with_capacity(1024);
    let mut line_start = 0;
    for line_end in memchr_iter(b'\n', bytes) {
        if line_end > line_start {
            let line = &bytes[line_start..line_end];
            if let Some(delim_idx) = memchr(b';', line) {
                let station = unsafe { std::str::from_utf8_unchecked(&line[..delim_idx]) };
                let measurement = unsafe { std::str::from_utf8_unchecked(&line[delim_idx+1..]) };
                if let Ok(value) = lexical_core::parse::<f64>(measurement.as_bytes()) {
                    local_map.entry(station)
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
        line_start = line_end + 1;
    }
    if line_start < bytes.len() {
        let line = &bytes[line_start..];
        if let Some(delim_idx) = memchr(b';', line) {
            let station = unsafe { std::str::from_utf8_unchecked(&line[..delim_idx]) };
            let measurement = unsafe { std::str::from_utf8_unchecked(&line[delim_idx+1..]) };
            if let Ok(value) = lexical_core::parse::<f64>(measurement.as_bytes()) {
                local_map.entry(station)
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
    local_map
}

fn main() -> io::Result<()> {
    // Check command-line arguments.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <measurements.txt>", args[0]);
        std::process::exit(1);
    }
    let file = File::open(&args[1])?;
    let mmap = unsafe { Mmap::map(&file)? };
    // Leak the mmap to obtain a 'static lifetime.
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
    let global_map: AHashMap<&'static str, Stats> = ranges
        .into_par_iter()
        .map(|(s, e)| {
            let chunk = &content[s..e];
            process_chunk(chunk)
        })
        .reduce(|| AHashMap::new(), merge_maps);
    let mut stations: Vec<(&'static str, Stats)> = global_map.into_iter().collect();
    stations.sort_by(|a, b| a.0.cmp(b.0));
    let total = stations.len();
    if total > 20 {
        for (station, stats) in stations.iter().take(10) {
            let mean = stats.sum / (stats.count as f64);
            println!("{};{:.1};{:.1};{:.1}", station, stats.min, mean, stats.max);
        }
        println!("... ({} records omitted) ...", total - 20);
        for (station, stats) in stations.iter().rev().take(10).collect::<Vec<_>>().into_iter().rev() {
            let mean = stats.sum / (stats.count as f64);
            println!("{};{:.1};{:.1};{:.1}", station, stats.min, mean, stats.max);
        }
    } else {
        for (station, stats) in stations {
            let mean = stats.sum / (stats.count as f64);
            println!("{};{:.1};{:.1};{:.1}", station, stats.min, mean, stats.max);
        }
    }
    Ok(())
}