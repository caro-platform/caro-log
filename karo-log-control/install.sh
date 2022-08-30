#!/bin/bash

self_dir=$(dirname $(realpath "${0}"))

echo -e "\e[32mInstalling log control\e[0m"

pushd ${self_dir} > /dev/null
cargo build --release

sudo mkdir -p /etc/karo/services/
sudo cp -f karo.log.control.service /etc/karo/services/

sudo cp -f ../target/release/karo-log-control /usr/bin/
popd > /dev/null
