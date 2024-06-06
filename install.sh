#!/bin/bash

self_dir=$(dirname $(realpath "${0}"))

echo "Install script dir: ${self_dir}"

pushd ${self_dir}/krossbar-logger/ > /dev/null
bash ./install.sh
popd > /dev/null

pushd ${self_dir}/krossbar-log-control/ > /dev/null
bash ./install.sh
popd > /dev/null
