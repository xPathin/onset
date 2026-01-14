#!/usr/bin/env bash
set -euo pipefail

PKGBUILD_URL="https://raw.githubusercontent.com/xPathin/onset/refs/heads/main/PKGBUILD"

echo "==> Installing onset (Arch Linux)"
echo "==> Fetching PKGBUILD"

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT
cd "$workdir"

curl -fsSL "$PKGBUILD_URL" -o PKGBUILD

echo
echo "==> PKGBUILD downloaded to: $workdir/PKGBUILD"
echo "==> You should inspect it before continuing"
echo
read -rp "Continue with build and install? [y/N] " confirm
[[ "$confirm" == "y" || "$confirm" == "Y" ]] || exit 1

makepkg -si --noconfirm