#!/bin/bash

# JSON Validator HTTP Server Build Script
# This script builds the HTTP JSON validator server for different platforms

set -e

# Configuration
PROJECT_NAME="json-validator-http"
VERSION=$(grep '^version' Cargo.toml | head -1 | awk -F'"' '{print $2}')
BUILD_DIR="target/release"
PLATFORMS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create build directory
mkdir -p dist

# Build for each platform
for platform in "${PLATFORMS[@]}"; do
    log_info "Building for $platform"
    
    # Set target specific variables
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="$PROJECT_NAME.exe"
        archive_name="$PROJECT_NAME-$VERSION-$platform.zip"
    else
        binary_name="$PROJECT_NAME"
        archive_name="$PROJECT_NAME-$VERSION-$platform.tar.gz"
    fi
    
    # Build the binary
    if [[ "$platform" == "aarch64-unknown-linux-gnu" ]]; then
        # Cross-compilation requires additional setup
        log_warn "Cross-compilation for $platform may require additional toolchain setup"
        cargo build --release --target "$platform"
    else
        cargo build --release --target "$platform"
    fi
    
    # Create distribution directory
    dist_dir="dist/$platform"
    mkdir -p "$dist_dir"
    
    # Copy binary
    cp "$BUILD_DIR/$platform/$binary_name" "$dist_dir/"
    
    # Copy configuration files
    cp -r config "$dist_dir/"
    cp README.md "$dist_dir/"
    cp Dockerfile "$dist_dir/"
    
    # Create archive
    cd dist
    if [[ "$platform" == *"windows"* ]]; then
        zip -r "$archive_name" "$platform/"
    else
        tar -czf "$archive_name" "$platform/"
    fi
    cd ..
    
    log_info "Created $archive_name"
done

# Create Docker image
log_info "Building Docker image"
docker build -t "$PROJECT_NAME:$VERSION" .
docker tag "$PROJECT_NAME:$VERSION" "$PROJECT_NAME:latest"

# Create version info
cat > "dist/VERSION" << EOF
Version: $VERSION
Build Date: $(date)
Build Platform: $(uname -m)
Git Commit: $(git rev-parse HEAD)
EOF

# Create checksums
log_info "Creating checksums"
cd dist
for file in *.tar.gz *.zip; do
    if [[ -f "$file" ]]; then
        sha256sum "$file" >> "checksums.txt"
    fi
done
cd ..

log_info "Build completed successfully!"
log_info "Distribution files are available in the 'dist' directory"
log_info "Docker image: $PROJECT_NAME:$VERSION"

# Print build summary
echo ""
echo "=== Build Summary ==="
echo "Version: $VERSION"
echo "Platforms: ${#PLATFORMS[@]}"
echo "Docker Image: $PROJECT_NAME:$VERSION"
echo "Distribution Directory: dist/"
echo ""

# List all distribution files
log_info "Distribution files:"
ls -la dist/