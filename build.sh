#!/bin/bash
set -e

echo "=========================================="
echo "Building Rust BAM Pair Filter"
echo "=========================================="
echo ""

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust not installed"
    echo "Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo ""

# Build static release binary (fully portable)
echo "Building static release binary (optimized, portable)..."
cargo build --release --target x86_64-unknown-linux-musl

if [ ! -f target/x86_64-unknown-linux-musl/release/filter_bam_pairs ]; then
    echo "Error: Build failed"
    exit 1
fi

# Copy to standard location for convenience
cp target/x86_64-unknown-linux-musl/release/filter_bam_pairs target/release/filter_bam_pairs 2>/dev/null || true

echo ""
echo "=========================================="
echo "Build successful!"
echo "=========================================="
echo ""
echo "Binary location: target/release/filter_bam_pairs"
echo "Binary size: $(ls -lh target/release/filter_bam_pairs | awk '{print $5}')"
echo ""

# Test binary
echo "Testing binary..."
./target/release/filter_bam_pairs --help > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✓ Binary works correctly"
else
    echo "✗ Binary test failed"
    exit 1
fi

# Show dependencies
echo ""
echo "Dependencies:"
ldd target/release/filter_bam_pairs 2>&1 | grep -v "not a dynamic" | head -10

echo ""
echo "=========================================="
echo "Usage:"
echo "=========================================="
echo "./target/release/filter_bam_pairs \\"
echo "    --input input.namesorted.bam \\"
echo "    --output filtered.bam \\"
echo "    --complexity 0.8 \\"
echo "    --min-mapped 90"
echo ""

# Offer to copy to bin
read -p "Copy to /usr/local/bin? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    sudo cp target/release/filter_bam_pairs /usr/local/bin/
    echo "✓ Installed to /usr/local/bin/filter_bam_pairs"
fi
