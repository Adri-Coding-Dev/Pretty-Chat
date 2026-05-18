#!/usr/bin/env bash
set -euo pipefail

echo "==> Compilando pchat en modo release..."
cargo build --release
echo "==> Binario generado: target/release/pchat"
