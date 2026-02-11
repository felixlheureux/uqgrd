#!/usr/bin/env bash
set -euo pipefail

REPO="felixlheureux/uqgrd"
BINARY="uqgrd"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[+]${NC} $*"; }
warn()  { echo -e "${YELLOW}[!]${NC} $*"; }
error() { echo -e "${RED}[✗]${NC} $*"; exit 1; }

# --- Detect OS ---
OS="$(uname -s)"
case "$OS" in
    Linux)  PLATFORM="linux" ;;
    Darwin) PLATFORM="macos" ;;
    *)      error "Unsupported OS: $OS" ;;
esac
info "Detected platform: $PLATFORM"

# --- Check dependencies ---
check_cmd() {
    if ! command -v "$1" &>/dev/null; then
        return 1
    fi
    return 0
}

# Rust
if ! check_cmd cargo; then
    warn "Rust not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    info "Rust installed"
fi
info "Rust: $(rustc --version)"

# OpenSSL / pkg-config
if [ "$PLATFORM" = "linux" ]; then
    if ! pkg-config --exists openssl 2>/dev/null; then
        warn "OpenSSL dev headers not found."
        if check_cmd apt-get; then
            info "Installing libssl-dev via apt..."
            sudo apt-get update -qq && sudo apt-get install -y -qq libssl-dev pkg-config
        elif check_cmd dnf; then
            info "Installing openssl-devel via dnf..."
            sudo dnf install -y openssl-devel pkg-config
        elif check_cmd pacman; then
            info "Installing openssl via pacman..."
            sudo pacman -S --noconfirm openssl pkg-config
        else
            error "Cannot auto-install OpenSSL. Install libssl-dev (or equivalent) manually."
        fi
    fi
elif [ "$PLATFORM" = "macos" ]; then
    if ! check_cmd brew; then
        error "Homebrew is required on macOS. Install it from https://brew.sh"
    fi
    if ! brew list openssl &>/dev/null; then
        info "Installing OpenSSL via Homebrew..."
        brew install openssl
    fi
    # Homebrew OpenSSL is keg-only — cargo needs these to find it
    OPENSSL_PREFIX="$(brew --prefix openssl)"
    export PKG_CONFIG_PATH="$OPENSSL_PREFIX/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
    export OPENSSL_DIR="$OPENSSL_PREFIX"
fi
info "OpenSSL: OK"

# Git
check_cmd git || error "git is required but not installed."

# --- Clone and build ---
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

info "Cloning $REPO..."
git clone --depth 1 "https://github.com/$REPO.git" "$TMPDIR/$BINARY" 2>&1 | tail -1

info "Building (release)... this may take a minute"
cargo build --release --manifest-path "$TMPDIR/$BINARY/Cargo.toml" 2>&1 | tail -5

# --- Install binary ---
mkdir -p "$INSTALL_DIR"
cp "$TMPDIR/$BINARY/target/release/$BINARY" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"
info "Installed $BINARY to $INSTALL_DIR/$BINARY"

# --- Ensure PATH ---
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    warn "$INSTALL_DIR is not in your PATH."

    SHELL_NAME="$(basename "$SHELL")"
    case "$SHELL_NAME" in
        zsh)  RC_FILE="$HOME/.zshrc" ;;
        bash) RC_FILE="$HOME/.bashrc" ;;
        fish) RC_FILE="$HOME/.config/fish/config.fish" ;;
        *)    RC_FILE="" ;;
    esac

    if [ -n "$RC_FILE" ]; then
        if [ "$SHELL_NAME" = "fish" ]; then
            EXPORT_LINE="set -gx PATH $INSTALL_DIR \$PATH"
        else
            EXPORT_LINE="export PATH=\"$INSTALL_DIR:\$PATH\""
        fi

        if ! grep -qF "$INSTALL_DIR" "$RC_FILE" 2>/dev/null; then
            echo "" >> "$RC_FILE"
            echo "# Added by uqgrd installer" >> "$RC_FILE"
            echo "$EXPORT_LINE" >> "$RC_FILE"
            info "Added $INSTALL_DIR to PATH in $RC_FILE"
            warn "Run 'source $RC_FILE' or restart your shell to update PATH"
        fi
    else
        warn "Add this to your shell config: export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
fi

# --- Done ---
echo ""
info "Installation complete! Run '$BINARY --help' to get started."
info "First step: '$BINARY credentials' to set up your UQAM login."
