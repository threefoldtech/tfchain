#!/bin/bash
set -e

echo "Building tfwallet (optimized)..."

# Build with maximum optimization flags
# -s: omit symbol table
# -w: omit DWARF debugging info
# -trimpath: remove file system paths from binary
CGO_ENABLED=0 go build \
    -ldflags="-s -w" \
    -trimpath \
    -o /tmp/tfwallet \
    ./cmd/tfwallet

SIZE=$(ls -lh /tmp/tfwallet | awk '{print $5}')
echo "Binary size: $SIZE"

# Compress with UPX if available (Linux only - macOS has issues)
if [[ "$OSTYPE" != "darwin"* ]] && command -v upx &> /dev/null; then
    echo "Compressing with UPX (maximum compression)..."
    upx --best --lzma /tmp/tfwallet
    SIZE=$(ls -lh /tmp/tfwallet | awk '{print $5}')
    echo "Compressed size: $SIZE"
fi

echo ""
echo "Build complete: /tmp/tfwallet"
