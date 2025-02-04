one-brc-o3-mini
================

Overview
--------
one-brc-o3-mini is a fast, multithreaded Rust tool that processes a large
file of station measurements. It uses memory mapping and parallelism to
efficiently compute statistics (min, mean, max) for each station.

Features
--------
- Uses memmap2 for fast file mapping.
- Processes data in parallel with Rayon.
- Efficient string parsing with memchr and lexical-core.
- Aggregates stats (min, mean, max) for each measurement.

Build and Run Instructions
--------------------------
1. Install Rust from https://rust-lang.org if not already installed.
2. Open a terminal in the repository directory.
3. Run the build and execute script with the binary name:

   ./run.sh one-brc-o3-mini

   This will:
     - Build the project in release mode.
     - Execute the binary on measurements.txt, printing timing info.

Usage
-----
The tool expects a file with records in the following format:

   StationName;measurement_value

Each line is processed and the statistics (min, mean, max) for each station
are printed. If there are many stations, only the first 10 and last 10
are shown, with a message indicating omitted records in the middle.

License
-------
This project is open source and available under the MIT License.

Contributions
-------------
Contributions and suggestions are welcome. Please submit pull requests or
open issues on GitHub.

Additional Information
----------------------
For more details on the dependencies and version locking, see Cargo.toml and
Cargo.lock.

Enjoy using one-brc-o3-mini!
