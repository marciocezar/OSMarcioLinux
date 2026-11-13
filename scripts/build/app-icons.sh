#!/usr/bin/env bash
# Gera os PNGs do ícone de um app OSMarcio a partir do SVG original.
# uso: browser-icons.sh [app]   (padrão: osmarcio-browser)
# Roda DENTRO do container (o rsvg-convert é instalado só no container
# efêmero, nunca no host). Saída versionada em assets/icons/hicolor/.
source "$(dirname "$0")/../lib/common.sh"
APP=${1:-osmarcio-browser}
SRC="$OSM_ROOT/apps/$APP/assets/icons/$APP.svg"
OUT="$OSM_ROOT/apps/$APP/assets/icons/hicolor"
[[ -f $SRC ]] || die "SVG não encontrado: $SRC"
if ! command -v rsvg-convert >/dev/null; then
  log "instalando librsvg2-bin no container efêmero"
  apt-get update -qq >/dev/null && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq librsvg2-bin >/dev/null
fi
for s in 16 24 32 48 64 128 256; do
  d="$OUT/${s}x${s}/apps"; mkdir -p "$d"
  rsvg-convert -w "$s" -h "$s" -f png -o "$d/$APP.png" "$SRC"
done
mkdir -p "$OUT/scalable/apps"; cp "$SRC" "$OUT/scalable/apps/$APP.svg"
file "$OUT"/*/apps/$APP.* | sed "s|$OSM_ROOT/||" >&2
