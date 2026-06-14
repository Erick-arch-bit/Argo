#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# install.sh — Instalación global del intérprete Argo
# ---------------------------------------------------------------------------

set -euo pipefail

# ─────────────────────────────────────────────────────────────
# COLORES ARGO Y CURSOR
# ─────────────────────────────────────────────────────────────
RST="\x1b[0m"
BLD="\x1b[1m"
DIM="\x1b[2m"
GRN="\x1b[32m"
PRP="\x1b[38;2;147;0;211m"
ORG="\x1b[38;2;255;140;0m"
GRAD1="\x1b[38;2;180;50;200m"
GRAD2="\x1b[38;2;220;100;100m"
GRAD3="\x1b[38;2;255;140;0m"

HIDE="\x1b[?25l"
SHOW="\x1b[?25h"

# ─────────────────────────────────────────────────────────────
# BADGE
# ─────────────────────────────────────────────────────────────
badge() {
    local text="$1"
    local bg="$2"
    echo -e "${bg}\x1b[30m\x1b[4m ${text} ${RST}"
}

# Fondos
PURPLE_BG="\x1b[48;2;147;0;211m"
ORANGE_BG="\x1b[48;2;255;140;0m"
GRAY_BG="\x1b[48;2;80;80;80m"

# ─────────────────────────────────────────────────────────────
# ANIMACIÓN DE CARGA
# ─────────────────────────────────────────────────────────────
animate() {
    local faces=("(◕‿◕)" "(◠‿◠)" "(◕‿◕)" "(⊙‿⊙)" "(◕‿◕)" "(¬‿¬)")
    local actions=(
        "Calibrating tentacles..."
        "Filling ink tanks..."
        "Syncing with nebula..."
        "Warming up reactor..."
        "Polishing shell..."
        "Almost there..."
    )
    
    echo -e "${HIDE}"
    
    for prog in $(seq 0 4 100); do
        local frame=$(( (prog / 4) % ${#faces[@]} ))
        local act=$(( (prog / 4) % ${#actions[@]} ))
        local filled=$(( prog / 5 ))
        local empty=$(( 20 - filled ))
        
        # Color degradado
        local r=$(( 128 + 127 * prog / 100 ))
        local g=$(( 0 + 140 * prog / 100 ))
        local b=$(( 128 - 128 * prog / 100 ))
        local c="\x1b[38;2;${r};${g};${b}m"
        
        # Barra
        local bar=""
        for ((i=0; i<20; i++)); do
            if [ $i -lt $filled ]; then
                local br=$(( 147 + 108 * i / 20 ))
                local bg=$(( 0 + 154 * i / 20 ))
                local bb=$(( 211 - 211 * i / 20 ))
                bar+="\x1b[38;2;${br};${bg};${bb}m█${RST}"
            else
                bar+="░"
            fi
        done
        
        printf "\r  %s%s%s  [%s]  %s%3d%%%s  %s%s%s" \
            "$c" "${faces[$frame]}" "$RST" \
            "$bar" "$c" "$prog" "$RST" \
            "$BLD" "${actions[$act]}" "$RST"
        
        if [ $prog -lt 100 ]; then
            sleep 0.18
        fi
    done
    
    echo ""
    echo -e "\r\x1b[K"
}

# ─────────────────────────────────────────────────────────────
# HEADER
# ─────────────────────────────────────────────────────────────
echo ""
echo -e "  ${BLD}Argo${RST} ${DIM}Launch sequence initiated.${RST}"
echo ""

# ─────────────────────────────────────────────────────────────
# COMPILACIÓN
# ─────────────────────────────────────────────────────────────
echo -e " $(badge "build" "$PURPLE_BG") Compiling Argo in release mode..."
echo ""

cargo build --release 2>&1 | while read -r line; do
    echo -e "  ${DIM}│${RST} $line"
done

echo ""

# ─────────────────────────────────────────────────────────────
# INSTALACIÓN
# ─────────────────────────────────────────────────────────────
INSTALL_DIR="${HOME}/.local/bin"

echo -e " $(badge "install" "$ORANGE_BG") Installing to ${DIM}${INSTALL_DIR}${RST}..."

if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "  ${DIM}│${RST} Creating directory..."
    mkdir -p "$INSTALL_DIR"
fi

cp target/release/argo "$INSTALL_DIR/argo"
chmod +x "$INSTALL_DIR/argo"

echo -e "  ${DIM}│${RST} Binary copied and permissions set"
echo ""

# ─────────────────────────────────────────────────────────────
# ÉXITO
# ─────────────────────────────────────────────────────────────
echo -e "${GRN}✔${RST} ${BLD}Argo installed successfully!${RST}"
echo ""

# Carita + Fair winds
echo -e "  ${GRAD3}(◕‿◕)${RST}  ${DIM}Fair winds:${RST}  ${BLD}May your code sail smooth!${RST}"
echo ""

# Next steps
echo -e " $(badge "next" "$GRAY_BG") ${BLD}Ready to set sail!${RST}"
echo ""
echo -e "  ${DIM}argo init${RST}     Create a new project"
echo -e "  ${DIM}argo repl${RST}     Start interactive session"
echo -e "  ${DIM}argo run${RST}      Execute your program"
echo -e "  ${DIM}argo test${RST}     Run your test suite"
echo -e "  ${DIM}argo build${RST}    Compile for production"
echo ""

# PATH auto‑injection
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    # Detectar perfil
    if [[ -n "$ZSH_VERSION" || -f "$HOME/.zshrc" ]]; then
        PROFILE="$HOME/.zshrc"
    else
        PROFILE="$HOME/.bashrc"
    fi

    if ! grep -q "${INSTALL_DIR}" "$PROFILE" 2>/dev/null; then
        echo "" >> "$PROFILE"
        echo "# Argo" >> "$PROFILE"
        echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$PROFILE"
        echo -e " $(badge "path" "$ORANGE_BG") Auto‑added ${DIM}${INSTALL_DIR}${RST} to ${DIM}${PROFILE}${RST}"
        echo ""
        echo -e "  ${DIM}Run this to use Argo right now:${RST}"
        echo -e "     ${BLD}source ${PROFILE}${RST}"
    else
        echo -e " $(badge "path" "$GRAY_BG") ${DIM}${INSTALL_DIR}${RST} already in ${DIM}${PROFILE}${RST}"
    fi
fi

echo -e ""
echo -e "${SHOW}"