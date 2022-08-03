#!/bin/bash

self_dir=$(dirname $(realpath "${0}"))

echo -e "\e[32mInstalling logger\e[0m"

pushd ${self_dir} > /dev/null
cargo build --release

sudo cp -f systemd/karo.logger.service /etc/systemd/system/

sudo mkdir -p /etc/karo/services/
sudo cp -f karo.logger.service /etc/karo/services/

sudo cp -f ../target/release/karo-logger /usr/bin/
popd > /dev/null
