#!/usr/bin/env bash
set -euo pipefail
cargo build --release
echo "Deploy artifact: target/release/api"
