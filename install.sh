#!/usr/bin/env bash
# install.sh — Instalación del intérprete Argo
# Detecta automáticamente la plataforma y arquitectura, descarga el
# binario precompilado de GitHub Releases o compila desde fuente.
set -euo pipefail

RST="\x1b[0m"; BLD="\x1b[1m"; DIM="\x1b[2m"; GRN="\x1b[32m"
PRP="\x1b[38;2;147;0;211m"; ORG="\x1b[38;2;255;140;0m"
RED="\x1b[31m"
PURPLE_BG="\x1b[48;2;147;0;211m"; ORANGE_BG="\x1b[48;2;255;140;0m"
GRAY_BG="\x1b[48;2;80;80;80m"

badge() { echo -e "${2}\x1b[30m\x1b[4m ${1} ${RST}"; }

# Detectar plataforma
detectar_plataforma() {
    local os arch
    case "$(uname -s)" in
        Linux)  os="linux" ;;
        Darwin) os="darwin" ;;
        *)      echo "error" && return ;;
    esac
    case "$(uname -m)" in
        x86_64|amd64) arch="amd64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)      echo "error" && return ;;
    esac
    echo "${os}-${arch}"
}

PLATAFORMA=$(detectar_plataforma)
if [ "$PLATAFORMA" = "error" ]; then
    echo -e "${RED}✖${RST} Plataforma no soportada: $(uname -s) $(uname -m)"
    echo "  Compila desde fuente: cargo build --release"
    exit 1
fi

case "$PLATAFORMA" in
    linux-amd64)  BINARIO="argo-linux-amd64" ;;
    linux-arm64)  BINARIO="argo-linux-arm64" ;;
    darwin-amd64) BINARIO="argo-darwin-amd64" ;;
    darwin-arm64) BINARIO="argo-darwin-arm64" ;;
esac

INSTALL_DIR="${HOME}/.local/bin"
VERSION="${VERSION:-latest}"
REPO="Argo-Lang/Argo"

echo ""
echo -e "  ${BLD}Argo${RST} ${DIM}Instalación para ${PLATAFORMA}${RST}"
echo ""

# Intentar descarga precompilada
descargar_y_verificar() {
    local url sha_url
    if [ "$VERSION" = "latest" ]; then
        url="https://github.com/${REPO}/releases/latest/download/${BINARIO}"
        sha_url="https://github.com/${REPO}/releases/latest/download/${BINARIO}.sha256"
    else
        url="https://github.com/${REPO}/releases/download/${VERSION}/${BINARIO}"
        sha_url="https://github.com/${REPO}/releases/download/${VERSION}/${BINARIO}.sha256"
    fi

    echo -e " $(badge "download" "$PURPLE_BG") Descargando ${DIM}${BINARIO}${RST}..."

    local tmpdir
    tmpdir=$(mktemp -d)
    local bin_path="${tmpdir}/${BINARIO}"

    if command -v curl &>/dev/null; then
        curl -sSL -o "$bin_path" "$url" || { rm -rf "$tmpdir"; return 1; }
    elif command -v wget &>/dev/null; then
        wget -q -O "$bin_path" "$url" || { rm -rf "$tmpdir"; return 1; }
    else
        rm -rf "$tmpdir"
        return 1
    fi

    # Verificar SHA256
    if command -v sha256sum &>/dev/null; then
        local expected actual
        expected=$(curl -sSL "$sha_url" 2>/dev/null | awk '{print $1}') || expected=""
        if [ -n "$expected" ]; then
            actual=$(sha256sum "$bin_path" | awk '{print $1}')
            if [ "$expected" != "$actual" ]; then
                echo -e "${RED}✖${RST} Verificación SHA256 fallida"
                rm -rf "$tmpdir"
                return 1
            fi
            echo -e "  ${DIM}│${RST} SHA256 verificada"
        fi
    fi

    chmod +x "$bin_path"
    mkdir -p "$INSTALL_DIR"
    mv "$bin_path" "${INSTALL_DIR}/argo"
    rm -rf "$tmpdir"
    return 0
}

if descargar_y_verificar; then
    echo -e " $(badge "install" "$ORANGE_BG") Binario instalado en ${DIM}${INSTALL_DIR}/argo${RST}"
else
    echo ""
    echo -e " $(badge "build" "$ORANGE_BG") No se pudo descargar; compilando desde fuente..."
    echo ""

    if ! command -v cargo &>/dev/null; then
        echo -e "${RED}✖${RST} Rust/Cargo no está instalado."
        echo "  Instálalo en: https://rustup.rs"
        exit 1
    fi

    cargo build --release 2>&1 | while read -r line; do
        echo -e "  ${DIM}│${RST} $line"
    done

    cp target/release/argo "$INSTALL_DIR/argo"
    chmod +x "$INSTALL_DIR/argo"
    echo -e " $(badge "install" "$ORANGE_BG") Binario compilado en ${DIM}${INSTALL_DIR}/argo${RST}"
fi

echo ""
echo -e "${GRN}✔${RST} ${BLD}Argo instalado exitosamente${RST}"

# PATH injection
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    if [[ -n "$ZSH_VERSION" || -f "$HOME/.zshrc" ]]; then
        PROFILE="$HOME/.zshrc"
    else
        PROFILE="$HOME/.bashrc"
    fi

    if ! grep -q "${INSTALL_DIR}" "$PROFILE" 2>/dev/null; then
        echo "" >> "$PROFILE"
        echo "# Argo" >> "$PROFILE"
        echo "export PATH=\"${INSTALL_DIR}:\$PATH\"" >> "$PROFILE"
        echo -e " $(badge "path" "$ORANGE_BG") PATH añadido en ${DIM}${PROFILE}${RST}"
        echo -e "  Ejecuta: ${BLD}source ${PROFILE}${RST}"
    fi
fi

echo ""
echo -e "  ${GRN}(◕‿◕)${RST}  ${DIM}Fair winds:${RST}  ${BLD}May your code sail smooth!${RST}"
echo ""
