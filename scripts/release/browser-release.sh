#!/usr/bin/env bash
# Publica localmente o pacote do OSMarcio Browser em images/releases/ com
# SHA-256. Não sobrescreve; não envia nada para a internet.
source "$(dirname "$0")/../lib/common.sh"
DEB=${1:?deb}
[[ -f $DEB ]] || die "pacote não encontrado: $DEB"
DEST="$OSM_ROOT/images/releases/$(basename "$DEB")"
refuse_overwrite "$DEST"
cp "$DEB" "$DEST"
( cd "$(dirname "$DEST")" && shasum -a 256 "$(basename "$DEST")" > "$(basename "$DEST").sha256" )
log "release local: ${DEST#$OSM_ROOT/} ($(du -h "$DEST" | cut -f1))"
cat "$DEST.sha256"
