#!/bin/bash

# Create deployment package script
# This script builds and creates a tar.gz package matching the deployed server structure

set -e

PACKAGE_NAME="pkg"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
ARCHIVE_NAME="${PACKAGE_NAME}_${TIMESTAMP}.tar.gz"

echo "Creating deployment package: ${ARCHIVE_NAME}"

# Build server binary in release mode
echo "🔨 Building server binary (release mode)..."
cd server
cargo build --release
cd ..
echo "✅ Server binary built"

# Build frontend in release mode
echo "🔨 Building frontend (release mode)..."
cd frontend
trunk build --release
cd ..
echo "✅ Frontend built"

# Create temporary directory structure
TEMP_DIR=$(mktemp -d)
PACKAGE_DIR="${TEMP_DIR}/${PACKAGE_NAME}"

mkdir -p "${PACKAGE_DIR}"

# Copy assets directory
if [ -d "server/assets" ]; then
    cp -r "server/assets" "${PACKAGE_DIR}/"
    echo "✓ Copied assets/"
else
    echo "⚠ Warning: server/assets directory not found"
fi

# Copy dist directory (frontend build)
if [ -d "frontend/dist" ]; then
    cp -r "frontend/dist" "${PACKAGE_DIR}/"
    echo "✓ Copied dist/"
else
    echo "⚠ Warning: frontend/dist directory not found"
fi

# Copy binary (built with musl target)
if [ -f "target/release/gggame" ]; then
    cp "target/release/gggame" "${PACKAGE_DIR}/gggame"
    echo "✓ Copied binary as gggame"
else
    echo "❌ Error: server binary not found at target/release/gggame"
    exit 1
fi

# Create logs directory
mkdir -p "${PACKAGE_DIR}/logs"
echo "✓ Created logs/ directory"

# Copy www directory
if [ -d "server/www" ]; then
    cp -r "server/www" "${PACKAGE_DIR}/"
    echo "✓ Copied www/"
else
    echo "⚠ Warning: server/www directory not found"
fi

# Create the tar.gz archive
cd "${TEMP_DIR}"
tar -czf "${ARCHIVE_NAME}" "${PACKAGE_NAME}"
mv "${ARCHIVE_NAME}" "${OLDPWD}/"
cd "${OLDPWD}/"

# Cleanup
rm -rf "${TEMP_DIR}"
pwd
echo ""
echo "✅ Package created successfully: ${ARCHIVE_NAME}"
echo "📁 Package contents:"
tar -tzf "${ARCHIVE_NAME}" | head -20
if [ $(tar -tzf "${ARCHIVE_NAME}" | wc -l) -gt 20 ]; then
    echo "   ... and $(( $(tar -tzf "${ARCHIVE_NAME}" | wc -l) - 20 )) more files"
fi
