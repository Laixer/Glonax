#!/bin/bash
# Copyright (c) 2021-2023 Laixer B.V.
#
# Build a Debian package for Glonax
#
# Run script from the workspace root and make
# sure the database runs on localhost.
#
# Usage: ./scripts/package.sh

set -e

if [ ! -d "./scripts" ]
then
  echo "Run script from the project directory"
  exit 1
fi

VERSION=3.0-1

# Build the project
cargo build --release

# Cleanup
rm -rf ./target/glonax_${VERSION}_amd64
rm -rf ./target/glonax_${VERSION}_amd64.deb

# Build directory tree
mkdir -p ./target/glonax_${VERSION}_amd64/DEBIAN
mkdir -p ./target/glonax_${VERSION}_amd64/etc/udev/rules.d
mkdir -p ./target/glonax_${VERSION}_amd64/etc/systemd/system
mkdir -p ./target/glonax_${VERSION}_amd64/usr/local/bin

# Copy config files
cp ./contrib/deb/control ./target/glonax_${VERSION}_amd64/DEBIAN
cp ./contrib/deb/postinst ./target/glonax_${VERSION}_amd64/DEBIAN
cp ./contrib/udev/79-glonax.rules ./target/glonax_${VERSION}_amd64/etc/udev/rules.d
cp ./contrib/systemd/glonax-ecud@.service ./target/glonax_${VERSION}_amd64/etc/systemd/system

# Copy binaries
cp ./target/release/glonax-csim ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-ecud ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-gnssd ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-input ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-netctl ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-proxyd ./target/glonax_${VERSION}_amd64/usr/local/bin

# Set permissions
chmod 755 ./target/glonax_${VERSION}_amd64/DEBIAN/postinst

# Build the package
dpkg-deb --build --root-owner-group ./target/glonax_${VERSION}_amd64

# Cleanup
rm -rf ./target/glonax_${VERSION}_amd64
