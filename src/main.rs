use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::thread;

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

fn merge_maps(
    mut global: HashMap<&'static str, Stats>,
    local: HashMap<&'static str, Stats>,
) -> HashMap<&'static str, Stats> {
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

fn process_chunk(chunk: &'static str) -> HashMap<&'static str, Stats> {
    let mut local_map = HashMap::new();
    for line in chunk.lines() {
        if let Some((station, measurement)) = line.split_once(';') {
            let value: f64 = match measurement.parse() {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("Invalid measurement '{}'", measurement);
                    continue;
                }
            };
            local_map.entry(station)
                .and_modify(|stats: &mut Stats| {
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
        } else if !line.is_empty() {
            eprintln!("Skipping invalid line: {}", line);
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
    // Map file into memory.
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    // SAFETY: measurements.txt is assumed to be valid UTF-8.
    let content = unsafe { std::str::from_utf8_unchecked(&mmap[..]) };
    // Leak content to obtain a 'static lifetime.
    let leaked: &'static str = Box::leak(content.to_owned().into_boxed_str());
    let len = leaked.len();
    let num_workers = 8;
    let chunk_size = len / num_workers;
    let mut ranges = Vec::with_capacity(num_workers);
    let mut start = 0;
    for i in 0..num_workers {
        let end = if i == num_workers - 1 {
            len
        } else {
            let mut pos = start + chunk_size;
            while pos < len && leaked.as_bytes()[pos] != b'\n' {
                pos += 1;
            }
            if pos < len { pos + 1 } else { len }
        };
        ranges.push((start, end));
        start = end;
    }
    let mut handles = Vec::with_capacity(num_workers);
    for (start, end) in ranges {
        let chunk: &'static str = &leaked[start..end];
        let handle = thread::spawn(move || process_chunk(chunk));
        handles.push(handle);
    }
    let mut global_map: HashMap<&'static str, Stats> = HashMap::new();
    for handle in handles {
        let local_map = handle.join().expect("Thread panicked");
        global_map = merge_maps(global_map, local_map);
    }
    let mut stations: Vec<(&'static str, Stats)> = global_map.into_iter().collect();
    stations.sort_by(|a, b| a.0.cmp(b.0));
    for (station, stats) in stations {
        let mean = stats.sum / (stats.count as f64);
        println!("{};{:.1};{:.1};{:.1}", station, stats.min, mean, stats.max);
    }
    Ok(())
}