#!/bin/bash
set -o nounset -o errexit -o pipefail
source .private/env
# cleanup
rm -rf out
rm -rf dist
rm -f game.zip

cargo build --release --target wasm32-unknown-unknown --no-default-features
wasm-bindgen --no-typescript --out-name bevy_game --out-dir dist --target web target/wasm32-unknown-unknown/release/wack-a-weed.wasm
cp wasm/* dist/
cp -r assets dist/
zip -r game.zip dist/*
