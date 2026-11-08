#!/usr/bin/env bash
#
# install.sh — Install Neuro Lab.
#
# Neuro Lab is a Tauri desktop app, not a CLI binary, so "install" means
# obtaining the app bundles. This script tries, in order:
#
#   1. Download the latest GitHub release bundle for this platform, if one
#      has been published (assets are matched by arch/extension, e.g.
#      "Neuro Lab_0.1.0_aarch64.dmg" or "neuro-lab_0.1.0_amd64.AppImage").
#   2. Clone the repository and build from source (same as install-local.sh).
#
# Usage:
#   ./scripts/install.sh
#   INSTALL_DIR=~/Applications ./scripts/install.sh
set -euo pipefail

REPO="hieudoanm/neuro.lab"
INSTALL_DIR="${INSTALL_DIR:-$HOME/Applications}"

OS="$(uname -s)"
ARCH="$(uname -m)"
case "$ARCH" in
  arm64 | aarch64) ARCH="aarch64" ;;
  x86_64 | amd64)  ARCH="x86_64" ;;
esac
OS_LOWER="$(echo "$OS" | tr '[:upper:]' '[:lower:]')"

echo "Detected platform: ${OS_LOWER}-${ARCH}"

# --- 1. Try published release assets ------------------------------------------

# Tauri bundle arch strings differ per platform: macOS uses aarch64/x86_64,
# Linux AppImages use x86_64 (and aarch64 where built), Windows uses x64/arm64.
asset_arch_candidates() {
  case "$1" in
    dmg)        echo "aarch64 x86_64" ;;
    appimage)   echo "x86_64 aarch64" ;;
    deb|rpm)    echo "amd64 x86_64 arm64 aarch64" ;;
    msi|exe)    echo "x64 x86_64 arm64 aarch64" ;;
  esac
}

pick_asset() {
  # $1 = extension filter; prints the best-matching asset name, if any.
  local ext="$1"
  command -v gh >/dev/null 2>&1 || return 1
  gh release view latest --repo "$REPO" --json assets \
    --jq '.assets[].name' 2>/dev/null | while read -r name; do
      [[ "$name" == *."$ext" ]] || continue
      for cand in $(asset_arch_candidates "$ext"); do
        if [[ "$name" == *"$cand"* ]]; then
          echo "$name"
          exit 0
        fi
      done
    done | head -1
}

download_and_install() {
  local name="$1" ext="${1##*.}"
  local url="https://github.com/${REPO}/releases/latest/download/$(python3 -c 'import sys,urllib.parse;print(urllib.parse.quote(sys.argv[1]))' "$name" 2>/dev/null || echo "$name")"
  echo "Downloading release asset: $name"
  curl -fsSL "$url" -o "/tmp/$name"
  mkdir -p "$INSTALL_DIR"
  case "$ext" in
    dmg)
      hdiutil attach "/tmp/$name" -nobrowse -quiet
      local vol
      vol="$(hdiutil info | awk '/\/Volumes\// {print $1}' | tail -1)"
      local app
      app="$(find "${vol:-/Volumes}" -maxdepth 2 -name '*.app' 2>/dev/null | head -1)"
      if [[ -n "$app" ]]; then
        cp -R "$app" "$INSTALL_DIR/"
        hdiutil detach "$vol" -quiet || true
        echo "Installed to $INSTALL_DIR/$(basename "$app")"
        rm -f "/tmp/$name"
        return 0
      fi
      [[ -n "$vol" ]] && hdiutil detach "$vol" -quiet 2>/dev/null || true
      return 1
      ;;
    msi|exe)
      echo "Windows installer downloaded to /tmp/$name — run it to install."
      return 0
      ;;
    *)
      chmod +x "/tmp/$name"
      mv "/tmp/$name" "$INSTALL_DIR/neuro-lab.${ext}"
      echo "Installed to $INSTALL_DIR/neuro-lab.${ext}"
      return 0
      ;;
  esac
}

for ext in dmg appimage deb msi exe; do
  case "$ext" in
    dmg)     [[ "$OS_LOWER" == "darwin" ]] || continue ;;
    appimage|deb) [[ "$OS_LOWER" == "linux" ]] || continue ;;
    msi|exe) [[ "$OS" == "Windows_NT" || "$OS_LOWER" == mingw* || "$OS_LOWER" == msys* || "$OS_LOWER" == cygwin* ]] || continue ;;
  esac
  asset="$(pick_asset "$ext" || true)"
  if [[ -n "$asset" ]]; then
    if download_and_install "$asset"; then
      exit 0
    fi
  fi
done

# --- 2. Fall back to building from source --------------------------------------

echo "No prebuilt release found for ${OS_LOWER}-${ARCH}; building from source."
echo

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

git clone --depth 1 "https://github.com/${REPO}.git" "$TMP_DIR/neuro.lab"
cd "$TMP_DIR/neuro.lab"

exec ./scripts/install-local.sh
