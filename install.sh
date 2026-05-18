#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m'

INSTALL_DIR="$HOME/.local/bin"
BIN_NAME="pchat"
SOURCE_BIN="target/release/$BIN_NAME"

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

# Comprobar si ya está instalado
if command -v "$BIN_NAME" &> /dev/null; then
    CURRENT=$(which "$BIN_NAME")
    echo -e "${YELLOW}pchat ya está instalado en: $CURRENT${NC}"
    echo -e "Este script lo reemplazará en $INSTALL_DIR"
    echo ""
fi

# Verificar que el binario compilado existe
if [[ ! -f "$SOURCE_BIN" ]]; then
    echo -e "${YELLOW}No se encontró $SOURCE_BIN. Compilando primero...${NC}"
    cargo build --release
    if [[ ! -f "$SOURCE_BIN" ]]; then
        echo -e "${RED}Error: no se pudo compilar el binario.${NC}"
        exit 1
    fi
fi

echo -e "El binario se copiará a: ${GREEN}$INSTALL_DIR/$BIN_NAME${NC}"
echo -e "Asegúrate de que ${GREEN}$INSTALL_DIR${NC} esté en tu PATH."
echo ""

read -p "¿Proceder con la instalación? (s/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Ss]$ ]]; then
    echo -e "${YELLOW}Instalación cancelada.${NC}"
    exit 0
fi

# Crear directorio si no existe
mkdir -p "$INSTALL_DIR"

# Copiar el binario
cp "$SOURCE_BIN" "$INSTALL_DIR/$BIN_NAME"
chmod +x "$INSTALL_DIR/$BIN_NAME"

echo -e "${GREEN}✓ Instalación completada.${NC}"
echo -e "Binario instalado en: ${CYAN}$INSTALL_DIR/$BIN_NAME${NC}"

# Verificar PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo ""
    echo -e "${YELLOW}⚠  $INSTALL_DIR no está en tu PATH.${NC}"
    echo -e "Añade esta línea a tu ~/.bashrc o ~/.zshrc:"
    echo -e "  ${CYAN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
else
    echo -e "Puedes ejecutarlo con: ${CYAN}pchat --backend mock \"https://example.com\"${NC}"
fi
