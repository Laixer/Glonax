#!/bin/bash

# Build directory tree
mkdir -p ./target/glonax_1.0-1_amd64/DEBIAN
mkdir -p ./target/glonax_1.0-1_amd64/etc/systemd/system
mkdir -p ./target/glonax_1.0-1_amd64/etc/glonax
mkdir -p ./target/glonax_1.0-1_amd64/usr/local/bin
# Copy files
cp ./contrib/deb/control ./target/glonax_1.0-1_amd64/DEBIAN
cp ./contrib/glonax.service ./target/glonax_1.0-1_amd64/etc/systemd/system
cp ./contrib/etc/glonaxd.toml ./target/glonax_1.0-1_amd64/etc/glonax
cp ./target/release/glonaxd ./target/release/glonax-netctl ./target/glonax_1.0-1_amd64/usr/local/bin
# Build the package
dpkg-deb --build --root-owner-group ./target/glonax_1.0-1_amd64
