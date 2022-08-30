#!/bin/bash

self_dir=$(dirname $(realpath "${0}"))

echo "Install script dir: ${self_dir}"

pushd ${self_dir}/karo-logger/ > /dev/null
bash ./install.sh
popd > /dev/null

pushd ${self_dir}/karo-log-control/ > /dev/null
bash ./install.sh
popd > /dev/null
