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

Benchmark
---------
| Commit   | Real     | User     | Sys     |
|----------|----------|----------|---------|
| f017bb2  | 0:16.61  | 1:24.90  | 0:18.49 |
| 5dd67c6  | 0:16.48  | 1:24.31  | 0:19.90 |
| 243c217  | 0:16.60  | 1:24.64  | 0:19.90 |
| a4418a8  | 0:15.92  | 1:22.03  | 0:19.93 |
| 7ca4527  | 0:16.58  | 1:25.57  | 0:19.18 |
| 63275ec  | 0:16.26  | 1:25.38  | 0:19.06 |
| 85165cf  | 0:19.24  | 1:53.74  | 0:17.12 |
| 4440c50  | 0:19.04  | 1:51.87  | 0:18.49 |
| 35193b0  | 0:22.79  | 2:23.88  | 0:17.85 |
| a351507  | 0:21.81  | 2:25.09  | 0:14.08 |
| ff8be76  | 0:43.07  | 0:02.01  | 0:26.66 |
| 4d67c28  | 0:24.14  | 2:37.38  | 0:17.24 |
| 65a67c4  | 0:23.49  | 2:38.91  | 0:13.76 |
| b062f98  | 2:16.00  | 3:34.21  | 1:04.69 |
| 1a9069f  | 10:19.74 | 69:47.61 | 5:12.63 |
| 2833dca  | 4:32.38  | 4:17.82  | 0:13.99 |

Real Times
----------
| Commit   | Real (sec) | Graph                                                      |
|----------|------------|------------------------------------------------------------|
| f017bb2  | 16.61      | #                                                          |
| 5dd67c6  | 16.48      | #                                                          |
| 243c217  | 16.60      | #                                                          |
| a4418a8  | 15.92      | #                                                          |
| 7ca4527  | 16.58      | #                                                          |
| 63275ec  | 16.26      | #                                                          |
| 85165cf  | 19.24      | #                                                          |
| 4440c50  | 19.04      | #                                                          |
| 35193b0  | 22.79      | ##                                                         |
| a351507  | 21.81      | ##                                                         |
| ff8be76  | 43.07      | ####                                                       |
| 4d67c28  | 24.14      | ##                                                         |
| 65a67c4  | 23.49      | ##                                                         |
| b062f98  | 136.00     | #############                                              |
| 1a9069f  | 619.74     | ############################################################ |
| 2833dca  | 272.38     | ##########################                                 |

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
