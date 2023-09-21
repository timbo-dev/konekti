#!/usr/bin/env sh

set -eu

getPackagesMetadataCommand() {
    echo $(cargo metadata --format-version 1)
}

filterPackagesByName() {
    echo $(jq -r '.packages[] | select(.source == null) | .name')
}

getPackagesByName() {
    echo $(getPackagesMetadataCommand | filterPackagesByName)
}

packages=$(getPackagesByName)

for package in $packages; do
    cargo build --release -p $package --target wasm32-wasi
done
