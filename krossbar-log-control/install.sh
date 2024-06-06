#!/bin/bash

self_dir=$(dirname $(realpath "${0}"))

echo -e "\e[32mInstalling log control\e[0m"

pushd ${self_dir} > /dev/null
cargo build --release

sudo mkdir -p /etc/krossbar/services/
sudo cp -f krossbar.log.control.service /etc/krossbar/services/

sudo cp -f ../target/release/krossbar-log-control /usr/bin/
popd > /dev/null
