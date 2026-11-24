#!/usr/bin/env bash
set -euo pipefail

echo "Building HeirSwitch Soroban contract WASM..."
cargo build --manifest-path contracts/heirswitch/Cargo.toml --target wasm32v1-none --release
echo "WASM ready at target/wasm32v1-none/release/heirswitch.wasm"
