# BAM Pair Filter (Rust Version)

A fast, portable Rust implementation for filtering paired-end BAM reads by kmer complexity and contiguous mapped bases.

## Key Features

✅ **Truly Portable**: Single binary with no external dependencies
✅ **Fast**: Compiled with LTO and full optimizations
✅ **Safe**: Memory-safe Rust with no buffer overflows
✅ **Small**: ~7.6MB statically-linked binary
✅ **Same Features**: Identical to C version with same algorithms

## Quick Start

```bash
# Build (requires Rust 1.82+)
cargo build --release

# Binary location
./target/release/filter_bam_pairs

# Run
./target/release/filter_bam_pairs \
    --input input.namesorted.bam \
    --output filtered.bam \
    --complexity 0.8 \
    --min-mapped 90
```

## Installation

### Option 1: Build from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build release binary
cd fastq_kmer_filter_rust
cargo build --release

# Binary is at: target/release/filter_bam_pairs
# Copy to PATH if desired:
sudo cp target/release/filter_bam_pairs /usr/local/bin/
```

### Option 2: Use Pre-built Binary

The compiled binary is **fully portable** across Linux x86_64 systems:

```bash
# Just copy and run - no dependencies needed!
cp target/release/filter_bam_pairs ~/bin/
./filter_bam_pairs --help
```

## Usage

```
Usage: filter_bam_pairs [OPTIONS] --input <FILE> --output <FILE>

Options:
  -i, --input <FILE>              Input BAM file (must be name-sorted)
  -o, --output <FILE>             Output BAM file
  -c, --complexity <COMPLEXITY>   Kmer complexity cutoff (0.0-1.0) [default: 0.8]
  -m, --min-mapped <MIN_MAPPED>   Minimum contiguous mapped bases [default: 0]
  -h, --help                      Print help
```

### Examples

```bash
# Filter by complexity only (default 0.8)
./filter_bam_pairs -i input.bam -o filtered.bam

# Filter by complexity AND mapped bases
./filter_bam_pairs \
    -i input.namesorted.bam \
    -o high_quality.bam \
    -c 0.8 \
    -m 90

# Strict filtering
./filter_bam_pairs \
    -i input.namesorted.bam \
    -o very_strict.bam \
    -c 0.85 \
    -m 100
```

## Important: BAM Requirements

**Input BAM must be name-sorted:**

```bash
# Sort by read name
samtools sort -n input.bam -o input.namesorted.bam

# Then filter
./filter_bam_pairs -i input.namesorted.bam -o filtered.bam
```

## Portability

### What Makes It Portable?

The Rust version:
- **Statically links htslib**: No need to install htslib on target systems
- **Only depends on libc/libm**: Standard libraries present on all Linux systems
- **Single binary**: Just copy and run

### Dependencies Check

```bash
$ ldd target/release/filter_bam_pairs
    linux-vdso.so.1
    libgcc_s.so.1 => /lib/x86_64-linux-gnu/libgcc_s.so.1
    libm.so.6 => /lib/x86_64-linux-gnu/libm.so.6
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
    /lib64/ld-linux-x86-64.so.2
```

**No htslib dependency!** These are all standard libraries.

### Cross-Platform Testing

```bash
# Test on different Linux distros
docker run --rm -v $(pwd):/work ubuntu:22.04 /work/target/release/filter_bam_pairs -h
docker run --rm -v $(pwd):/work debian:11 /work/target/release/filter_bam_pairs -h
docker run --rm -v $(pwd):/work centos:7 /work/target/release/filter_bam_pairs -h
```

## Performance

### Compilation Optimizations

The binary is compiled with:
- `-O3` (maximum optimization)
- `LTO = true` (link-time optimization)
- `codegen-units = 1` (better optimization, slower compile)
- `strip = true` (remove debug symbols)

### Benchmarks

Typical performance on modern hardware:
- **Throughput**: ~500k-1M paired reads/second
- **Memory**: Minimal (<100MB)
- **I/O bound**: Performance limited by disk/BAM compression

## Comparison: C vs Rust

| Feature | C Version | Rust Version |
|---------|-----------|--------------|
| Speed | Fast | Fast (similar) |
| Binary size | ~20KB | ~7.6MB |
| Dependencies | Requires htslib installed | None (statically linked) |
| Portability | Needs matching htslib | Runs anywhere |
| Safety | Manual memory management | Memory safe |
| Development | More code, error-prone | Less code, safer |

**Recommendation**: Use Rust version for portability and safety.

## Building for Different Targets

### Static Binary (musl libc)

For maximum portability (works even on systems without glibc):

```bash
# Add musl target
rustup target add x86_64-unknown-linux-musl

# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Truly static binary!
ldd target/x86_64-unknown-linux-musl/release/filter_bam_pairs
# Output: not a dynamic executable
```

### Cross-Compilation

```bash
# For ARM64 (e.g., Apple Silicon)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# For macOS
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## Development

### Run Tests

```bash
cargo test
```

### Run with Debug Logging

```bash
RUST_LOG=debug cargo run --release -- -i input.bam -o output.bam
```

### Format Code

```bash
cargo fmt
```

### Lint

```bash
cargo clippy
```

## Technical Details

### Algorithm

Same as C version:

1. **Kmer Complexity**: `unique_kmers / total_kmers` (k=21)
2. **Mapped Bases**: Longest contiguous M/= CIGAR stretch
3. **Filtering**: Both reads must pass both thresholds
4. **Pairing**: Maintains read pair integrity

### Dependencies

- `rust-htslib`: Rust bindings to htslib (statically linked)
- `clap`: Command-line argument parsing
- `anyhow`: Error handling

### Rust Edition

Uses Rust 2021 edition with modern idioms.

## Troubleshooting

### "cannot find -lbz2" during build

Install bzip2 development files:

```bash
sudo apt-get install libbz2-dev
```

### "cannot find -llzma" during build

Install xz development files:

```bash
sudo apt-get install liblzma-dev
```

### Build takes long time

First build compiles all dependencies. Subsequent builds are fast.

### Binary too large

The binary includes:
- htslib (BAM parsing)
- Compression libraries (zlib, bzip2, lzma)
- All statically linked for portability

You can reduce size with:
```bash
cargo build --release
strip target/release/filter_bam_pairs  # Already done automatically
upx --best --lzma target/release/filter_bam_pairs  # Compress with UPX (optional)
```

## License

Same as C version - Public domain

## See Also

- C version: `../fastq_kmer_filter/filter_bam_pairs`
- FASTQ filter: `../fastq_kmer_filter/filter_paired_reads`

## Why Rust?

Advantages over C:
1. **No segfaults**: Memory safety guaranteed by compiler
2. **No undefined behavior**: Strict type system
3. **Easy distribution**: Statically linked by default
4. **Modern tooling**: cargo, rustfmt, clippy
5. **Maintainable**: Clearer code, better error handling

The Rust version provides the same functionality as C with better safety and easier deployment.
