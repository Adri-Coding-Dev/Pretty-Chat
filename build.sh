#!/usr/bin/env bash
set -euo pipefail

# Colores ANSI
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

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

# Verificar que cargo está instalado
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: 'cargo' no está instalado. Instala Rust primero: https://rustup.rs${NC}"
    exit 1
fi

echo -e "Este script compilará pchat en modo ${GREEN}release${NC}."
echo -e "La compilación puede tardar unos minutos."
echo ""

read -p "¿Quieres continuar? (s/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Ss]$ ]]; then
    echo -e "${YELLOW}Compilación cancelada.${NC}"
    exit 0
fi

echo -e "${CYAN}Iniciando compilación...${NC}"
cargo build --release

if [[ $? -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}✓ Compilación exitosa.${NC}"
    echo -e "Binario generado: ${CYAN}target/release/pchat${NC}"
    echo -e "Puedes probarlo con: ${YELLOW}./target/release/pchat --backend mock \"https://example.com\"${NC}"
else
    echo -e "${RED}✗ Error durante la compilación.${NC}"
    exit 1
fi
