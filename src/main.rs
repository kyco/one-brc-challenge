use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::str;
use std::thread;

use memmap2::Mmap;

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

fn merge_maps(
    mut global: HashMap<String, Stats>,
    local: HashMap<String, Stats>,
) -> HashMap<String, Stats> {
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

fn process_chunk(chunk: &str) -> HashMap<String, Stats> {
    let mut local_map = HashMap::new();
    for line in chunk.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 2 {
            eprintln!("Skipping invalid line: {}", line);
            continue;
        }
        let station = parts[0].to_string();
        let value: f64 = match parts[1].parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Invalid measurement '{}'", parts[1]);
                continue;
            }
        };

        local_map
            .entry(station)
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
    let content = match str::from_utf8(&mmap) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error parsing file as UTF-8: {}", e);
            std::process::exit(1);
        }
    };

    let num_workers = 8;
    let len = content.len();
    let chunk_size = len / num_workers;
    let mut ranges = Vec::with_capacity(num_workers);
    let mut start = 0;

    for i in 0..num_workers {
        let mut end = if i == num_workers - 1 {
            len
        } else {
            // tentative end, then move to next newline
            let mut pos = start + chunk_size;
            while pos < len && content.as_bytes()[pos] != b'\n' {
                pos += 1;
            }
            if pos < len {
                pos + 1
            } else {
                len
            }
        };
        ranges.push((start, end));
        start = end;
    }

    let mut handles = Vec::with_capacity(num_workers);
    for (start, end) in ranges {
        // Create a slice for the chunk.
        let chunk = &content[start..end];
        let chunk = chunk.to_owned(); // Own the chunk for the thread.
        let handle = thread::spawn(move || process_chunk(&chunk));
        handles.push(handle);
    }

    let mut global_map = HashMap::new();
    for handle in handles {
        let local_map = handle.join().expect("Thread panicked");
        global_map = merge_maps(global_map, local_map);
    }

    let mut stations: Vec<(String, Stats)> = global_map.into_iter().collect();
    stations.sort_by(|a, b| a.0.cmp(&b.0));

    for (station, stats) in stations {
        let mean = stats.sum / (stats.count as f64);
        println!("{};{:.1};{:.1};{:.1}",
                 station, stats.min, mean, stats.max);
    }
    Ok(())
}