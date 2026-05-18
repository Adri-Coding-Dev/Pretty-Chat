#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "==> Compilando e instalando pchat..."
cargo build --release
cp target/release/pchat "$INSTALL_DIR/pchat"
echo "==> Instalado en $INSTALL_DIR/pchat"
echo "==> Asegurate de que $INSTALL_DIR este en tu PATH"
