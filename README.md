# gc-gcm

[![](https://docs.rs/gc-gcm/badge.svg)](https://docs.rs/gc-gcm)


A Rust library and CLI for working with GCM/ISO files (raw bit-for-bit disk images) for the Nintendo GameCube.

## Install CLI

```
cargo install gc-gcm --features=bin
```

```
~ ❯❯❯ gcm --help
gc-gcm 0.8.0

USAGE:
    gcm <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    explain    Give a brief explanation about the GCM format
    extract    Extract a GCM ISO to a given folder
    help       Prints this message or the help of the given subcommand(s)
    tree       List the file tree of the given GCM ISO
```

## Benchmarks

Benchmarking extracting all files from ISOs across various systems against [wit](https://wit.wiimm.de/):

### System 1

(Linux 5.9, 16 GB, AMD Ryzen 9 4900HS)

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `wit x melee.iso ./extracted` | 727.1 ± 11.1 | 716.1 | 769.0 | 2.20 ± 0.05 |
| `gcm extract melee.iso ./extracted` | 331.1 ± 6.2 | 321.9 | 345.9 | 1.00 |

### System 2

(Windows 10, Ryzen 7 3700X, on SSD)

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `wit x GALE01.iso ./extracted` | 8.551 ± 2.532 | 6.942 | 15.378 | 2.09 ± 0.83 |
| `gcm extract GALE01.iso ./extracted` | 4.092 ± 1.082 | 3.056 | 5.732 | 1.00 |


(Windows 10, Ryzen 7 3700X, on HDD, in single thread mode)

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `wit x GALE01.iso ./extracted` | 10.452 ± 1.812 | 8.898 | 13.982 | 1.00 |
| `gcm extract --single-thread GALE01.iso ./extracted` | 11.810 ± 1.838 | 9.372 | 14.135 | 1.13 ± 0.26 |


### System 3

(Windows 10, on HDD, multithreaded mode)

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `wit x melee.iso ./extracted` | 109.495 ± 6.301 | 105.039 | 113.950 | 1.20 ± 0.25 |
| `gcm extract melee.iso ./extracted` | 91.584 ± 18.295 | 78.647 | 104.520 | 1.00 |


**Note:** Multithreaded mode will harm performance if the ISO is on some form of physical media, such a hard drive.
