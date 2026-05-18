#!/usr/bin/env bash
set -euo pipefail

CONFIG_DIR="$HOME/.config/pchat"
THEMES_DIR="$CONFIG_DIR/themes"
mkdir -p "$CONFIG_DIR" "$THEMES_DIR"

# Copiar temas por defecto
cp themes/*.toml "$THEMES_DIR/"
echo "==> Temas copiados a $THEMES_DIR"

# Generar config.toml si no existe
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
  cat > "$CONFIG_DIR/config.toml" <<EOF
# Configuración de pchat
theme = "tokyo-night"
default_backend = "internal"
compact = false
EOF
  echo "==> Archivo de configuración creado en $CONFIG_DIR/config.toml"
fi
