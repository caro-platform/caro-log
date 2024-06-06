#!/bin/bash

self_dir=$(dirname $(realpath "${0}"))

echo -e "\e[32mInstalling logger\e[0m"

pushd ${self_dir} > /dev/null
cargo build --release

sudo cp -f systemd/krossbar.logger.service /etc/systemd/system/

sudo mkdir -p /etc/krossbar/services/
sudo cp -f krossbar.logger.service /etc/krossbar/services/

sudo cp -f ../target/release/krossbar-logger /usr/bin/
popd > /dev/null
