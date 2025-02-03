use std::env;
use std::fs::File;
use std::io;
use std::sync::Arc;
use std::thread;

use ahash::AHashMap; // fast hashing
use memmap2::Mmap;

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
        // Using split_once to avoid extra allocation.
        if let Some((station, measurement)) = line.split_once(';') {
            // Parse measurement; silently skip if invalid.
            if let Ok(value) = measurement.parse::<f64>() {
                local_map
                    .entry(station.to_string())
                    .and_modify(|stats: &mut Stats| {
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
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <measurements.txt>", args[0]);
        std::process::exit(1);
    }
    let file = File::open(&args[1])?;
    let mmap = unsafe { Mmap::map(&file)? };
    // Wrap the mmap in an Arc so it lives long enough.
    let shared_mmap = Arc::new(mmap);
    // SAFETY: the file is assumed to be valid UTF-8.
    let content = unsafe { std::str::from_utf8_unchecked(&shared_mmap[..]) };
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
    let mut handles = Vec::with_capacity(num_workers);
    for (start, end) in ranges {
        let shared = Arc::clone(&shared_mmap);
        let handle = thread::spawn(move || {
            let chunk_bytes = &shared[start..end];
            // SAFETY: file is valid UTF-8.
            let chunk = unsafe { std::str::from_utf8_unchecked(chunk_bytes) };
            process_chunk(chunk)
        });
        handles.push(handle);
    }
    let mut global_map: AHashMap<String, Stats> = AHashMap::new();
    for handle in handles {
        let local_map = handle.join().expect("Thread panicked");
        global_map = merge_maps(global_map, local_map);
    }
    let mut stations: Vec<(String, Stats)> = global_map.into_iter().collect();
    stations.sort_by(|a, b| a.0.cmp(&b.0));
    for (station, stats) in stations {
        let mean = stats.sum / (stats.count as f64);
        println!("{};{:.1};{:.1};{:.1}", station, stats.min, mean, stats.max);
    }
    Ok(())
}