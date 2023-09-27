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

VERSION=3.0-2
ARCH=$(uname -m)

case $ARCH in
    x86_64)
        ARCH_NAME="amd64"
        ;;
    aarch64)
        ARCH_NAME="arm64"
        ;;
    *)
        ARCH_NAME="any"
        ;;
esac

PACKAGE_DIR="./target/glonax_${VERSION}_$ARCH_NAME"
PACKAGE_NAME="glonax_${VERSION}_$ARCH_NAME.deb"

# Build the project
cargo build --release

# Cleanup
rm -rf $PACKAGE_DIR
rm -rf ./target/glonax_${VERSION}_$ARCH_NAME.deb

# Build directory tree
mkdir -p $PACKAGE_DIR/DEBIAN
mkdir -p $PACKAGE_DIR/etc/udev/rules.d
mkdir -p $PACKAGE_DIR/etc/systemd/system
mkdir -p $PACKAGE_DIR/usr/local/bin
mkdir -p $PACKAGE_DIR/usr/local/share/glonax

# Copy config files
cp ./contrib/deb/* $PACKAGE_DIR/DEBIAN
cp ./contrib/udev/* $PACKAGE_DIR/etc/udev/rules.d
cp ./contrib/systemd/* $PACKAGE_DIR/etc/systemd/system
cp -r ./contrib/etc/* $PACKAGE_DIR/etc
cp -r ./contrib/share/* $PACKAGE_DIR/usr/local/share/glonax

# Copy binaries
cp ./target/release/glonax-csim $PACKAGE_DIR/usr/local/bin
cp ./target/release/glonax-gnssd $PACKAGE_DIR/usr/local/bin
cp ./target/release/glonax-input $PACKAGE_DIR/usr/local/bin
cp ./target/release/glonax-netctl $PACKAGE_DIR/usr/local/bin
cp ./target/release/glonax-diag $PACKAGE_DIR/usr/local/bin
cp ./target/release/glonax-agent $PACKAGE_DIR/usr/local/bin
cp ./target/release/glonax-proxyd $PACKAGE_DIR/usr/local/bin

# Set package architecture
sed -i "s/{ARCH}/$ARCH_NAME/" $PACKAGE_DIR/DEBIAN/control

# Set permissions
chmod 755 $PACKAGE_DIR/DEBIAN/postinst

# Build the package
dpkg-deb --build --root-owner-group $PACKAGE_DIR

# Cleanup
rm -rf $PACKAGE_DIR
