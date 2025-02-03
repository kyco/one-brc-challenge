use std::env;
use std::fs::File;
use std::io;
use std::sync::Arc;

use ahash::AHashMap;
use memmap2::Mmap;
use rayon::prelude::*;

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

fn merge_maps(
    mut global: AHashMap<String, Stats>,
    local: AHashMap<String, Stats>,
) -> AHashMap<String, Stats> {
    for (station, stats) in local {
        global
            .entry(station)
            .and_modify(|gstats| {
                if stats.min < gstats.min {
                    gstats.min = stats.min;
                }
                if stats.max > gstats.max {
                    gstats.max = stats.max;
                }
                gstats.sum += stats.sum;
                gstats.count += stats.count;
            })
            .or_insert(stats);
    }
    global
}

fn process_chunk(chunk: &str) -> AHashMap<String, Stats> {
    let mut local_map: AHashMap<String, Stats> = AHashMap::new();
    for line in chunk.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some((station, measurement)) = line.split_once(';') {
            if let Ok(value) = measurement.parse::<f64>() {
                local_map
                    .entry(station.to_owned())
                    .and_modify(|stats| {
                        if value < stats.min {
                            stats.min = value;
                        }
                        if value > stats.max {
                            stats.max = value;
                        }
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
    // Open file and memory-map it.
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <measurements.txt>", args[0]);
        std::process::exit(1);
    }
    let file = File::open(&args[1])?;
    let mmap = unsafe { Mmap::map(&file)? };
    // SAFETY: Assume file is valid UTF-8.
    let content = unsafe { std::str::from_utf8_unchecked(&mmap[..]) };
    // Wrap mmap in Arc so it stays alive.
    let mmap_arc = Arc::new(&mmap);
    // Here, content is a view into mmap_arc. We use content only for computing ranges.
    let len = content.len();
    let num_workers = 8;
    let chunk_size = len / num_workers;
    let mut ranges = Vec::with_capacity(num_workers);
    let mut start = 0;
    for _ in 0..num_workers {
        let end = if start + chunk_size >= len {
            len
        } else {
            let mut pos = start + chunk_size;
            while pos < len && content.as_bytes()[pos] != b'\n' {
                pos += 1;
            }
            if pos < len { pos + 1 } else { len }
        };
        ranges.push((start, end));
        start = end;
    }
    // Process each chunk in parallel using Rayon.
    let global_map: AHashMap<String, Stats> = ranges
        .into_par_iter()
        .map(|(s, e)| {
            // Obtain a reference to the chunk via mmap_arc.
            // Note: We use the original content slice which remains valid
            // because mmap_arc lives until main ends.
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
    // Keep mmap_arc alive until here.
    drop(mmap_arc);
    Ok(())
}