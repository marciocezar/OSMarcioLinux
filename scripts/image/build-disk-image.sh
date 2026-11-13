#!/usr/bin/env bash
# Wrapper do host: gera rootfs + imagem de disco UEFI dentro do container
# e publica o artefato versionado com manifesto e SHA-256.
#
# uso: build-disk-image.sh <arm64|amd64> <perfil> <plataforma: utm|qemu-x86_64>
source "$(dirname "$0")/../lib/common.sh"

ARCH=${1:?arch}; PROFILE=${2:?perfil}; PLATFORM=${3:?plataforma}
cap() { printf "%s%s" "$(printf %s "${1:0:1}" | tr a-z A-Z)" "${1:1}"; }

case "$PLATFORM/$ARCH" in
  utm/arm64)          FINAL="images/utm/OSMarcioLinux-UTM-AArch64-$(cap "$PROFILE")-v$OSM_VERSION.img" ;;
  qemu-x86_64/amd64)  FINAL="images/virtualbox/OSMarcioLinux-x86_64-$(cap "$PROFILE")-v$OSM_VERSION.img" ;;
  *) die "combinação plataforma/arch não suportada: $PLATFORM/$ARCH" ;;
esac

refuse_overwrite "$OSM_ROOT/$FINAL"
docker image inspect "$OSM_BUILDER_IMAGE" >/dev/null 2>&1 || "$OSM_ROOT/scripts/build/builder-image.sh"

STAMP=$(date -u +%Y%m%dT%H%M%SZ)
OUTDIR="out/$PLATFORM-$PROFILE-$STAMP"
mkdir -p "$OSM_ROOT/$OUTDIR"

log "[1/2] rootfs ($ARCH/$PROFILE)"
in_builder scripts/build/rootfs.sh "$ARCH" "$PROFILE" "$OUTDIR/rootfs.tar"
log "[2/2] imagem de disco"
in_builder scripts/image/mkimage-uefi.sh "$ARCH" "$OUTDIR/rootfs.tar" "$OUTDIR/disk.img"

mkdir -p "$(dirname "$OSM_ROOT/$FINAL")"
mv "$OSM_ROOT/$OUTDIR/disk.img" "$OSM_ROOT/$FINAL"
mv "$OSM_ROOT/$OUTDIR/disk.img.packages.tsv" "$OSM_ROOT/$FINAL.packages.tsv"
"$OSM_ROOT/scripts/release/write-manifest.sh" "$FINAL" "$ARCH" "$PLATFORM" "$PROFILE" "$OSM_ROOT/$OUTDIR/disk.img.build-info"
rm -f "$OSM_ROOT/$OUTDIR/rootfs.tar"
log "artefato: $FINAL"
