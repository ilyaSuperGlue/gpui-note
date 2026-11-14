#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$PROJECT_DIR"

build_mac() {
    echo "==> Building Apple Silicon macOS"
    cargo build \
        --release \
        --locked \
        --target aarch64-apple-darwin

    echo "Artifact: target/aarch64-apple-darwin/release/noters"
}

build_linux() {
    echo "==> Building x86-64 Linux with cargo-zigbuild"
    RUST_FONTCONFIG_DLOPEN=1 cargo zigbuild \
        --release \
        --locked \
        --target x86_64-unknown-linux-gnu

    echo "Artifact: target/x86_64-unknown-linux-gnu/release/noters"
}

build_windows() {
    echo "==> Building x86-64 Windows with cargo-zigbuild"
    cargo zigbuild \
        --release \
        --locked \
        --target x86_64-pc-windows-gnu

    echo "Artifact: target/x86_64-pc-windows-gnu/release/noters.exe"
}

case "${1:-all}" in
    all)
        build_mac
        build_linux
        build_windows
        ;;
    mac)
        build_mac
        ;;
    linux)
        build_linux
        ;;
    windows)
        build_windows
        ;;
    -h|--help)
        echo "Usage: $0 [all|mac|linux|windows]"
        ;;
    *)
        echo "Unknown target: $1" >&2
        echo "Usage: $0 [all|mac|linux|windows]" >&2
        exit 2
        ;;
esac
