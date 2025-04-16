# rs-zip

A high-performance, open-source command-line utility designed for modern data management, rs-zip combines battle-tested LZMA/XZ compression algorithms with an innovative custom archive format to deliver unparalleled efficiency for both individual files and complex data structures. Built in Rust for memory safety and blazing-fast execution


## Features

LZMA/XZ Compression** (`.xz` files)
  - Single-file compression with padding optimization
  - Adjustable compression levels (0-9)
Custom Archive Format** (`.rsz` files)
  - Multi-file/directory compression
  - Preserves file structure and metadata
  - Recursive directory compression
Cross-platform support (Linux, macOS, Windows)
Robust error handling and validation

## Installation

### Prerequisites
- Rust toolchain (1.60+)
- Cargo package manager
- liblzma development files (Linux only)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/guap-codes/rs-zip.git
cd rs-zip

# Build release version
cargo build --release

# Install system-wide (optional)
cargo install --path .
```

### Linux Dependencies:

```bash
sudo apt-get install pkg-config liblzma-dev  # Debian/Ubuntu
sudo dnf install xz-devel                    # Fedora
```
Note: Windows users and pure-Rust builds don’t need these dependencies.

### Usage
* Basic Commands

Compress single file (XZ format):
```bash
rs-zip -i input.txt -o compressed.xz
```
Decompress XZ file:
```bash
rs-zip -d -i compressed.xz
```
Create multi-file archive (RSZ format):
```bash
rs-zip -f rsz -i file1.txt dir/ -r -o archive.rsz

rs-zip -f rsz -i test.txt test1.txt test2.txt -o combo900.rsz

```
Extract RSZ archive:
```bash
rs-zip -d -f rsz -i archive.rsz -o output_dir/
```

* Advanced Options
Flag            Description               Example
-i/--inputs     Input files/directories   -i file1.txt dir/
-o/--output     Output path               -o backup.rsz
-d/--decompress Decompress mode           -d -i archive.rsz
-l/--level      Compression level (0-9)   -l 9 (max compression)
-f/--format     File format (xz/rsz)      -f rsz
-r/--recursive  Recursive dir compression -i dir/ -r

### File Formats
XZ Mode (Single File)

   * Standard LZMA2 compression (.xz extension)

   * Automatically pads small files (<64 bytes) for better compression

   * Compatible with standard XZ utilities

RSZ Mode (Multi-file Archive)

   * Custom format with metadata header:
   
    [File Count][File1 Metadata][File1 Data][File2 Metadata][File2 Data]...
   

   * File Metadata:

      - Filename length (8 bytes LE)

      - Filename (UTF-8)

      - Original file size (8 bytes LE)

   * Uses XZ compression for archive contents

- Decompression Process

    XZ Files:

      * Remove padding using stored original size

      * Write decompressed content to output path

    RSZ Archives:

      * Read archive header

      * Recreate directory structure

      * Decompress files with original names and sizes

      * Preserve file modification times

### Troubleshooting

Common Issues:

    * "No such file or directory": Ensure output directory exists

    * Invalid compression level: Use values 0-9

    * UTF-8 filename issues: Non-UTF8 filenames are skipped in RSZ archives

    * Permission denied: Run with elevated privileges for system files

### Benchmarks
Run benchmark tests:
```bash
cargo bench --bench benchmarks
```

Benhmark analysis result:
See the bench_analysis.md file on a detailed analysis of the current benchmark results.


MIT License - See [LICENSE]|(LICENSE) for details

### Contributing

    Fork the repository

    Create feature branch (git checkout -b feature/improvement)

    Commit changes (git commit -am 'Add new feature')

    Push to branch (git push origin feature/improvement)

    Create new Pull Request

### Acknowledgements:

    xz2-rs for LZMA compression

    clap-rs for CLI parsing
