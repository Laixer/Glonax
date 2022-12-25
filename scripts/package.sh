#!/bin/bash
set -e

VERSION=1.4-1

# Build directory tree
mkdir -p ./target/glonax_${VERSION}_amd64/DEBIAN
mkdir -p ./target/glonax_${VERSION}_amd64/etc/udev/rules.d
mkdir -p ./target/glonax_${VERSION}_amd64/etc/systemd/system
mkdir -p ./target/glonax_${VERSION}_amd64/usr/local/bin

# Copy files
cp ./contrib/deb/control ./target/glonax_${VERSION}_amd64/DEBIAN
cp ./contrib/udev/79-glonax.rules ./target/glonax_${VERSION}_amd64/etc/udev/rules.d
cp ./contrib/systemd/glonax-ecud@.service ./target/glonax_${VERSION}_amd64/etc/systemd/system
cp ./contrib/systemd/glonax-execd.service ./target/glonax_${VERSION}_amd64/etc/systemd/system
cp ./contrib/systemd/glonax-inputd@.service ./target/glonax_${VERSION}_amd64/etc/systemd/system
cp ./target/release/glonax-ecud ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-execd ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-inputd ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax-netctl ./target/glonax_${VERSION}_amd64/usr/local/bin
cp ./target/release/glonax ./target/glonax_${VERSION}_amd64/usr/local/bin

# Build the package
dpkg-deb --build --root-owner-group ./target/glonax_${VERSION}_amd64
