use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::thread;

use crossbeam_channel::unbounded;

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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <measurements.txt>", args[0]);
        std::process::exit(1);
    }

    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);

    // Create a channel to send lines for processing
    let (sender, receiver) = unbounded::<String>();

    // Use 8 worker threads (machine has 8 cores)
    let num_workers = 8;
    let mut handles = Vec::with_capacity(num_workers);

    for _ in 0..num_workers {
        let rcv: crossbeam_channel::Receiver<String> = receiver.clone();
        let handle = thread::spawn(move || {
            let mut local_map: HashMap<String, Stats> = HashMap::new();
            for line in rcv.iter() {
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
            local_map
        });
        handles.push(handle);
    }

    // Read file lines and send them to worker threads
    for line in reader.lines() {
        let line = line?;
        sender.send(line).expect("Failed to send line");
    }
    // Drop sender to signal workers no more messages
    drop(sender);

    // Merge results from workers
    let mut global_map: HashMap<String, Stats> = HashMap::new();
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
