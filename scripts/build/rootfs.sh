#!/usr/bin/env bash
# Gera o root filesystem do OSMarcio Linux com mmdebstrap.
# Roda DENTRO do container de build (ver scripts/lib/common.sh: in_builder).
#
# uso: rootfs.sh <arch: arm64|amd64> <perfil: minimal|desktop|developer> <saida.tar>
source "$(dirname "$0")/../lib/common.sh"

ARCH=${1:?arquitetura}; PROFILE=${2:?perfil}; OUT=${3:?saida}
PROFILE_DIR="$OSM_ROOT/distro/profiles/$PROFILE"
[[ -f $PROFILE_DIR/packages.list ]] || die "perfil inexistente: $PROFILE"

case $ARCH in
  arm64) KPKG=linux-image-arm64; GRUBPKG=grub-efi-arm64 ;;
  amd64) KPKG=linux-image-amd64; GRUBPKG=grub-efi-amd64 ;;
  *) die "arquitetura não suportada: $ARCH" ;;
esac

# Perfis herdam: developer -> desktop -> minimal.
pkgs=()
collect() { grep -vE '^\s*(#|$)' "$OSM_ROOT/distro/profiles/$1/packages.list"; }
case $PROFILE in
  minimal)   mapfile -t pkgs < <(collect minimal) ;;
  desktop)   mapfile -t pkgs < <(collect minimal; collect desktop) ;;
  developer) mapfile -t pkgs < <(collect minimal; collect desktop; collect developer) ;;
esac
pkgs+=("$KPKG" "$GRUBPKG")
INCLUDE=$(IFS=,; echo "${pkgs[*]}")

log "rootfs $ARCH/$PROFILE: ${#pkgs[@]} pacotes (suite $OSM_SUITE)"

mmdebstrap \
  --arch="$ARCH" \
  --variant=minbase \
  --components="main,contrib,non-free-firmware" \
  --include="$INCLUDE" \
  --essential-hook='echo "grub-efi-'"$ARCH"' grub2/update_nvram boolean false" | chroot "$1" debconf-set-selections' \
  --customize-hook="$OSM_ROOT/scripts/build/customize-rootfs.sh \"\$1\" $ARCH $PROFILE" \
  "$OSM_SUITE" "$OUT" \
  "deb $OSM_MIRROR $OSM_SUITE main contrib non-free-firmware" \
  "deb $OSM_MIRROR $OSM_SUITE-updates main contrib non-free-firmware" \
  "deb http://security.debian.org/debian-security $OSM_SUITE-security main contrib non-free-firmware"

log "rootfs gerado: $OUT ($(du -h "$OUT" | cut -f1))"
