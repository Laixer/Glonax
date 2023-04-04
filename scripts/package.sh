#!/bin/bash
set -e

VERSION=3.0-1

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
cp ./contrib/systemd/glonax-inputd.service ./target/glonax_${VERSION}_amd64/etc/systemd/system

# Copy binaries
cp ./target/release/glonax-ecud ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-inputd ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-netctl ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-signetd ./target/glonax_${VERSION}_amd64/usr/local/bin

# Set permissions
chmod 755 ./target/glonax_${VERSION}_amd64/DEBIAN/postinst

# Build the package
dpkg-deb --build --root-owner-group ./target/glonax_${VERSION}_amd64

# Cleanup
rm -rf ./target/glonax_${VERSION}_amd64
