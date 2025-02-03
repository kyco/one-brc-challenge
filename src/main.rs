use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;

struct Stats {
    min: f64,
    max: f64,
    sum: f64,
    count: u64,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <measurements.txt>", args[0]);
        std::process::exit(1);
    }

    let file = File::open(&args[1])?;
    let reader = BufReader::new(file);
    let mut data: HashMap<String, Stats> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
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

        data.entry(station).and_modify(|stats| {
            if value < stats.min { stats.min = value; }
            if value > stats.max { stats.max = value; }
            stats.sum += value;
            stats.count += 1;
        }).or_insert(Stats {
            min: value,
            max: value,
            sum: value,
            count: 1,
        });
    }

    let mut stations: Vec<(String, Stats)> = data.into_iter().collect();
    stations.sort_by(|a, b| a.0.cmp(&b.0));

    for (station, stats) in stations {
        let mean = stats.sum / (stats.count as f64);
        println!("{};{:.1};{:.1};{:.1}",
                 station, stats.min, mean, stats.max);
    }

    Ok(())
}