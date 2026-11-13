#!/usr/bin/env bash
# Gera os PNGs do ícone do OSMarcio Browser a partir do SVG original.
# Roda DENTRO do container (o rsvg-convert é instalado só no container
# efêmero, nunca no host). Saída versionada em assets/icons/hicolor/.
source "$(dirname "$0")/../lib/common.sh"
SRC="$OSM_ROOT/apps/osmarcio-browser/assets/icons/osmarcio-browser.svg"
OUT="$OSM_ROOT/apps/osmarcio-browser/assets/icons/hicolor"
[[ -f $SRC ]] || die "SVG não encontrado: $SRC"
if ! command -v rsvg-convert >/dev/null; then
  log "instalando librsvg2-bin no container efêmero"
  apt-get update -qq >/dev/null && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq librsvg2-bin >/dev/null
fi
for s in 16 24 32 48 64 128 256; do
  d="$OUT/${s}x${s}/apps"; mkdir -p "$d"
  rsvg-convert -w "$s" -h "$s" -f png -o "$d/osmarcio-browser.png" "$SRC"
done
mkdir -p "$OUT/scalable/apps"; cp "$SRC" "$OUT/scalable/apps/osmarcio-browser.svg"
file "$OUT"/*/apps/osmarcio-browser.* | sed "s|$OSM_ROOT/||" >&2
