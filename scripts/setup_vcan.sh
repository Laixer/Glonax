#!/bin/bash
# Copyright (c) 2021-2023 Laixer B.V.
#
# Build a Debian package for Glonax
#
# Run script from the workspace root and make
# sure the database runs on localhost.
#
# Usage: ./scripts/setup_vcan.sh

set -e

if [ "$(id -u)" != "0" ]
then
  echo "Please run as root"
  exit 1
fi

modprobe vcan

ip link add dev vcan0 type vcan
ip link set up vcan0

echo "vcan0 is up"

ip link add dev vcan1 type vcan
ip link set up vcan1

echo "vcan1 is up"
