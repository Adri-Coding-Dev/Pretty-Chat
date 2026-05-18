#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

CONFIG_DIR="$HOME/.config/pchat"
THEMES_DIR="$CONFIG_DIR/themes"
LOCAL_THEMES="./themes"

 # Título ASCII Art
 echo -e "${CYAN}╔══════════════════════════════════════════════════════╗${NC}"
 echo -e "${CYAN}║${NC}  ${MAGENTA}██████╗  ██████╗██╗  ██╗ █████╗ ████████╗${NC}           ${CYAN}║${NC}"
 echo -e "${CYAN}║${NC}  ${MAGENTA}██╔══██╗██╔════╝██║  ██║██╔══██╗╚══██╔══╝${NC}           ${CYAN}║${NC}"
 echo -e "${CYAN}║${NC}  ${MAGENTA}██████╔╝██║     ███████║███████║   ██║   ${NC}           ${CYAN}║${NC}"
 echo -e "${CYAN}║${NC}  ${MAGENTA}██╔═══╝ ██║     ██╔══██║██╔══██║   ██║   ${NC}           ${CYAN}║${NC}"
 echo -e "${CYAN}║${NC}  ${MAGENTA}██║     ╚██████╗██║  ██║██║  ██║   ██║   ${NC}           ${CYAN}║${NC}"
 echo -e "${CYAN}║${NC}  ${MAGENTA}╚═╝      ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ${NC}           ${CYAN}║${NC}"
 echo -e "${CYAN}║${NC}  ${GREEN}Modern Terminal Live Chat Viewer${NC}                    ${CYAN}║${NC}"
 echo -e "${CYAN}╚══════════════════════════════════════════════════════╝${NC}"
 echo ""

# Verificar que existe la carpeta local de temas
if [[ ! -d "$LOCAL_THEMES" ]]; then
    echo -e "${RED}Error: No se encuentra el directorio local 'themes'.${NC}"
    echo -e "Ejecuta este script desde la raíz del proyecto pchat."
    exit 1
fi

echo -e "Este script realizará las siguientes acciones:"
echo -e "  1. Crear ${GREEN}$CONFIG_DIR${NC} (si no existe)"
echo -e "  2. Copiar temas a ${GREEN}$THEMES_DIR${NC}"
echo -e "  3. Generar ${GREEN}$CONFIG_DIR/config.toml${NC} (si no existe)"
echo ""

read -p "¿Quieres continuar? (s/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Ss]$ ]]; then
    echo -e "${YELLOW}Configuración cancelada.${NC}"
    exit 0
fi

echo ""

# Crear directorios
mkdir -p "$CONFIG_DIR"
echo -e "${GREEN}✓ Directorio $CONFIG_DIR creado.${NC}"

mkdir -p "$THEMES_DIR"
echo -e "${GREEN}✓ Directorio $THEMES_DIR creado.${NC}"

# Copiar temas
if ls "$LOCAL_THEMES"/*.toml &> /dev/null; then
    cp "$LOCAL_THEMES"/*.toml "$THEMES_DIR/"
    echo -e "${GREEN}✓ Temas copiados a $THEMES_DIR${NC}"
    echo -e "  Archivos:"
    for theme in "$LOCAL_THEMES"/*.toml; do
        echo -e "    - $(basename "$theme")"
    done
else
    echo -e "${YELLOW}⚠  No se encontraron archivos .toml en $LOCAL_THEMES${NC}"
fi

# Crear config.toml si no existe
CONFIG_FILE="$CONFIG_DIR/config.toml"
if [[ ! -f "$CONFIG_FILE" ]]; then
    cat > "$CONFIG_FILE" <<EOF
# Configuración de pchat
theme = "tokyo-night-enhanced"
default_backend = "mock"
compact = false
EOF
    echo -e "${GREEN}✓ Archivo de configuración creado en $CONFIG_FILE${NC}"
else
    echo -e "${YELLOW}⚠  $CONFIG_FILE ya existe. No se ha modificado.${NC}"
fi

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}  Configuración completada.${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Ahora puedes ejecutar: ${CYAN}pchat --backend mock --theme tokyo-night-enhanced \"https://example.com\"${NC}"
