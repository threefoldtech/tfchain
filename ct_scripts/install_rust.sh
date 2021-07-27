#!/bin/bash

set -ex

curl https://sh.rustup.rs -sSf | sh -s -- -y
export PATH="$PATH:$HOME/.cargo/bin"
rustup install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup default stable
# cargo build "--release"


